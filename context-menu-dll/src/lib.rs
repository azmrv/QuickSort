//! Windows Shell Extension DLL entry points.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod icon;
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
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    let factory = QuickSortClassFactory;
    let unknown: IUnknown = factory.into();

    unsafe { unknown.query(riid, ppv) }
}

/// DllCanUnloadNow - returns S_OK if no instances exist.
#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    let count = INSTANCE_COUNT.load(Ordering::SeqCst);
    if count == 0 {
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
