use std::cell::RefCell;
use std::ffi::{CStr, OsString, c_void};
use std::mem::{ManuallyDrop, MaybeUninit};
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::{mem, ptr};
use std::sync::OnceLock;

use parking_lot::Mutex;
use quicksort_lib::folder::repository::JsonRepository;
use quicksort_lib::models::Folder;
use quicksort_lib::move_engine::MoveEngine;
use windows::core::{BOOL, GUID, HRESULT, IUnknown, Interface, Owned, PWSTR, Ref as WinRef, Result as WinResult, implement};
use windows::Win32::Foundation::{CLASS_E_NOAGGREGATION, E_FAIL, E_NOINTERFACE, E_NOTIMPL, E_POINTER, S_OK, SEVERITY_SUCCESS};
use windows::Win32::System::Com::{IClassFactory, IClassFactory_Impl, IDataObject, FORMATETC, DVASPECT_CONTENT, TYMED_HGLOBAL};
use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
use windows::Win32::System::Ole::CF_HDROP;
use windows::Win32::System::Registry::HKEY;
use windows::Win32::UI::Shell::Common::{ITEMIDLIST, STRRET, STRRET_WSTR};
use windows::Win32::UI::Shell::{
    CMF_DEFAULTONLY, CMINVOKECOMMANDINFO, DROPFILES, GCS_HELPTEXTA, GCS_HELPTEXTW, GCS_VALIDATEA,
    GCS_VALIDATEW, GCS_VERBA, GCS_VERBW, IContextMenu, IContextMenu_Impl, IShellExtInit,
    IShellExtInit_Impl, SHGDN_FORPARSING, SHGDN_NORMAL, SHGDNF, SHGetDesktopFolder, StrRetToStrW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    HMENU, InsertMenuItemW, MENUITEMINFOW, MFS_ENABLED, MIIM_ID, MIIM_STATE, MIIM_STRING,
    CreatePopupMenu, InsertMenuW, MF_BYPOSITION, MF_POPUP, MF_STRING,
};
use windows::Win32::System::Com::StructuredStorage::ReleaseStgMedium;

pub(crate) static INSTANCE_COUNT: AtomicU32 = AtomicU32::new(0);

#[implement(IShellExtInit, IContextMenu)]
pub struct QuickSortShellExt {
    item_paths: RefCell<Vec<PathBuf>>,
    folders: Mutex<Vec<Folder>>,
}

impl QuickSortShellExt {
    pub const CLS_ID: GUID = GUID::from_u128(0x12345678_1234_1234_1234_1234567890AB);
}

static LOG_INIT: OnceLock<()> = OnceLock::new();

impl Default for QuickSortShellExt {
    fn default() -> Self {
        INSTANCE_COUNT.fetch_add(1, Ordering::SeqCst);

        // Инициализируем логгер только один раз
        LOG_INIT.get_or_init(|| {
            let log_dir = match std::env::var("APPDATA") {
                Ok(appdata) => {
                    let mut p = std::path::PathBuf::from(appdata);
                    p.push("QuickSort");
                    let _ = std::fs::create_dir_all(&p);
                    p.push("quicksort_dll.log");
                    p
                }
                Err(_) => {
                    // Запасной вариант — рядом с DLL
                    let mut p = std::env::current_exe().unwrap_or_default();
                    p.set_file_name("quicksort_dll.log");
                    p
                }
            };

            match std::fs::File::create(&log_dir) {
                Ok(file) => {
                    let config = simplelog::ConfigBuilder::new()
                        .add_filter_allow_str("quicksort")
                        .build();
                    let _ = simplelog::WriteLogger::init(
                        simplelog::LevelFilter::Debug,
                        config,
                        file,
                    );
                    log::info!("DLL logging started. Log path: {}", log_dir.display());
                }
                Err(e) => {
                    let fallback = std::env::current_exe()
                        .unwrap_or_default()
                        .with_file_name("quicksort_dll_fallback.log");
                    if let Ok(f) = std::fs::File::create(&fallback) {
                        let _ = simplelog::WriteLogger::init(
                            simplelog::LevelFilter::Debug,
                            simplelog::Config::default(),
                            f,
                        );
                        log::error!(
                            "Failed to create main log file: {}. Writing to fallback.",
                            e
                        );
                    }
                }
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
        let paths = if let Some(data_obj) = &*data_obj {
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

// ------------------- Вспомогательные функции (из Gist) -------------------
unsafe fn dropfiles_to_paths(files: &DROPFILES) -> Vec<PathBuf> {
    let mut res = Vec::new();
    let is_wide = files.fWide.as_bool();
    let mut str_ptr = unsafe {
        ptr::from_ref(files)
            .cast::<u8>()
            .add((&raw const files.pFiles).read_unaligned() as _)
    };
    while is_wide && unsafe { str_ptr.cast::<u16>().read() != 0 } || unsafe { str_ptr.read() } != 0 {
        let (bytes_shift, path) = if is_wide {
            let s = PCWSTR(str_ptr.cast::<u16>());
            (
                2 * (unsafe { s.len() } + 1),
                PathBuf::from(OsString::from_wide(unsafe { s.as_wide() })),
            )
        } else {
            let s = unsafe { CStr::from_ptr(str_ptr.cast()) };
            (
                s.count_bytes() + 1,
                PathBuf::from(s.to_string_lossy().into_owned()),
            )
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
        tymed: TYMED_HGLOBAL.0.cast_unsigned(),
        ptd: ptr::null_mut(),
    };
    let storage = unsafe { data_obj.GetData(&raw const fmt) }?;
    let _ = ManuallyDrop::into_inner(storage.pUnkForRelease);
    if storage.tymed != TYMED_HGLOBAL.0.cast_unsigned() {
        return Err(E_FAIL.into());
    }
    let global = unsafe { Owned::new(storage.u.hGlobal) };
    if global.is_invalid() {
        return Err(E_POINTER.into());
    }
    let lock = unsafe { GlobalLock(*global) };
    if lock.is_null() {
        return Err(E_POINTER.into());
    }
    let files = unsafe { &*lock.cast_const().cast::<DROPFILES>() };
    let files_list = unsafe { dropfiles_to_paths(files) };
    unsafe { GlobalUnlock(*global) }.ok();
    Ok(files_list)
}

fn itemidlist_to_path(item_list: *const ITEMIDLIST) -> WinResult<PathBuf> {
    let shell_folder = unsafe { SHGetDesktopFolder() }?;
    let mut name = STRRET {
        uType: STRRET_WSTR.0.cast_unsigned(),
        ..Default::default()
    };
    unsafe {
        shell_folder.GetDisplayNameOf(
            item_list,
            SHGDNF(SHGDN_NORMAL.0 | SHGDN_FORPARSING.0),
            &raw mut name,
        )
    }?;
    let mut path = MaybeUninit::uninit();
    unsafe { StrRetToStrW(&raw mut name, None, path.as_mut_ptr()) }?;
    let path = unsafe { path.assume_init() };
    Ok(OsString::from_wide(unsafe { path.as_wide() }).into())
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
        if flags & CMF_DEFAULTONLY != 0 {
            return S_OK;
        }

        let repo = match JsonRepository::new() {
            Ok(r) => r,
            Err(e) => {
                log::error!("JsonRepository::new failed: {:?}", e);
                return E_FAIL;
            }
        };
        let config = match repo.load() {
            Ok(c) => c,
            Err(e) => {
                log::error!("load config failed: {:?}", e);
                return E_FAIL;
            }
        };
        let mut folders = config.folders.clone();
        *self.this.folders.lock() = folders.clone();

        unsafe {
            let h_submenu = CreatePopupMenu().unwrap();
            let mut current_id = min_cmd_id;
            let max_id = max_cmd_id;

            for folder in folders.iter().filter(|f| f.favorite) {
                if current_id > max_id { break; }
                let wide_name: Vec<u16> = OsString::from(&folder.name).encode_wide().chain(Some(0)).collect();
                let mut item = MENUITEMINFOW {
                    cbSize: mem::size_of::<MENUITEMINFOW>() as u32,
                    fMask: MIIM_ID | MIIM_STATE | MIIM_STRING,
                    wID: current_id,
                    fState: MFS_ENABLED,
                    dwTypeData: PWSTR::from_raw(wide_name.as_ptr() as *mut _),
                    cch: (wide_name.len() - 1) as u32,
                    ..Default::default()
                };
                InsertMenuItemW(h_submenu, 0xFFFFFFFF, true, &raw const item);
                current_id += 1;
            }

            if current_id <= max_id {
                let other_text = windows::core::w!("📂 Другие папки...");
                let mut item = MENUITEMINFOW {
                    cbSize: mem::size_of::<MENUITEMINFOW>() as u32,
                    fMask: MIIM_ID | MIIM_STATE | MIIM_STRING,
                    wID: current_id,
                    fState: MFS_ENABLED,
                    dwTypeData: PWSTR::from_raw(other_text.as_ptr() as *mut _),
                    cch: other_text.len() as u32,
                    ..Default::default()
                };
                InsertMenuItemW(h_submenu, 0xFFFFFFFF, true, &raw const item);
                current_id += 1;
            }

            let root_text = windows::core::w!("QuickSort");
            InsertMenuW(menu, menu_index, MF_BYPOSITION | MF_POPUP, h_submenu.0 as usize, root_text.as_ptr());
        }

        MAKE_HRESULT(SEVERITY_SUCCESS.cast_signed(), 0, (max_cmd_id - min_cmd_id + 1) as i32)
    }

    fn InvokeCommand(&self, info: *const CMINVOKECOMMANDINFO) -> WinResult<()> {
        if info.is_null() {
            return E_POINTER.ok();
        }
        let ici = unsafe { *info };
        let verb = (ici.lpVerb.0 as u32) & 0xFFFF;

        let folders = self.this.folders.lock();
        let favorites: Vec<&Folder> = folders.iter().filter(|f| f.favorite).collect();
        let total_fav = favorites.len() as u32;

        if verb < total_fav {
            let target = &favorites[verb as usize].path;
            for path in self.this.item_paths.borrow().iter() {
                if let Err(e) = MoveEngine::move_file(path, target) {
                    log::error!("Move failed: {:?}", e);
                }
            }
        } else if verb == total_fav {
            for path in self.this.item_paths.borrow().iter() {
                let exe_path = std::env::current_exe()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| "quicksort.exe".to_string());
                let params: Vec<u16> = format!("select-folder --file \"{}\"", path.display())
                    .encode_utf16().chain(Some(0)).collect();
                unsafe {
                    windows::Win32::UI::Shell::ShellExecuteW(
                        None,
                        windows::core::w!("open"),
                        &exe_path,
                        windows::core::PCWSTR(params.as_ptr()),
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
        }
            .ok()
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
        let iface_id = unsafe { *iface_id };
        let object = unsafe { &mut *obj_out };
        *object = ptr::null_mut();

        match iface_id {
            IUnknown::IID => {
                *object = IUnknown::from(QuickSortShellExt::default()).into_raw();
            }
            IShellExtInit::IID => {
                *object = IShellExtInit::from(QuickSortShellExt::default()).into_raw();
            }
            IContextMenu::IID => {
                *object = IContextMenu::from(QuickSortShellExt::default()).into_raw();
            }
            _ => return Err(E_NOINTERFACE.into()),
        }
        Ok(())
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

#[expect(non_snake_case)]
fn MAKE_HRESULT(sev: i32, fac: i32, code: i32) -> HRESULT {
    HRESULT((sev << 31) | (fac << 16) | code)
}