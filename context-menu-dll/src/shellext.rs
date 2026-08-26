//! Windows Shell Extension (COM) for QuickSort.
//!
//! This DLL is loaded by Explorer.exe and provides a cascading context menu.
//! It communicates with the main Tauri app via Named Pipe.

use std::cell::RefCell;
use std::ffi::{c_void, CStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;
use std::{mem, ptr};

use quicksort_ipc_contract::OverwritePolicy;

use parking_lot::Mutex;
use windows::core::{
    implement, w, IUnknown, Interface, Ref as WinRef, Result as WinResult, BOOL, GUID, HRESULT,
    PCWSTR, PSTR, PWSTR,
};
use windows::Win32::Foundation::{
    CLASS_E_NOAGGREGATION, E_FAIL, E_NOINTERFACE, E_NOTIMPL, E_POINTER, HWND, LPARAM, S_OK,
};
use windows::Win32::System::Com::{
    IClassFactory, IClassFactory_Impl, IDataObject, DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL,
};
use windows::Win32::System::Memory::GlobalLock;
use windows::Win32::System::Ole::{ReleaseStgMedium, CF_HDROP};
use windows::Win32::System::Registry::HKEY;
use windows::Win32::UI::Shell::{
    Common::ITEMIDLIST, IContextMenu, IContextMenu_Impl, IShellExtInit, IShellExtInit_Impl,
    SHBrowseForFolderW, SHGetPathFromIDListW, BIF_RETURNONLYFSDIRS, BROWSEINFOW, CMF_DEFAULTONLY,
    CMINVOKECOMMANDINFO, DROPFILES, GCS_VALIDATEA, GCS_VALIDATEW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, InsertMenuItemW, HMENU, MENUITEMINFOW, MFS_ENABLED, MFT_SEPARATOR,
    MIIM_BITMAP, MIIM_FTYPE, MIIM_ID, MIIM_STATE, MIIM_STRING, MIIM_SUBMENU,
};

use crate::icon;
use crate::pipe_client::{move_to_folder, select_folder};

/// Cached app icon bitmap for MIIM_BITMAP on the root "QuickSort" menu entry.
/// Loaded once on first use.
static APP_ICON_BITMAP: OnceLock<Option<usize>> = OnceLock::new();

fn get_app_icon_bitmap() -> Option<windows::Win32::Graphics::Gdi::HBITMAP> {
    let opt = APP_ICON_BITMAP
        .get_or_init(|| icon::load_app_icon_bitmap().map(|bmp| bmp.0.expose_provenance()));
    opt.map(|addr| windows::Win32::Graphics::Gdi::HBITMAP(ptr::with_exposed_provenance_mut(addr)))
}

// ============================================================================
// Logging initialization
// ============================================================================

static LOG_INIT: OnceLock<()> = OnceLock::new();

fn init_logging() {
    LOG_INIT.get_or_init(|| {
        let log_dir = match std::env::var("APPDATA") {
            Ok(appdata) => {
                let mut p = std::path::PathBuf::from(appdata);
                p.push("QuickSort");
                let _ = std::fs::create_dir_all(&p);
                p.push("quicksort_dll.log");
                p
            }
            Err(_) => std::env::current_exe()
                .unwrap_or_default()
                .with_file_name("quicksort_dll.log"),
        };

        if let Ok(file) = std::fs::File::create(&log_dir) {
            let config = simplelog::ConfigBuilder::new()
                .add_filter_allow_str("context_menu_dll")
                .build();
            let _ = simplelog::WriteLogger::init(simplelog::LevelFilter::Debug, config, file);
            log::info!("DLL logging started.");
        }
    });
}

// ============================================================================
// COM class: QuickSortShellExt
// ============================================================================

pub static INSTANCE_COUNT: AtomicU32 = AtomicU32::new(0);
pub const CLSID_QUICKSORT: GUID = GUID::from_u128(0x12345678_1234_1234_1234_1234567890AB);

// Simple folder struct for menu building
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MenuFolder {
    id: String,
    name: String,
    path: String,
    is_favorite: bool,
    color: Option<String>, // e.g. "#FF5733"
}

#[implement(IShellExtInit, IContextMenu)]
pub struct QuickSortShellExt {
    item_paths: RefCell<Vec<PathBuf>>,
    folders: Mutex<Vec<MenuFolder>>,
    min_cmd_id: std::cell::Cell<u32>,
}

impl Default for QuickSortShellExt {
    fn default() -> Self {
        INSTANCE_COUNT.fetch_add(1, Ordering::SeqCst);
        init_logging();

        Self {
            item_paths: Default::default(),
            folders: Mutex::new(Vec::new()),
            min_cmd_id: std::cell::Cell::new(0),
        }
    }
}

impl Drop for QuickSortShellExt {
    fn drop(&mut self) {
        INSTANCE_COUNT.fetch_sub(1, Ordering::SeqCst);
    }
}

// ============================================================================
// IShellExtInit implementation
// ============================================================================

impl IShellExtInit_Impl for QuickSortShellExt_Impl {
    fn Initialize(
        &self,
        _folder_idl: *const ITEMIDLIST,
        data_obj: WinRef<'_, IDataObject>,
        _prog_id: HKEY,
    ) -> WinResult<()> {
        log::info!(
            "IShellExtInit::Initialize called (folder_idl present: {}, has_data_obj: {})",
            !_folder_idl.is_null(),
            data_obj.as_ref().is_some()
        );
        let paths = match data_obj.as_ref() {
            Some(obj) => match extract_files_from_dataobject(obj) {
                Ok(p) => p,
                Err(e) => {
                    log::error!("Failed to extract files from IDataObject: {:?}", e);
                    return Err(e);
                }
            },
            None => {
                log::warn!("IDataObject is null — proceeding with empty selection");
                Vec::new()
            }
        };
        log::info!("Initialize: got {} paths", paths.len());
        self.this.item_paths.replace(paths);
        Ok(())
    }
}

// ============================================================================
// Helper functions for file extraction
// ============================================================================

unsafe fn dropfiles_to_paths(files: &DROPFILES) -> Vec<PathBuf> {
    let mut res = Vec::new();
    let is_wide = files.fWide.as_bool();
    let mut str_ptr = files as *const DROPFILES as *const u8;
    str_ptr = str_ptr.add(files.pFiles as usize);

    loop {
        if is_wide {
            if *(str_ptr as *const u16) == 0 {
                break;
            }
        } else {
            if *str_ptr == 0 {
                break;
            }
        }

        let (bytes_shift, path) = if is_wide {
            let s = PCWSTR(str_ptr as *const u16);
            let len = s.len();
            (
                2 * (len + 1),
                PathBuf::from(OsString::from_wide(s.as_wide())),
            )
        } else {
            let s = CStr::from_ptr(str_ptr as *const i8);
            let bytes = s.to_bytes();
            (
                bytes.len() + 1,
                PathBuf::from(String::from_utf8_lossy(bytes).into_owned()),
            )
        };
        res.push(path);
        str_ptr = str_ptr.add(bytes_shift);
    }
    res
}

fn extract_files_from_dataobject(data_obj: &IDataObject) -> WinResult<Vec<PathBuf>> {
    let fmt = FORMATETC {
        cfFormat: CF_HDROP.0,
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
        ptd: ptr::null_mut(),
    };

    let mut storage = unsafe { data_obj.GetData(&fmt) }?;
    let global = unsafe { storage.u.hGlobal };

    if global.is_invalid() {
        unsafe { ReleaseStgMedium(&mut storage) };
        return Err(E_POINTER.into());
    }

    let lock = unsafe { GlobalLock(global) };
    if lock.is_null() {
        unsafe { ReleaseStgMedium(&mut storage) };
        return Err(E_POINTER.into());
    }

    let files = unsafe { &*(lock as *const DROPFILES) };
    let files_list = unsafe { dropfiles_to_paths(files) };

    unsafe { windows::Win32::System::Memory::GlobalUnlock(global) }.ok();
    unsafe { ReleaseStgMedium(&mut storage) };

    Ok(files_list)
}

// ============================================================================
// IContextMenu implementation
// ============================================================================

fn make_menu_item_with_icon(
    id: u32,
    text: &[u16],
    icon: Option<windows::Win32::Graphics::Gdi::HBITMAP>,
) -> MENUITEMINFOW {
    let len = text.len().saturating_sub(1);
    let mut f_mask = MIIM_ID | MIIM_STATE | MIIM_STRING;
    let mut icon_bmp = None;

    if let Some(bmp) = icon {
        f_mask |= MIIM_BITMAP;
        icon_bmp = Some(bmp);
    }

    MENUITEMINFOW {
        cbSize: mem::size_of::<MENUITEMINFOW>() as u32,
        fMask: f_mask,
        wID: id,
        fState: MFS_ENABLED,
        dwTypeData: PWSTR::from_raw(text.as_ptr() as *mut _),
        cch: len as u32,
        hbmpItem: icon_bmp.unwrap_or_default(),
        ..Default::default()
    }
}

fn make_colored_menu_item(id: u32, text: &[u16], color_hex: Option<&str>) -> MENUITEMINFOW {
    let len = text.len().saturating_sub(1);
    let mut f_mask = MIIM_ID | MIIM_STATE | MIIM_STRING;
    let mut circle_bmp = None;

    if let Some(hex) = color_hex {
        if let Some(colorref) = icon::parse_color_to_colorref(hex) {
            if let Some(bmp) = icon::create_colored_circle_bitmap(colorref) {
                log::debug!(
                    "Colored circle created for '{}' color={}: bitmap handle={}",
                    String::from_utf16_lossy(text),
                    hex,
                    bmp.0 as usize
                );
                f_mask |= MIIM_BITMAP;
                circle_bmp = Some(bmp);
            } else {
                log::warn!(
                    "Failed to create colored circle for '{}' color={}",
                    String::from_utf16_lossy(text),
                    hex
                );
            }
        }
    }

    MENUITEMINFOW {
        cbSize: mem::size_of::<MENUITEMINFOW>() as u32,
        fMask: f_mask,
        wID: id,
        fState: MFS_ENABLED,
        dwTypeData: PWSTR::from_raw(text.as_ptr() as *mut _),
        cch: len as u32,
        hbmpItem: circle_bmp.unwrap_or_default(),
        ..Default::default()
    }
}

fn make_separator(id: u32) -> MENUITEMINFOW {
    MENUITEMINFOW {
        cbSize: mem::size_of::<MENUITEMINFOW>() as u32,
        fMask: MIIM_ID | MIIM_FTYPE,
        wID: id,
        fType: MFT_SEPARATOR,
        ..Default::default()
    }
}

impl IContextMenu_Impl for QuickSortShellExt_Impl {
    fn QueryContextMenu(
        &self,
        menu: HMENU,
        menu_index: u32,
        min_cmd_id: u32,
        max_cmd_id: u32,
        flags: u32,
    ) -> HRESULT {
        log::info!(
            "QueryContextMenu called: menu_index={}, min_cmd_id={}, max_cmd_id={}, flags=0x{:x}",
            menu_index,
            min_cmd_id,
            max_cmd_id,
            flags
        );

        if flags & CMF_DEFAULTONLY != 0 {
            log::info!("QueryContextMenu: CMF_DEFAULTONLY set, returning S_OK");
            return S_OK;
        }

        // Skip menu in system dialogs (Open/Save) where IDataObject has no real files.
        // AllFilesystemObjects is broad; this check prevents showing QuickSort in
        // file pickers, common dialogs, and other non-Explorer contexts.
        {
            let paths = self.this.item_paths.borrow();
            if paths.is_empty() {
                log::info!("QueryContextMenu: no item paths, skipping (system dialog?)");
                return S_OK;
            }
        }

        let folders = match load_folders_from_json() {
            Ok(f) => f,
            Err(e) => {
                log::error!("Failed to load folders: {}", e);
                return E_FAIL;
            }
        };

        self.this.min_cmd_id.set(min_cmd_id);
        *self.this.folders.lock() = folders.clone();

        let favorites: Vec<&MenuFolder> = folders.iter().filter(|f| f.is_favorite).collect();
        let all_folders: Vec<&MenuFolder> = folders.iter().collect();
        let has_all_folders_entry = !all_folders.is_empty();

        let mut used = 0u32;
        // Reserve slots: separator + "Все папки..." (conditional) + "Выбрать путь..." (always)
        let bottom_items = (if has_all_folders_entry { 2 } else { 0 }) + 1;
        let available = max_cmd_id.saturating_sub(min_cmd_id) + 1;

        let max_fav = std::cmp::min(
            favorites.len() as u32,
            available.saturating_sub(bottom_items),
        );

        unsafe {
            let h_submenu = CreatePopupMenu().unwrap();
            let mut current_id = min_cmd_id;

            // Submenu items: favorites with colored circles
            for folder in favorites.iter().take(max_fav as usize) {
                let label = format!("\u{2605} {}", folder.name);
                let wide: Vec<u16> = OsString::from(&label)
                    .encode_wide()
                    .chain(Some(0))
                    .collect();
                let _ = InsertMenuItemW(
                    h_submenu,
                    0xFFFFFFFF,
                    true,
                    &make_colored_menu_item(current_id, &wide, folder.color.as_deref()),
                );
                current_id += 1;
                used += 1;
            }

            // Separator — only when we have favorites and at least one bottom item
            if !favorites.is_empty() && (has_all_folders_entry || true) && used < available {
                let _ = InsertMenuItemW(h_submenu, 0xFFFFFFFF, true, &make_separator(current_id));
                current_id += 1;
                used += 1;
            }

            // "All folders..." entry
            if has_all_folders_entry && used < available {
                let all_wide: Vec<u16> = w!("Все папки...")
                    .as_wide()
                    .iter()
                    .copied()
                    .chain(Some(0))
                    .collect();
                let _ = InsertMenuItemW(
                    h_submenu,
                    0xFFFFFFFF,
                    true,
                    &make_colored_menu_item(current_id, &all_wide, None),
                );
                current_id += 1;
                used += 1;
            }

            // "Choose path..." entry — always available
            if used < available {
                let choose_wide: Vec<u16> = w!("Выбрать путь...")
                    .as_wide()
                    .iter()
                    .copied()
                    .chain(Some(0))
                    .collect();
                let _ = InsertMenuItemW(
                    h_submenu,
                    0xFFFFFFFF,
                    true,
                    &make_colored_menu_item(current_id, &choose_wide, None),
                );
                current_id += 1;
                used += 1;
            }

            // Root "QuickSort" entry with app icon
            let root_wide: Vec<u16> = w!("QuickSort")
                .as_wide()
                .iter()
                .copied()
                .chain(Some(0))
                .collect();
            let root_item = make_menu_item_with_icon(
                current_id, // id after all submenu items
                &root_wide,
                get_app_icon_bitmap(),
            );
            let _ = InsertMenuItemW(
                menu,
                menu_index,
                true,
                &MENUITEMINFOW {
                    fMask: MIIM_ID | MIIM_STATE | MIIM_STRING | MIIM_BITMAP | MIIM_SUBMENU,
                    hSubMenu: h_submenu,
                    ..root_item
                },
            );

            HRESULT(used as i32)
        }
    }

    fn InvokeCommand(&self, info: *const CMINVOKECOMMANDINFO) -> WinResult<()> {
        log::info!("InvokeCommand called");
        if info.is_null() {
            log::error!("InvokeCommand: info is null");
            return E_POINTER.ok();
        }
        let ici = unsafe { *info };
        // Explorer sends the wID from InsertMenuItemW in the low 16 bits of lpVerb.
        // The dispatch below uses wID-based lookup, not position-based.
        let verb = (ici.lpVerb.0 as usize) & 0xFFFF;

        let folders = self.this.folders.lock();
        let favorites: Vec<&MenuFolder> = folders.iter().filter(|f| f.is_favorite).collect();
        let max_fav = favorites.len();

        let sources: Vec<String> = self
            .this
            .item_paths
            .borrow()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        if sources.is_empty() {
            log::warn!("No files selected");
            return E_FAIL.ok();
        }

        log::info!(
            "InvokeCommand: verb={}, max_fav={}, sources={}",
            verb,
            max_fav,
            sources.len()
        );

        // Position 0..max_fav-1 → favorite folder
        if verb < max_fav {
            let target = favorites[verb];
            log::info!("Moving to: {} ({})", target.name, target.id);

            let target_id = target.id.clone();
            std::thread::spawn(move || {
                match move_to_folder(sources, target_id, OverwritePolicy::Skip) {
                    Ok(resp) => {
                        log::info!("Move OK: {:?}", resp);
                    }
                    Err(e) => {
                        log::error!("Move failed: {}", e);
                    }
                }
            });
        // Position max_fav = separator (not clickable)
        // Position max_fav+1 = "All folders" (only if has_all_folders_entry)
        // Position max_fav+2 (or max_fav+1) = "Choose path"
        } else {
            // Calculate the ID for "All folders" and "Choose path"
            let has_folders = !folders.is_empty();
            let all_folders_id = max_fav + 1; // separator + 1
            let choose_path_id = if has_folders {
                all_folders_id + 1
            } else {
                all_folders_id
            };

            if verb == all_folders_id && has_folders {
                // "Все папки..." — open folder selector via IPC pipe
                let source_clone = sources.clone();
                std::thread::spawn(move || match select_folder(source_clone) {
                    Ok(resp) => {
                        log::info!("SelectFolder OK: {:?}", resp);
                    }
                    Err(e) => {
                        log::error!("SelectFolder failed: {}", e);
                    }
                });
            } else if verb == choose_path_id {
                // "Выбрать путь..." — open native folder picker
                self.handle_choose_path(sources);
            }
        }

        Ok(())
    }

    fn GetCommandString(
        &self,
        _cmd_id: usize,
        flags: u32,
        _reserved: *const u32,
        _name_out: PSTR,
        _name_out_len: u32,
    ) -> WinResult<()> {
        match flags {
            GCS_VALIDATEA | GCS_VALIDATEW => S_OK,
            _ => E_NOTIMPL,
        }
        .ok()
    }
}

// ============================================================================
// "Выбрать путь..." handler — native folder picker + IPC move
// ============================================================================

/// Shows the native Windows folder picker and sends the selected files
/// to the server via the IPC pipe, so the operation is recorded in history.
impl QuickSortShellExt_Impl {
    fn handle_choose_path(&self, sources: Vec<String>) {
        let title: Vec<u16> = unsafe {
            w!("Выберите папку назначения")
                .as_wide()
                .iter()
                .copied()
                .chain(Some(0))
                .collect()
        };

        let browse_info = BROWSEINFOW {
            hwndOwner: HWND(ptr::null_mut()),
            pidlRoot: ptr::null_mut(),
            pszDisplayName: PWSTR::null(),
            lpszTitle: PCWSTR::from_raw(title.as_ptr()),
            ulFlags: BIF_RETURNONLYFSDIRS,
            lpfn: None,
            lParam: LPARAM(0),
            iImage: 0,
        };

        unsafe {
            let pidl = SHBrowseForFolderW(&browse_info);
            if pidl.is_null() {
                log::info!("ChoosePath: user cancelled folder picker");
                return;
            }

            let mut path_buf = [0u16; 260];
            if SHGetPathFromIDListW(pidl, &mut path_buf).as_bool() {
                let target_dir = String::from_utf16_lossy(
                    &path_buf[..path_buf.iter().position(|&c| c == 0).unwrap_or(260)],
                )
                .to_string();
                log::info!("ChoosePath: target directory = {}", target_dir);

                // Send the move command via IPC pipe so it is recorded in history.
                let sources_clone = sources.clone();
                let target = target_dir.clone();
                std::thread::spawn(move || {
                    match crate::pipe_client::client::move_to_path(
                        sources_clone,
                        target,
                        quicksort_ipc_contract::OverwritePolicy::AutoRename,
                    ) {
                        Ok(resp) => {
                            log::info!(
                                "ChoosePath: server responded: status={:?}, msg={}",
                                resp.status,
                                resp.message
                            );
                        }
                        Err(e) => {
                            log::error!("ChoosePath: IPC call failed: {:?}", e);
                        }
                    }
                });
            } else {
                log::warn!("ChoosePath: SHGetPathFromIDListW failed");
            }

            // Free the PIDL allocated by SHBrowseForFolderW.
            windows::Win32::System::Com::CoTaskMemFree(Some(pidl as *const _));
        }
    }
}

// ============================================================================
// Helper: load folders from JSON (temporary, will be replaced)
// ============================================================================

fn load_folders_from_json() -> Result<Vec<MenuFolder>, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA not set".to_string())?;
    let mut path = PathBuf::from(appdata);
    path.push("QuickSort");
    path.push("folders.json");

    if !path.exists() {
        return Ok(vec![]);
    }

    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;

    #[derive(serde::Deserialize)]
    struct ConfigFile {
        folders: Vec<FolderData>,
    }

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct FolderData {
        id: String,
        name: String,
        path: String,
        #[serde(alias = "is_favorite")]
        favorite: bool,
        #[serde(alias = "sort_order")]
        order: i32,
        color: Option<String>,
    }

    let config: ConfigFile =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;

    let folders = config
        .folders
        .into_iter()
        .map(|f| MenuFolder {
            id: f.id,
            name: f.name,
            path: f.path,
            is_favorite: f.favorite,
            color: f.color,
        })
        .collect();

    Ok(folders)
}

// ============================================================================
// ClassFactory
// ============================================================================

#[implement(IClassFactory)]
#[derive(Default)]
pub struct QuickSortClassFactory;

impl IClassFactory_Impl for QuickSortClassFactory_Impl {
    fn CreateInstance(
        &self,
        outer: WinRef<'_, IUnknown>,
        iface_id: *const GUID,
        obj_out: *mut *mut c_void,
    ) -> WinResult<()> {
        if outer.is_some() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }

        unsafe {
            *obj_out = ptr::null_mut();
        }

        match unsafe { *iface_id } {
            IUnknown::IID => {
                unsafe {
                    *obj_out = IUnknown::from(QuickSortShellExt::default()).into_raw();
                }
                Ok(())
            }
            IShellExtInit::IID => {
                unsafe {
                    *obj_out = IShellExtInit::from(QuickSortShellExt::default()).into_raw();
                }
                Ok(())
            }
            IContextMenu::IID => {
                unsafe {
                    *obj_out = IContextMenu::from(QuickSortShellExt::default()).into_raw();
                }
                Ok(())
            }
            _ => Err(E_NOINTERFACE.into()),
        }
    }

    fn LockServer(&self, lock: BOOL) -> WinResult<()> {
        if lock.as_bool() {
            INSTANCE_COUNT.fetch_add(1, Ordering::SeqCst);
        } else {
            INSTANCE_COUNT.fetch_sub(1, Ordering::SeqCst);
        }
        Ok(())
    }
}
