//! [`IContextMenu`]-based Shell extension core.
//!
//! Exports [`ShellExtension`] that implements the expected COM interfaces.
//! [`execute_command`] can be seen as the entrypoint run on every path.

use std::cell::RefCell;
use std::ffi::{CStr, OsString, c_void};
use std::mem::{ManuallyDrop, MaybeUninit};
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::{fmt, mem, ptr};

use windows::Win32::Foundation::{
    CLASS_E_NOAGGREGATION, E_FAIL, E_NOINTERFACE, E_NOTIMPL, E_POINTER, HINSTANCE, S_OK,
    SEVERITY_SUCCESS,
};
use windows::Win32::System::Com::{
    DVASPECT_CONTENT, FORMATETC, IClassFactory, IClassFactory_Impl, IDataObject, TYMED_HGLOBAL,
};
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
    HMENU, InsertMenuItemW, MENUITEMINFOW, MFS_ENABLED, MIIM_BITMAP, MIIM_ID, MIIM_STATE,
    MIIM_STRING,
};
use windows::core::{
    BOOL, GUID, HRESULT, IUnknown, Interface, Owned, PCWSTR, PSTR, PWSTR, Ref as WinRef,
    Result as WinResult, implement,
};

use crate::{DLL_HMODULE, icon};

pub(crate) static INSTANCE_COUNT: AtomicU32 = AtomicU32::new(0);

#[implement(IShellExtInit, IContextMenu)]
pub struct ShellExtension {
    /// List of paths set in [`<ShellExtension_Impl as IShellExtInit_Impl>::Initialize`].
    item_paths: RefCell<Vec<PathBuf>>,
}

impl ShellExtension {
    pub const CLS_ID: GUID = GUID::from_u128(0x12345678_9abc_def1_2345_6789abcdef42);
}

impl Default for ShellExtension {
    fn default() -> Self {
        log::trace!("@ShellExtension::default");
        INSTANCE_COUNT.fetch_add(1, Ordering::SeqCst);
        Self {
            item_paths: Default::default(),
        }
    }
}

impl Drop for ShellExtension {
    fn drop(&mut self) {
        log::trace!("@ShellExtension::drop");
        INSTANCE_COUNT.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Required for [`IContextMenu`].
impl IShellExtInit_Impl for ShellExtension_Impl {
    fn Initialize(
        &self,
        folder_idl: *const ITEMIDLIST,
        data_obj: WinRef<'_, IDataObject>,
        prog_id: HKEY,
    ) -> WinResult<()> {
        log::debug!(
            "@IShellExtInit_Impl::Initialize => folder_idl: {:?}, data_obj: {:?}, prog_id: {:?}",
            folder_idl,
            &*data_obj,
            prog_id,
        );
        // > For shortcut menu extensions, `pdtobj` identifies the selected
        // > file objects, `hkeyProgID` identifies the file type of the object
        // > with focus, and `pidlFolder` is either `NULL` (for file objects)
        // > or specifies the folder for which the shortcut menu is being
        // > requested (for folder background shortcut menus).
        let paths = if let Some(data_obj) = &*data_obj {
            log::debug!("Receiving file paths from IShellExtInit.");
            let fmt = FORMATETC {
                cfFormat: CF_HDROP.0,
                dwAspect: DVASPECT_CONTENT.0,
                // > The most common value is -1, which identifies all of the data.
                lindex: -1,
                tymed: TYMED_HGLOBAL.0.cast_unsigned(),
                // > A NULL value is used whenever the specified data
                // > format is independent of the target device or when the
                // > caller doesn't care what device is used.
                ptd: ptr::null_mut(),
            };
            // SAFETY: the format value is valid.
            let storage = unsafe { data_obj.GetData(&raw const fmt) }
                .inspect_err(|err| log::error!("IDataObject::GetData failed: {:?}", err))?;
            let _storage_rel = ManuallyDrop::into_inner(storage.pUnkForRelease);

            if storage.tymed != TYMED_HGLOBAL.0.cast_unsigned() {
                log::error!("Received tymed is not HGLOBAL: {:?}", storage.tymed);
                return E_FAIL.ok();
            }

            // SAFETY:
            //  * the variant is checked just above, so the union value is correct;
            //  * the global is returned to us, so we have ownership of it;
            let global = unsafe { Owned::new(storage.u.hGlobal) };

            if global.is_invalid() {
                log::error!("Received global is null.");
                return E_POINTER.ok();
            }

            // SAFETY: the passed pointer cannot be null at this point.
            let lock = unsafe { GlobalLock(*global) };

            if lock.is_null() {
                log::error!("Received global lock pointer is null.");
                return E_POINTER.ok();
            };

            // SAFETY:
            //  * the pointer cannot be null at this point;
            //  * `CF_HDROP` is requested, so the cast is valid;
            let files = unsafe { &*lock.cast_const().cast::<DROPFILES>() };
            // SAFETY: the value is received from the API, so must be valid.
            let files_list = unsafe { dropfiles_to_paths(files) };

            // SAFETY:
            //  * the pointer is still valid;
            //  * a lock is still held at this point;
            unsafe { GlobalUnlock(*global) }
                .inspect_err(|err| log::error!("GlobalUnlock failed: {:?}", err))?;

            files_list
        } else {
            log::debug!("Receiving a folder path from IShellExtInit.");

            if folder_idl.is_null() {
                log::error!("Folder path is null as well.");
                return E_POINTER.ok();
            }

            vec![itemidlist_to_path(folder_idl).inspect_err(|err| {
                log::error!("Failed to convert the ITEMIDLIST to a path: {:?}", err);
            })?]
        };

        log::debug!("Files list received from IShellExtInit: {:#?}", paths);
        self.this.item_paths.replace(paths);
        Ok(())
    }
}

/// Extracts paths from the given files container.
///
/// For use with [`IContextMenu`].
/// See <https://learn.microsoft.com/en-us/windows/win32/shell/clipboard#cf_hdrop>.
///
/// # Safety
///
/// The passed value must have a valid files offset pointing to a string list.
unsafe fn dropfiles_to_paths(files: &DROPFILES) -> Vec<PathBuf> {
    let mut res = Vec::new();
    let is_wide = files.fWide.as_bool();
    log::trace!("DROPFILES is wide encoded: {}.", is_wide);
    // SAFETY: `pFiles` is the offset to the string data.
    let mut str_ptr = unsafe {
        ptr::from_ref(files)
            .cast::<u8>()
            .add((&raw const files.pFiles).read_unaligned() as _)
    };

    #[expect(
        clippy::cast_ptr_alignment,
        reason = "Only for wide; safety upheld by caller."
    )]
    // SAFETY:
    //  * the validity of the offset is upheld by the caller;
    //  * the read is performed with the right type depending on `is_wide`;
    while is_wide && unsafe { str_ptr.cast::<u16>().read() != 0 } || unsafe { str_ptr.read() } != 0
    {
        log::trace!("str_ptr: {:?}", str_ptr);
        let (bytes_shift, path) = if is_wide {
            let s = PCWSTR(str_ptr.cast::<u16>());
            // SAFETY: the pointer is valid at this point.
            (
                2 * (unsafe { s.len() } + 1),
                PathBuf::from(OsString::from_wide(unsafe { s.as_wide() })),
            )
        } else {
            // SAFETY: the pointer is valid at this point.
            let s = unsafe { CStr::from_ptr(str_ptr.cast()) };
            (
                s.count_bytes() + 1,
                PathBuf::from(s.to_string_lossy().into_owned()),
            )
        };
        log::trace!("path: {}", path.display());
        res.push(path);
        log::trace!("bytes_shift: {}", bytes_shift);
        // SAFETY: the strings are concatenated.
        str_ptr = unsafe { str_ptr.add(bytes_shift) };
    }

    res
}

/// Extracts paths from the given item ID list.
///
/// For use with [`IContextMenu`].
/// See <https://learn.microsoft.com/en-us/windows/win32/shell/namespace-intro#item-id-lists>.
fn itemidlist_to_path(item_list: *const ITEMIDLIST) -> WinResult<PathBuf> {
    // SAFETY: always safe to call.
    let shell_folder = unsafe { SHGetDesktopFolder() }
        .inspect_err(|err| log::error!("SHGetDesktopFolder failed: {:?}", err))?;
    let mut name = STRRET {
        uType: STRRET_WSTR.0.cast_unsigned(),
        ..Default::default()
    };
    // SAFETY: the name pointer is valid.
    unsafe {
        shell_folder.GetDisplayNameOf(
            item_list,
            SHGDNF(SHGDN_NORMAL.0 | SHGDN_FORPARSING.0),
            &raw mut name,
        )
    }
    .inspect_err(|err| log::error!("IShellFolder::GetDisplayNameOf failed: {:?}", err))?;

    let mut path = MaybeUninit::uninit();
    // SAFETY: both pointers are valid.
    unsafe { StrRetToStrW(&raw mut name, None, path.as_mut_ptr()) }
        .inspect_err(|err| log::error!("StrRetToStrW failed: {:?}", err))?;
    // SAFETY: errors are checked for, so the value is correct at this point.
    let path = unsafe { path.assume_init() };

    // SAFETY: same.
    Ok(OsString::from_wide(unsafe { path.as_wide() }).into())
}

/// API for WinXP+ Shell integration (effectively deprecated starting from Win11).
impl IContextMenu_Impl for ShellExtension_Impl {
    fn GetCommandString(
        &self,
        cmd_id: usize,
        flags: u32,
        _reserved: *const u32,
        name_out: PSTR,
        name_out_len: u32,
    ) -> WinResult<()> {
        log::debug!(
            "@IContextMenu_Impl::GetCommandString => cmd_id: {}, flags: {:x?}, name_out: {:?}, name_out_len: {}",
            cmd_id,
            flags,
            name_out,
            name_out_len,
        );

        unsafe fn write_out<T: Default + Copy + fmt::Debug>(val: &[T], out: *mut T, max_len: u32) {
            let len = val.len().min(max_len as usize - 1);
            // SAFETY: upheld by caller.
            unsafe {
                out.copy_from_nonoverlapping(val.as_ptr(), len);
                out.add(len).write(T::default());
            }
        }

        match flags {
            // The menu is always enabled.
            GCS_VALIDATEA | GCS_VALIDATEW => S_OK,
            GCS_VERBA | GCS_VERBW | GCS_HELPTEXTA | GCS_HELPTEXTW if name_out.is_null() => {
                E_POINTER
            }
            // The verb is a "language-independent" value identifying the command.
            // SAFETY: the out pointer is not null at this point.
            GCS_VERBA => unsafe {
                write_out(
                    "SampleShellextVerb".as_bytes(),
                    name_out.as_ptr(),
                    name_out_len,
                );
                S_OK
            },
            // SAFETY: same.
            GCS_VERBW => unsafe {
                let s = windows::core::w!("SampleShellextVerb");
                write_out(s.as_wide(), name_out.as_ptr().cast(), name_out_len);
                S_OK
            },
            // SAFETY: same.
            GCS_HELPTEXTA => unsafe {
                write_out(
                    "Sample Shell extension verb".as_bytes(),
                    name_out.as_ptr(),
                    name_out_len,
                );
                S_OK
            },
            // SAFETY: same.
            GCS_HELPTEXTW => unsafe {
                let s = windows::core::w!("Sample Shell extension verb");
                write_out(s.as_wide(), name_out.as_ptr().cast(), name_out_len);
                S_OK
            },
            _ => {
                log::error!("Unknown requested command flags: {:x?}", flags);
                E_NOTIMPL
            }
        }
        .ok()
    }

    fn QueryContextMenu(
        &self,
        menu: HMENU,
        menu_index: u32,
        min_cmd_id: u32,
        max_cmd_id: u32,
        flags: u32,
    ) -> HRESULT {
        log::debug!(
            "@IContextMenu_Impl::QueryContextMenu => menu: {:?}, menu_index: {}, min_cmd_id: {}, max_cmd_id: {}, flags: {:x?}",
            menu,
            menu_index,
            min_cmd_id,
            max_cmd_id,
            flags,
        );

        // > This flag provides a hint for the shortcut menu extension to add
        // > nothing if it does not modify the default item in the menu.
        if flags & CMF_DEFAULTONLY != 0 {
            return S_OK;
        }

        if menu.is_invalid() {
            return E_POINTER;
        }

        let item_text = windows::core::w!("Sample Shell extension verb");
        let dll_icon = match icon::resource_icon_to_bitmap(
            HINSTANCE(ptr::with_exposed_provenance_mut(
                DLL_HMODULE.load(Ordering::SeqCst),
            )),
            // As configured in the `resource.rc`.
            windows::core::w!("IDI_MYAPP_ICON"),
        ) {
            Ok(i) => i,
            // This method returns `HRESULT` instead of `Result` in order to be
            // able to embed non-error information into it, so do this manually.
            Err(err) => return err.into(),
        };
        let menu_item = MENUITEMINFOW {
            cbSize: mem::size_of::<MENUITEMINFOW>() as _,
            fMask: MIIM_ID | MIIM_STATE | MIIM_STRING | MIIM_BITMAP,
            wID: min_cmd_id,
            fState: MFS_ENABLED,
            dwTypeData: PWSTR::from_raw(item_text.as_ptr().cast_mut()),
            // SAFETY: the value is just above, so valid.
            cch: unsafe { item_text.len() } as _,
            hbmpItem: dll_icon,
            // We're only ever setting a single entry, so no need for the rest.
            ..Default::default()
        };

        // SAFETY:
        //  * the validity of the menu handle is checked above;
        //  * the menu item is built here and referenced, so valid;
        if let Err(err) = unsafe { InsertMenuItemW(menu, menu_index, true, &raw const menu_item) } {
            log::error!("InsertMenuItemW failed: {:?}", err);
            return err.into();
        }

        // > If successful, returns an `HRESULT` value that has its severity
        // > value set to `SEVERITY_SUCCESS` and its code value set to the
        // > offset of the largest command identifier that was assigned, plus one.
        // Here, there is only one, of ID `min_id`, so `min_id - min_id + 1 == 1`.
        MAKE_HRESULT(SEVERITY_SUCCESS.cast_signed(), 0, 1)
    }

    fn InvokeCommand(&self, info: *const CMINVOKECOMMANDINFO) -> WinResult<()> {
        log::debug!("@IContextMenu_Impl::InvokeCommand => info: {:?}", info);

        if info.is_null() {
            log::error!("Info pointer is null.");
            return E_POINTER.ok();
        }

        // As set by `IShellExtInit::Initialize`.
        execute_command(self.this.item_paths.borrow().as_slice());

        Ok(())
    }
}

/// Executes the extension's command on the given paths.
fn execute_command(paths: &[impl AsRef<Path>]) {
    log::info!(
        "Executing command for the following paths:\n{:#?}",
        paths.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
    );
}

#[implement(IClassFactory)]
#[derive(Default)]
pub struct ShellextClassFactory;

impl IClassFactory_Impl for ShellextClassFactory_Impl {
    fn CreateInstance(
        &self,
        outer: WinRef<'_, IUnknown>,
        iface_id: *const GUID,
        obj_out: *mut *mut c_void,
    ) -> WinResult<()> {
        log::debug!(
            "@ShellextClassFactory_Impl::CreateInstance => outer: {:?}, iface_id: {:?}, obj_out: {:?}",
            &*outer,
            iface_id,
            obj_out,
        );

        if outer.is_some() {
            log::error!("Outer parameter is non-null.");
            // > The `pUnkOuter` parameter was non-`NULL` and the object does
            // > not support aggregation.
            return Err(CLASS_E_NOAGGREGATION.into());
        }

        let iface_id = if iface_id.is_null() {
            log::error!("Interface ID pointer is null.");
            return Err(E_POINTER.into());
        } else {
            // SAFETY: the pointer was just checked.
            unsafe { *iface_id }
        };
        log::debug!("Queried interface ID: {:?}", iface_id);
        let object = if obj_out.is_null() {
            log::error!("Object return pointer is null.");
            return Err(E_POINTER.into());
        } else {
            // SAFETY: the pointer was just checked.
            unsafe { &mut *obj_out }
        };
        // > If an error occurs, the interface pointer is `NULL`.
        *object = ptr::null_mut();

        match iface_id {
            IUnknown::IID => {
                log::debug!("Requested interface is IUnknown.");
                *object = IUnknown::from(ShellExtension::default()).into_raw();
            }
            IShellExtInit::IID => {
                log::debug!("Requested interface is IShellExtInit.");
                *object = IShellExtInit::from(ShellExtension::default()).into_raw();
            }
            IContextMenu::IID => {
                log::debug!("Requested interface is IContextMenu.");
                *object = IContextMenu::from(ShellExtension::default()).into_raw();
            }
            // > The object that `ppvObject` points to does not support the
            // > interface identified by `riid`.
            _ => {
                log::error!("Unsupported interface.");
                return Err(E_NOINTERFACE.into());
            }
        }

        Ok(())
    }

    fn LockServer(&self, lock: BOOL) -> WinResult<()> {
        log::debug!("@ShellextClassFactory_Impl::LockServer => lock: {:?}", lock);

        if lock.as_bool() {
            INSTANCE_COUNT.fetch_add(1, Ordering::SeqCst);
        } else {
            INSTANCE_COUNT.fetch_sub(1, Ordering::SeqCst);
        }

        Ok(())
    }
}

/// Taken from `winapi`.
#[expect(non_snake_case)]
#[inline]
pub fn MAKE_HRESULT(sev: i32, fac: i32, code: i32) -> HRESULT {
    HRESULT((sev << 31) | (fac << 16) | code)
}
