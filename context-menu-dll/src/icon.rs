//! Utilities to manipulate resource icons and bitmaps.

use std::collections::HashMap;
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::sync::LazyLock;

use parking_lot::RwLock;
use windows::Win32::Foundation::{E_FAIL, HINSTANCE};
use windows::Win32::Graphics::Gdi::HBITMAP;
use windows::Win32::Graphics::GdiPlus::{
    Color as GpColor, GdipCreateBitmapFromHICON, GdipCreateHBITMAPFromBitmap, GdiplusShutdown,
    GdiplusStartup, GdiplusStartupInput, GpBitmap, Status as GpStatus,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, HICON, IMAGE_ICON, LR_DEFAULTCOLOR, LoadImageW, SM_CXSMICON, SM_CYSMICON,
};
use windows::core::{Owned, PCWSTR, Result as WinResult};

/// Global cache of icon [`PCWSTR`] names to [`HBITMAP`]s in raw values.
static ICON_TO_BITMAP_CACHE: LazyLock<RwLock<HashMap<usize, usize>>> =
    LazyLock::new(Default::default);

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
    if status.0 == 0 { Ok(()) } else { Err(status) }
}
