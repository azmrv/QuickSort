//! Windows Shell Extension DLL entry points.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod icon;
mod lifecycle;
mod pipe_client;
mod shellext;

use std::ptr;
use std::sync::atomic::Ordering;
use windows::core::{IUnknown, Interface, GUID, HRESULT};
use windows::Win32::Foundation::{
    CLASS_E_CLASSNOTAVAILABLE, E_INVALIDARG, E_POINTER, S_FALSE, S_OK,
};

use shellext::{QuickSortClassFactory, CLSID_QUICKSORT, INSTANCE_COUNT};

/// DllGetClassObject - returns a class factory for the requested CLSID.
#[no_mangle]
pub extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    unsafe {
        *ppv = ptr::null_mut();
    }

    if rclsid.is_null() || riid.is_null() {
        return E_INVALIDARG;
    }

    if unsafe { *rclsid } != CLSID_QUICKSORT {
        log::error!(
            "DllGetClassObject: unknown CLSID {:?}, expected {:?}",
            unsafe { *rclsid },
            CLSID_QUICKSORT
        );
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    // Only hand out a class factory while the owning QuickSort app is running
    // for the current user. When the owner is dead (app exited, another user's
    // session, or a stale registration) the extension becomes inert so it never
    // interferes with this user's Explorer / file dialogs (redirects, "New
    // Folder", context menus). This is the safety boundary that keeps the
    // in-process shell extension from touching another user's shell.
    if !crate::lifecycle::is_owner_alive() {
        log::info!("DllGetClassObject: owner not running, refusing class object");
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    log::info!("DllGetClassObject: creating factory");
    let factory = QuickSortClassFactory;
    let unknown: IUnknown = factory.into();

    unsafe { unknown.query(riid, ppv) }
}

/// DllCanUnloadNow - returns S_OK when the DLL may be unloaded by COM:
/// no live instances, or the owning QuickSort app is no longer running.
#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    let count = INSTANCE_COUNT.load(Ordering::SeqCst);
    if crate::lifecycle::can_unload(count) {
        S_OK
    } else {
        S_FALSE
    }
}

/// DllRegisterServer - registers the COM server.
/// For now, we rely on activation.reg.
#[no_mangle]
pub extern "system" fn DllRegisterServer() -> HRESULT {
    S_OK
}

/// DllUnregisterServer - removes the COM server.
/// For now, we rely on the .reg file.
#[no_mangle]
pub extern "system" fn DllUnregisterServer() -> HRESULT {
    S_OK
}
