use std::cell::RefCell;
use std::ffi::{CStr, OsString, c_void};
use std::mem::MaybeUninit;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::{mem, ptr};
use std::sync::OnceLock;

use parking_lot::Mutex;
use quicksort::folder::repository::{JsonRepository, FolderRepository};
use quicksort::models::Folder;
use quicksort::move_engine::MoveEngine;
use windows::core::{BOOL, GUID, HRESULT, IUnknown, Interface, PCWSTR, PSTR, PWSTR, Ref as WinRef, Result as WinResult, implement};
use windows::Win32::Foundation::{CLASS_E_NOAGGREGATION, E_FAIL, E_NOINTERFACE, E_NOTIMPL, E_POINTER, S_OK, HMODULE};
use windows::Win32::System::Com::{IClassFactory, IClassFactory_Impl, IDataObject, FORMATETC, DVASPECT_CONTENT, TYMED_HGLOBAL, CoTaskMemFree};
use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleExW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT};
use windows::Win32::System::Memory::GlobalLock;
use windows::Win32::System::Ole::{ReleaseStgMedium, CF_HDROP};
use windows::Win32::System::Registry::HKEY;
use windows::Win32::UI::Shell::Common::{ITEMIDLIST, STRRET, STRRET_WSTR};
use windows::Win32::UI::Shell::{
    CMF_DEFAULTONLY, CMINVOKECOMMANDINFO, DROPFILES, GCS_VALIDATEA,
    GCS_VALIDATEW, IContextMenu, IContextMenu_Impl, IShellExtInit,
    IShellExtInit_Impl, SHGDN_FORPARSING, SHGDN_NORMAL, SHGDNF, SHGetDesktopFolder, StrRetToStrW,
};
use windows::Win32::UI::WindowsAndMessaging::{HMENU, InsertMenuItemW, MENUITEMINFOW, MFS_ENABLED, MIIM_ID, MIIM_STATE, MIIM_STRING, CreatePopupMenu, InsertMenuW, MF_BYPOSITION, MF_POPUP, MessageBoxW, MB_OK};
use windows_core::w;

pub(crate) static INSTANCE_COUNT: AtomicU32 = AtomicU32::new(0);
pub const CLSID_QUICKSORT: GUID = GUID::from_u128(0x12345678_1234_1234_1234_1234567890AB);

#[implement(IShellExtInit, IContextMenu)]
pub struct QuickSortShellExt {
    item_paths: RefCell<Vec<PathBuf>>,
    folders: Mutex<Vec<Folder>>,
}

impl QuickSortShellExt {
    // pub const CLS_ID: GUID = GUID::from_u128(0x12345678_1234_1234_1234_1234567890AB);
}

static LOG_INIT: OnceLock<()> = OnceLock::new();

impl Default for QuickSortShellExt {
    fn default() -> Self {
        INSTANCE_COUNT.fetch_add(1, Ordering::SeqCst);

        LOG_INIT.get_or_init(|| {
            let log_dir = match std::env::var("APPDATA") {
                Ok(appdata) => {
                    let mut p = std::path::PathBuf::from(appdata);
                    p.push("QuickSort");
                    let _ = std::fs::create_dir_all(&p);
                    p.push("quicksort_dll.log");
                    p
                }
                Err(_) => std::env::current_exe().unwrap_or_default().with_file_name("quicksort_dll.log"),
            };

            if let Ok(file) = std::fs::File::create(&log_dir) {
                let config = simplelog::ConfigBuilder::new().add_filter_allow_str("quicksort").build();
                let _ = simplelog::WriteLogger::init(simplelog::LevelFilter::Debug, config, file);
                log::info!("DLL logging started.");
            }
        });

        Self {
            item_paths: Default::default(),
            folders: Mutex::new(Vec::new()),
        }
    }
}

impl Drop for QuickSortShellExt {
    fn drop(&mut self) {
        INSTANCE_COUNT.fetch_sub(1, Ordering::SeqCst);
    }
}

// ------------------- IShellExtInit -------------------
impl IShellExtInit_Impl for QuickSortShellExt_Impl {
    fn Initialize(
        &self,
        folder_idl: *const ITEMIDLIST,
        data_obj: WinRef<'_, IDataObject>,
        _prog_id: HKEY,
    ) -> WinResult<()> {
        let paths = if let Some(data_obj) = data_obj.as_ref() {
            extract_files_from_dataobject(data_obj)?
        } else {
            if folder_idl.is_null() {
                return E_POINTER.ok();
            }
            vec![itemidlist_to_path(folder_idl)?]
        };
        self.this.item_paths.replace(paths);
        Ok(())
    }
}

// ------------------- Вспомогательные функции -------------------
unsafe fn dropfiles_to_paths(files: &DROPFILES) -> Vec<PathBuf> {
    let mut res = Vec::new();
    let is_wide = files.fWide.as_bool();
    let mut str_ptr = unsafe { ptr::from_ref(files).cast::<u8>().add(files.pFiles as usize) };

    loop {
        if is_wide {
            if unsafe { str_ptr.cast::<u16>().read() } == 0 { break; }
        } else {
            if unsafe { str_ptr.read() } == 0 { break; }
        }

        let (bytes_shift, path) = if is_wide {
            let s = PCWSTR(str_ptr.cast::<u16>());
            let len = unsafe { s.len() };
            (2 * (len + 1), PathBuf::from(OsString::from_wide(unsafe { s.as_wide() })))
        } else {
            let s = unsafe { CStr::from_ptr(str_ptr.cast::<i8>()) };
            let bytes = s.to_bytes();
            (bytes.len() + 1, PathBuf::from(String::from_utf8_lossy(bytes).into_owned()))
        };
        res.push(path);
        str_ptr = unsafe { str_ptr.add(bytes_shift) };
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

fn itemidlist_to_path(item_list: *const ITEMIDLIST) -> WinResult<PathBuf> {
    let shell_folder = unsafe { SHGetDesktopFolder() }?;
    let mut name = STRRET { uType: STRRET_WSTR.0 as u32, ..Default::default() };

    unsafe { shell_folder.GetDisplayNameOf(item_list, SHGDNF(SHGDN_NORMAL.0 | SHGDN_FORPARSING.0), &mut name) }?;

    let mut path_ptr = MaybeUninit::<PWSTR>::uninit();
    unsafe { StrRetToStrW(&mut name as *mut STRRET, None, path_ptr.as_mut_ptr()) }?;
    let path_ptr = unsafe { path_ptr.assume_init() };

    let path_os = OsString::from_wide(unsafe { path_ptr.as_wide() });
    unsafe { CoTaskMemFree(Some(path_ptr.as_ptr().cast())) };

    Ok(path_os.into())
}

// ------------------- IContextMenu -------------------
impl IContextMenu_Impl for QuickSortShellExt_Impl {
    fn QueryContextMenu(
        &self,
        menu: HMENU,
        menu_index: u32,
        min_cmd_id: u32,
        max_cmd_id: u32,
        flags: u32,
    ) -> HRESULT {
        log::info!("QueryContextMenu called");
        // или
        unsafe { MessageBoxW(None, w!("QueryContextMenu called!"), w!("QuickSort"), MB_OK); }
        if flags & CMF_DEFAULTONLY != 0 {
            return S_OK;
        }

        let repo = match JsonRepository::new() {
            Ok(r) => r,
            Err(e) => { log::error!("JsonRepository::new failed: {:?}", e); return E_FAIL; }
        };
        let config = match repo.load() {
            Ok(c) => c,
            Err(e) => { log::error!("load config failed: {:?}", e); return E_FAIL; }
        };

        let folders = config.folders.clone();
        *self.this.folders.lock() = folders.clone();

        unsafe {
            let h_submenu = CreatePopupMenu().unwrap();
            let mut current_id = min_cmd_id;
            let max_id = max_cmd_id;

            for folder in folders.iter().filter(|f| f.favorite) {
                if current_id > max_id { break; }
                let wide_name: Vec<u16> = std::ffi::OsString::from(&folder.name).encode_wide().chain(Some(0)).collect();
                let item = MENUITEMINFOW {
                    cbSize: mem::size_of::<MENUITEMINFOW>() as u32,
                    fMask: MIIM_ID | MIIM_STATE | MIIM_STRING,
                    wID: current_id,
                    fState: MFS_ENABLED,
                    dwTypeData: PWSTR::from_raw(wide_name.as_ptr() as *mut _),
                    cch: (wide_name.len() - 1) as u32,
                    ..Default::default()
                };
                let _ = InsertMenuItemW(h_submenu, 0xFFFFFFFF, true, &item);
                current_id += 1;
            }

            if current_id <= max_id {
                let other_text = windows::core::w!("📂 Другие папки...");
                let item = MENUITEMINFOW {
                    cbSize: mem::size_of::<MENUITEMINFOW>() as u32,
                    fMask: MIIM_ID | MIIM_STATE | MIIM_STRING,
                    wID: current_id,
                    fState: MFS_ENABLED,
                    dwTypeData: PWSTR::from_raw(other_text.as_ptr() as *mut _),
                    cch: other_text.len() as u32,
                    ..Default::default()
                };
                let _ = InsertMenuItemW(h_submenu, 0xFFFFFFFF, true, &item);
                current_id += 1;
            }

            let root_text = windows::core::w!("QuickSort");
            let _ = InsertMenuW(menu, menu_index, MF_BYPOSITION | MF_POPUP, h_submenu.0 as usize, root_text);

            HRESULT((current_id - min_cmd_id) as i32)
        }
    }

    fn InvokeCommand(&self, info: *const CMINVOKECOMMANDINFO) -> WinResult<()> {
        if info.is_null() {
            return E_POINTER.ok();
        }
        let ici = unsafe { *info };
        let verb = (ici.lpVerb.0 as usize) & 0xFFFF;

        let folders = self.this.folders.lock();
        let favorites: Vec<&Folder> = folders.iter().filter(|f| f.favorite).collect();
        let total_fav = favorites.len();

        if verb < total_fav {
            let target = &favorites[verb].path;
            for path in self.this.item_paths.borrow().iter() {
                if let Err(e) = MoveEngine::move_file(path, target) {
                    log::error!("Move failed: {:?}", e);
                }
            }
        } else if verb == total_fav {
            for path in self.this.item_paths.borrow().iter() {
                let exe_path = get_quicksort_exe_path();
                let exe_path_wide: Vec<u16> = exe_path.as_os_str().encode_wide().chain(Some(0)).collect();

                let file_arg = format!("\"{}\"", path.display());
                let params = format!("select-folder --file {}", file_arg);
                let params_wide: Vec<u16> = std::ffi::OsString::from(&params)
                    .encode_wide()
                    .chain(Some(0))
                    .collect();

                unsafe {
                    let _ = windows::Win32::UI::Shell::ShellExecuteW(
                        None,
                        windows::core::w!("open"),
                        windows::core::PCWSTR(exe_path_wide.as_ptr()),
                        windows::core::PCWSTR(params_wide.as_ptr()),
                        None,
                        windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
                    );
                }
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
        }.ok()
    }
}

fn get_quicksort_exe_path() -> PathBuf {
    let mut path = [0u16; 260];
    let len = unsafe { GetModuleFileNameW(None, &mut path) };
    if len > 0 {
        let dll_path = OsString::from_wide(&path[..len as usize])
            .to_string_lossy()
            .to_string();
        PathBuf::from(dll_path).with_file_name("quicksort.exe")
    } else {
        PathBuf::from("quicksort.exe")
    }
}

// ------------------- ClassFactory -------------------
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

        unsafe { *obj_out = ptr::null_mut(); }

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