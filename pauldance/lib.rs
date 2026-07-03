#![cfg(windows)]

use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

use windows::Win32::Foundation::{
    CLASS_E_CLASSNOTAVAILABLE, E_NOINTERFACE, E_POINTER, HINSTANCE, S_FALSE, S_OK,
};
use windows::Win32::System::Com::IClassFactory;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::core::{BOOL, GUID, HRESULT, Interface};

mod icon;
mod shellext;
use shellext::{INSTANCE_COUNT, ShellExtension, ShellextClassFactory};

static DLL_HMODULE: AtomicUsize = AtomicUsize::new(0);

#[unsafe(no_mangle)]
extern "system" fn DllMain(dll: HINSTANCE, reason: u32, _reserved: *const c_void) -> BOOL {
    log::debug!("@DllMain => dll: {:?}, reason: {}", dll, reason);

    if reason == DLL_PROCESS_ATTACH {
        DLL_HMODULE.store(dll.0.expose_provenance(), Ordering::SeqCst);
    }

    true.into()
}

#[unsafe(no_mangle)]
extern "system" fn DllCanUnloadNow() -> HRESULT {
    log::debug!("@DllCanUnloadNow");

    if INSTANCE_COUNT.load(Ordering::SeqCst) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}

#[unsafe(no_mangle)]
extern "system" fn DllGetClassObject(
    cls_id: *const GUID,
    iface_id: *const GUID,
    obj_out: *mut *mut c_void,
) -> HRESULT {
    log::debug!(
        "@DllGetClassObject => cls_id: {:?}, iface_id: {:?}, obj_out: {:?}",
        cls_id,
        iface_id,
        obj_out,
    );
    let obj_out = if obj_out.is_null() {
        return E_POINTER;
    } else {
        // SAFETY: the pointer was just checked.
        unsafe { &mut *obj_out }
    };
    // > If an error occurs, the interface pointer is `NULL`.
    *obj_out = ptr::null_mut();

    let cls_id = if cls_id.is_null() {
        return E_POINTER;
    } else {
        // SAFETY: the pointer was just checked.
        unsafe { *cls_id }
    };
    // Ensure `cls_id` matches the expected `CLSID` value.
    if cls_id != ShellExtension::CLS_ID {
        // > The DLL does not support the class (object definition).
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    let iface_id = if iface_id.is_null() {
        return E_POINTER;
    } else {
        // SAFETY: the pointer was just checked.
        unsafe { *iface_id }
    };
    // Ensure `iface_id` matches the expected `IID_IClassFactory` value.
    if iface_id != IClassFactory::IID {
        return E_NOINTERFACE;
    }

    // Finally, create and return the desired factory.
    *obj_out = IClassFactory::from(ShellextClassFactory {}).into_raw();
    S_OK
}
