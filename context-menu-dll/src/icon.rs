//! Icon loading utilities for the context menu.
//!
//! Loads the QuickSort icon from the .ico file next to the DLL and converts
//! it to an HBITMAP suitable for MIIM_BITMAP in context menu items.
//!
//! Uses plain Win32 GDI (no GDI+) to avoid deadlocks in Explorer's process.

#![allow(dead_code)]

use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::sync::OnceLock;

use windows::core::{Result as WinResult, PCWSTR};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, ReleaseDC, SelectObject, GetDC,
    HBITMAP, HGDIOBJ,
};
use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};
use windows::Win32::UI::WindowsAndMessaging::{
    DrawIconEx, GetSystemMetrics, LoadImageW, HICON, IMAGE_ICON, DI_NORMAL, LR_DEFAULTCOLOR,
    LR_LOADFROMFILE, SM_CXICON, SM_CXSMICON, SM_CYICON,
};

/// Cached DLL module handle as raw pointer (set once on first use).
static DLL_HMODULE: OnceLock<usize> = OnceLock::new();

/// Returns the HMODULE of the current DLL module as a raw pointer.
fn get_dll_hmodule() -> usize {
    *DLL_HMODULE.get_or_init(|| {
        let name = windows::core::w!("context_menu_dll.dll");
        unsafe { GetModuleHandleW(name) }
            .map(|h| h.0 as usize)
            .unwrap_or(0)
    })
}

/// Loads the QuickSort app icon and returns an HBITMAP for MIIM_BITMAP.
///
/// Tries the embedded DLL resource first, then falls back to quicksort.ico
/// next to the DLL on disk. Returns `None` if neither source is available.
pub fn load_app_icon_bitmap() -> Option<HBITMAP> {
    let module = get_dll_hmodule();

    // Try loading from DLL resource (QUICKSORT_ICON)
    if module != 0 {
        let icon_name = windows::core::w!("QUICKSORT_ICON");
        let hmodule = HMODULE(module as *mut _);
        if let Ok(hicon) = load_icon_from_resource(hmodule, icon_name) {
            if let Some(bmp) = icon_to_bitmap(&hicon) {
                log::info!("Icon loaded from DLL resource");
                return Some(bmp);
            }
        }
    }

    // Fallback: load from quicksort.ico next to the DLL
    let hicon = load_icon_from_file("quicksort.ico")?;
    let bmp = icon_to_bitmap(&hicon)?;
    log::info!("Icon loaded from file");
    Some(bmp)
}

/// Loads an icon from a .ico file next to the DLL.
fn load_icon_from_file(filename: &str) -> Option<HICON> {
    let module = get_dll_hmodule();
    let mut dll_path_buf = [0u16; 512];
    let hmodule = HMODULE(module as *mut _);
    let len = unsafe { GetModuleFileNameW(Some(hmodule), &mut dll_path_buf) };
    if len == 0 {
        log::warn!("GetModuleFileNameW failed for icon path");
        return None;
    }
    let dll_path = OsString::from_wide(&dll_path_buf[..len as usize]);
    let dll_dir = std::path::Path::new(&dll_path).parent()?;
    let icon_path = dll_dir.join(filename);

    log::info!("Loading icon from: {}", icon_path.display());

    let icon_path_wide: Vec<u16> = icon_path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let icon_size = unsafe { GetSystemMetrics(SM_CXICON) };

    let hicon = unsafe {
        LoadImageW(
            None,
            PCWSTR::from_raw(icon_path_wide.as_ptr()),
            IMAGE_ICON,
            icon_size,
            icon_size,
            LR_DEFAULTCOLOR | LR_LOADFROMFILE,
        )
    }
    .ok()?;

    Some(HICON(hicon.0))
}

/// Loads an icon from a DLL resource by name.
fn load_icon_from_resource(dll: HMODULE, icon_name: PCWSTR) -> WinResult<HICON> {
    let icon_size = unsafe { GetSystemMetrics(SM_CXSMICON) };
    let hinstance = windows::Win32::Foundation::HINSTANCE(dll.0);
    let hicon = unsafe {
        LoadImageW(
            Some(hinstance),
            icon_name,
            IMAGE_ICON,
            icon_size,
            icon_size,
            LR_DEFAULTCOLOR,
        )
    }?;
    Ok(HICON(hicon.0))
}

/// Converts an HICON to an HBITMAP using plain GDI (no GDI+).
fn icon_to_bitmap(icon: &HICON) -> Option<HBITMAP> {
    unsafe {
        let screen_dc = GetDC(None);
        if screen_dc.is_invalid() {
            log::warn!("GetDC(None) failed");
            return None;
        }

        let icon_w = GetSystemMetrics(SM_CXICON);
        let icon_h = GetSystemMetrics(SM_CYICON);

        let bitmap = CreateCompatibleBitmap(screen_dc, icon_w, icon_h);
        if bitmap.is_invalid() {
            ReleaseDC(None, screen_dc);
            log::warn!("CreateCompatibleBitmap failed");
            return None;
        }

        let mem_dc = CreateCompatibleDC(Some(screen_dc));
        if mem_dc.is_invalid() {
            let _ = windows::Win32::Graphics::Gdi::DeleteObject(HGDIOBJ(bitmap.0));
            ReleaseDC(None, screen_dc);
            log::warn!("CreateCompatibleDC failed");
            return None;
        }

        let old_obj = SelectObject(mem_dc, HGDIOBJ(bitmap.0));

        let _ = DrawIconEx(mem_dc, 0, 0, *icon, icon_w, icon_h, 0, None, DI_NORMAL);

        SelectObject(mem_dc, old_obj);
        let _ = DeleteDC(mem_dc);
        ReleaseDC(None, screen_dc);

        Some(bitmap)
    }
}
