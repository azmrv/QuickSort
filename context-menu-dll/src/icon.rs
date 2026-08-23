//! Icon and bitmap utilities for the context menu.
//!
//! - `load_app_icon_bitmap()` — loads the QuickSort icon for the root menu entry
//! - `create_colored_circle_bitmap()` — creates a small colored circle for submenu items
//!
//! Uses plain Win32 GDI (no GDI+) to avoid deadlocks in Explorer's process.
//! Bitmaps are created with `CreateBitmap` for exact pixel dimensions.

#![allow(dead_code)]

use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::sync::OnceLock;

use windows::core::{Result as WinResult, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HMODULE};
use windows::Win32::Graphics::Gdi::{
    CreateBitmap, CreateCompatibleDC, CreateSolidBrush, DeleteDC, DeleteObject, Ellipse,
    GetDC, ReleaseDC, SelectObject, HBITMAP, HGDIOBJ,
};
use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};
use windows::Win32::UI::WindowsAndMessaging::{
    DrawIconEx, LoadImageW, HICON, IMAGE_ICON, DI_NORMAL, LR_DEFAULTCOLOR,
    LR_LOADFROMFILE,
};

/// Size of context menu icons in pixels.
const ICON_SIZE: i32 = 16;

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

/// Creates an exact-size 32bpp bitmap and draws into it via a memory DC.
///
/// Returns (bitmap, memory_dc, old_object) — caller must clean up.
fn create_exact_bitmap(size: i32) -> Option<(HBITMAP, windows::Win32::Graphics::Gdi::HDC, HGDIOBJ)> {
    unsafe {
        let screen_dc = GetDC(None);
        if screen_dc.is_invalid() {
            log::warn!("GetDC(None) failed");
            return None;
        }

        // CreateBitmap: exact pixel dimensions, 32bpp, 1 plane — no DPI scaling
        let bitmap = CreateBitmap(size, size, 1, 32, None);
        if bitmap.is_invalid() {
            ReleaseDC(None, screen_dc);
            log::warn!("CreateBitmap failed");
            return None;
        }

        let mem_dc = CreateCompatibleDC(Some(screen_dc));
        ReleaseDC(None, screen_dc);
        if mem_dc.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            log::warn!("CreateCompatibleDC failed");
            return None;
        }

        let old_obj = SelectObject(mem_dc, HGDIOBJ(bitmap.0));
        Some((bitmap, mem_dc, old_obj))
    }
}

/// Loads the QuickSort app icon as a 16x16 HBITMAP.
///
/// Used for MIIM_BITMAP on the root "QuickSort" context menu entry.
pub fn load_app_icon_bitmap() -> Option<HBITMAP> {
    let module = get_dll_hmodule();

    // Try loading from DLL resource (QUICKSORT_ICON)
    if module != 0 {
        let icon_name = windows::core::w!("QUICKSORT_ICON");
        let hmodule = HMODULE(module as *mut _);
        if let Ok(hicon) = load_icon_from_resource(hmodule, icon_name) {
            if let Some(bmp) = icon_to_bitmap(&hicon) {
                log::info!("Icon loaded from DLL resource ({}x{})", ICON_SIZE, ICON_SIZE);
                return Some(bmp);
            }
        }
    }

    // Fallback: load from quicksort.ico next to the DLL
    let hicon = load_icon_from_file("quicksort.ico")?;
    let bmp = icon_to_bitmap(&hicon)?;
    log::info!("Icon loaded from file ({}x{})", ICON_SIZE, ICON_SIZE);
    Some(bmp)
}

/// Creates a small colored circle bitmap for submenu folder items.
///
/// `color` is a Windows COLORREF value: `0x00BBGGRR`.
pub fn create_colored_circle_bitmap(color: u32) -> Option<HBITMAP> {
    let (bitmap, mem_dc, old_obj) = create_exact_bitmap(ICON_SIZE)?;

    unsafe {
        let padding = 2;

        // Draw filled circle
        let brush = CreateSolidBrush(COLORREF(color));
        let old_brush = SelectObject(mem_dc, HGDIOBJ(brush.0));
        let _ = Ellipse(
            mem_dc,
            padding,
            padding,
            ICON_SIZE - padding,
            ICON_SIZE - padding,
        );
        SelectObject(mem_dc, old_brush);
        let _ = DeleteObject(HGDIOBJ(brush.0));

        SelectObject(mem_dc, old_obj);
        let _ = DeleteDC(mem_dc);
    }

    Some(bitmap)
}

/// Parses a hex color string like "#FF5733" or "FF5733" to COLORREF (0x00BBGGRR).
pub fn parse_color_to_colorref(color: &str) -> Option<u32> {
    let hex = color.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    // COLORREF is 0x00BBGGRR
    Some((b as u32) << 16 | (g as u32) << 8 | (r as u32))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

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

    let hicon = unsafe {
        LoadImageW(
            None,
            PCWSTR::from_raw(icon_path_wide.as_ptr()),
            IMAGE_ICON,
            ICON_SIZE,
            ICON_SIZE,
            LR_DEFAULTCOLOR | LR_LOADFROMFILE,
        )
    }
    .ok()?;

    Some(HICON(hicon.0))
}

/// Loads an icon from a DLL resource by name.
fn load_icon_from_resource(dll: HMODULE, icon_name: PCWSTR) -> WinResult<HICON> {
    let hinstance = windows::Win32::Foundation::HINSTANCE(dll.0);
    let hicon = unsafe {
        LoadImageW(
            Some(hinstance),
            icon_name,
            IMAGE_ICON,
            ICON_SIZE,
            ICON_SIZE,
            LR_DEFAULTCOLOR,
        )
    }?;
    Ok(HICON(hicon.0))
}

/// Converts an HICON to an exact-size HBITMAP using CreateBitmap.
fn icon_to_bitmap(icon: &HICON) -> Option<HBITMAP> {
    let (bitmap, mem_dc, old_obj) = create_exact_bitmap(ICON_SIZE)?;

    unsafe {
        let _ = DrawIconEx(mem_dc, 0, 0, *icon, ICON_SIZE, ICON_SIZE, 0, None, DI_NORMAL);
        SelectObject(mem_dc, old_obj);
        let _ = DeleteDC(mem_dc);
    }

    Some(bitmap)
}
