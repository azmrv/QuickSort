mod shellext;
mod pipe_client;

use windows::core::{GUID, HRESULT, Interface};
use windows::Win32::Foundation::{CLASS_E_CLASSNOTAVAILABLE, S_FALSE, E_POINTER, S_OK};
use windows::Win32::System::Com::IClassFactory;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK};
use windows::core::w;
use std::ffi::c_void;
use shellext::QuickSortClassFactory;
use shellext::INSTANCE_COUNT;
pub use shellext::CLSID_QUICKSORT;



#[no_mangle]
pub unsafe extern "system" fn DllMain(
    _hinst: windows::Win32::Foundation::HINSTANCE,
    reason: u32,
    _reserved: *const c_void,
) -> bool {
    if reason == 1 { // DLL_PROCESS_ATTACH
        MessageBoxW(None, w!("DLL_PROCESS_ATTACH"), w!("QuickSort DLL"), MB_OK);
    }
    true
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    MessageBoxW(None, w!("DllGetClassObject called!"), w!("QuickSort DLL"), MB_OK);
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        return E_POINTER;
    }
    if *rclsid == CLSID_QUICKSORT {
        let factory: IClassFactory = QuickSortClassFactory::default().into();
        factory.query(riid, ppv)
    } else {
        CLASS_E_CLASSNOTAVAILABLE
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}