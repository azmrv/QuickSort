//! Utilities to manipulate resource icons and bitmaps.

use std::collections::HashMap;
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::sync::{LazyLock, OnceLock};

use parking_lot::RwLock;
use windows::core::{w, Owned, Result as WinResult, PCWSTR};
use windows::Win32::Foundation::{E_FAIL, HINSTANCE};
use windows::Win32::Graphics::Gdi::HBITMAP;
use windows::Win32::Graphics::GdiPlus::{
    Color as GpColor, GdipCreateBitmapFromHICON, GdipCreateHBITMAPFromBitmap, GdiplusShutdown,
    GdiplusStartup, GdiplusStartupInput, GpBitmap, Status as GpStatus,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, LoadImageW, HICON, IMAGE_ICON, LR_DEFAULTCOLOR, SM_CXSMICON, SM_CYSMICON,
};

/// Global cache of icon [`PCWSTR`] names to [`HBITMAP`]s in raw values.
static ICON_TO_BITMAP_CACHE: LazyLock<RwLock<HashMap<usize, usize>>> =
    LazyLock::new(Default::default);

/// Cached DLL module handle (set once on first use).
static DLL_HINSTANCE: OnceLock<SyncHINSTANCE> = OnceLock::new();

/// Wrapper around HINSTANCE to make it Send+Sync (it's a process-wide constant).
#[derive(Clone, Copy)]
struct SyncHINSTANCE(HINSTANCE);
unsafe impl Send for SyncHINSTANCE {}
unsafe impl Sync for SyncHINSTANCE {}

/// Returns the HINSTANCE of the current DLL module.
///
/// Uses `GetModuleHandleW` with the DLL file name to obtain the handle.
/// The handle is cached for subsequent calls.
pub fn get_dll_instance() -> HINSTANCE {
    DLL_HINSTANCE
        .get_or_init(|| {
            let name = w!("context_menu_dll.dll");
            let h = unsafe { GetModuleHandleW(name) }
                .map(|h| HINSTANCE(h.0))
                .unwrap_or(HINSTANCE(ptr::null_mut()));
            SyncHINSTANCE(h)
        })
        .0
}

/// Loads the QuickSort app icon from DLL resources and returns an HBITMAP.
///
/// Returns `None` if the icon cannot be loaded (e.g., resource not found).
pub fn load_app_icon_bitmap() -> Option<HBITMAP> {
    // Try loading from DLL resources first
    let dll = get_dll_instance();
    if !dll.0.is_null() {
        let icon_name = w!("QUICKSORT_ICON");
        match resource_icon_to_bitmap(dll, icon_name) {
            Ok(bmp) => return Some(bmp),
            Err(e) => {
                log::warn!(
                    "Failed to load icon from DLL resource: {:?}, trying file",
                    e
                );
            }
        }
    }

    // Fallback: load from quicksort.ico next to the DLL
    load_icon_from_dll_dir("quicksort.ico")
}

/// Loads an icon from a file next to the DLL and converts it to HBITMAP.
fn load_icon_from_dll_dir(icon_filename: &str) -> Option<HBITMAP> {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::UI::WindowsAndMessaging::{LR_LOADFROMFILE, SM_CXICON};

    // Get DLL directory
    let dll = get_dll_instance();
    let mut dll_path_buf = [0u16; 512];
    let len = unsafe {
        windows::Win32::System::LibraryLoader::GetModuleFileNameW(
            Some(HMODULE(dll.0)),
            &mut dll_path_buf,
        )
    };
    if len == 0 {
        log::warn!("GetModuleFileNameW failed");
        return None;
    }
    let dll_path = OsString::from_wide(&dll_path_buf[..len as usize]);
    let dll_dir = std::path::Path::new(&dll_path).parent()?;
    let icon_path = dll_dir.join(icon_filename);

    log::info!("Loading icon from: {}", icon_path.display());

    let icon_path_wide: Vec<u16> = icon_path.as_os_str().encode_wide().chain(Some(0)).collect();

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

    icon_to_bitmap(HICON(hicon.0)).ok()
}

/// Loads the given `icon_name` from `dll` and converts it to a bitmap.
pub fn resource_icon_to_bitmap(dll: HINSTANCE, icon_name: PCWSTR) -> WinResult<HBITMAP> {
    if let Some(&bmp_addr) = ICON_TO_BITMAP_CACHE
        .read()
        .get(&icon_name.as_ptr().expose_provenance())
    {
        Ok(HBITMAP(ptr::with_exposed_provenance_mut(bmp_addr)))
    } else {
        let icon = load_small_icon(dll, icon_name).inspect_err(|err| {
            log::error!(
                "Failed to load icon {:?} from DLL {:?}: {:?}",
                icon_name,
                dll,
                err,
            );
        })?;
        let icon_bmp = icon_to_bitmap(*icon).map_err(|err| {
            log::error!("Failed to convert the icon to a bitmap: {:?}", err);
            E_FAIL
        })?;

        ICON_TO_BITMAP_CACHE.write().insert(
            icon_name.as_ptr().expose_provenance(),
            icon_bmp.0.expose_provenance(),
        );
        Ok(icon_bmp)
    }
}

/// Loads the given `icon_name` from `dll` in small size.
///
/// `icon_name` can be an actual string pointer or a special value constructed
/// from the `MAKEINTRESOURCE` macro. The handle is returned in an owned
/// fashion for immediate [`Drop`] compatibility.
fn load_small_icon(dll: HINSTANCE, icon_name: PCWSTR) -> WinResult<Owned<HICON>> {
    // `LoadIconWithScaleDown` is basically not available to us due to:
    // https://developercommunity.visualstudio.com/t/LoadIconWithScaleDown-not-in-the-default/10646099?sort=newest&topics=Known+Issue+in%3A+Visual+Studio+2017+Version+15.5
    // SAFETY: always safe to call supposing the arguments are valid.
    unsafe {
        LoadImageW(
            Some(dll),
            icon_name,
            IMAGE_ICON,
            GetSystemMetrics(SM_CXSMICON),
            GetSystemMetrics(SM_CYSMICON),
            LR_DEFAULTCOLOR,
        )
    }
    // SAFETY: the handle has just been created, so is owned by us.
    .map(|h| unsafe { Owned::new(HICON(h.0)) })
}

/// Converts the given icon to a bitmap.
///
/// Currently implemented using the GDI+ library because the regular GDI did
/// not yield good results: the icon was very badly rendered. The handle is
/// returned directly in a raw fashion so its ownership may be passed onto the
/// system without releasing the resources by mistake.
fn icon_to_bitmap(icon: HICON) -> GpResult<HBITMAP> {
    let gp_token =
        GpToken::new().inspect_err(|err| log::error!("Failed to initialize GDI+: {:?}", err))?;

    let mut gp_bmp: MaybeUninit<*mut GpBitmap> = MaybeUninit::uninit();
    // SAFETY: both pointers are valid, respectively for reading and for writing.
    gp_status_ok(unsafe { GdipCreateBitmapFromHICON(icon, gp_bmp.as_mut_ptr()) })
        .inspect_err(|err| log::error!("GdipCreateBitmapFromHICON failed: {:?}", err))?;
    // SAFETY: errors are checked for, so the pointer is valid at this point.
    let gp_bmp = unsafe { gp_bmp.assume_init() };
    // `GpBitmap` does not have a destructor.

    let mut bmp: MaybeUninit<HBITMAP> = MaybeUninit::uninit();
    // SAFETY:
    //  * the GDI+ bitmap pointer comes from the API;
    //  * the GDI bitmap pointer is valid for writing;
    gp_status_ok(unsafe {
        GdipCreateHBITMAPFromBitmap(
            gp_bmp,
            bmp.as_mut_ptr(),
            GpColor::Transparent.cast_unsigned(),
        )
    })
    .inspect_err(|err| log::error!("GdipCreateHBITMAPFromBitmap failed: {:?}", err))?;
    // SAFETY: errors are checked for, so the pointer is valid at this point.
    let bmp = unsafe { bmp.assume_init() };
    // This value is returned, so does not need releasing.

    // Explicit drop to show the role of the token guard.
    mem::drop(gp_token);
    Ok(bmp)
}

/// RAII wrapper around a GDI+ session token with an adequate [`Drop`].
#[repr(transparent)]
struct GpToken(usize);

impl GpToken {
    /// Initializes a GDI+ session by calling [`GdiplusStartup`] with default values.
    pub fn new() -> GpResult<Self> {
        let input = GdiplusStartupInput {
            // > Must be 1.
            GdiplusVersion: 1,
            // Not useful here.
            // > The default value is NULL.
            DebugEventCallback: 0,
            // For easier API interaction:
            // > If you don't want to be responsible for calling the hook and
            // > unhook functions, then set this member to `FALSE`.
            SuppressBackgroundThread: false.into(),
            // For extra safety, although:
            // > GDI+ version 1.0 doesn't support external image codecs, so
            // > this field is ignored.
            SuppressExternalCodecs: true.into(),
        };
        let mut token: MaybeUninit<usize> = MaybeUninit::uninit();
        // SAFETY:
        //  * the token pointer is valid for writing;
        //  * the input pointer is valid for reading;
        //  * the output pointer can be null because `SuppressBackgroundThread`
        //    is set to `FALSE`, as per the documentation;
        gp_status_ok(unsafe {
            GdiplusStartup(token.as_mut_ptr(), &raw const input, ptr::null_mut())
        })
        .inspect_err(|err| log::error!("GdiplusStartup failed: {:?}", err))?;
        // SAFETY: errors are checked for, so the pointer is valid at this point.
        Ok(Self(unsafe { token.assume_init() }))
    }
}

/// Calls [`GdiplusShutdown`] with the stored token value.
impl Drop for GpToken {
    fn drop(&mut self) {
        // SAFETY: the passed value comes from the API.
        unsafe { GdiplusShutdown(self.0) };
    }
}

/// Shortcut to [`Result`] with a GDI+ error type set.
type GpResult<T> = Result<T, GpStatus>;

/// Maps GDI+ statuses to [`GpResult`]s.
#[inline]
fn gp_status_ok(status: GpStatus) -> GpResult<()> {
    if status.0 == 0 {
        Ok(())
    } else {
        Err(status)
    }
}
