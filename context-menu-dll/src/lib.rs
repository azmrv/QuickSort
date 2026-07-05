mod shellext;

use std::ffi::c_void;
use windows::core::{GUID, HRESULT, IUnknown, Interface};
use windows::Win32::Foundation::{CLASS_E_CLASSNOTAVAILABLE, S_FALSE, S_OK, E_POINTER};
use windows::Win32::System::Com::IClassFactory;
use shellext::{QuickSortShellExt, QuickSortClassFactory};

const CLSID_QUICKSORT: GUID = GUID::from_u128(0x12345678_1234_1234_1234_1234567890AB);

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
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