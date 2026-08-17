//! gdi32.dll – Builtin-Implementierung von joys-win (PHASE 10).
//!
//! Abbildung auf eine in-process Grafik-Infrastruktur:
//! - Device Contexts (HDC) mit Puffer (RGBA)
//! - Bitmaps (HBITMAP) als Pixel-Puffer; SelectObject bindet eine Bitmap an
//!   einen DC
//! - SetPixelV/GetPixel arbeiten real auf den Pixeln (keine Dummies)
//!
//! Es gibt (noch) KEINE Fenster-/Bildschirmausgabe; die Pixel liegen im
//! Speicher und sind über GetPixel lesbar (für Tests und spätere
//! Bildschirm-Ausgabe).

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::runtime::ExeError;

#[derive(Debug)]
enum GdiObject {
    Dc {
        bound_bitmap: Option<usize>,
        width: usize,
        height: usize,
        own_pixels: Vec<u8>, // RGBA, falls keine Bitmap gebunden
    },
    Bitmap {
        width: usize,
        height: usize,
        pixels: Vec<u8>, // RGBA
    },
}

static OBJECTS: std::sync::LazyLock<std::sync::Mutex<HashMap<usize, GdiObject>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
static NEXT_HANDLE: AtomicUsize = AtomicUsize::new(0x2000);

fn alloc(obj: GdiObject) -> usize {
    let h = NEXT_HANDLE.fetch_add(1, Ordering::SeqCst);
    if let Ok(mut m) = OBJECTS.lock() {
        m.insert(h, obj);
    }
    h
}

fn gdi_remove(h: usize) -> bool {
    OBJECTS
        .lock()
        .map(|mut m| m.remove(&h).is_some())
        .unwrap_or(false)
}

/// Schreibt einen Pixel (COLORREF 0x00BBGGRR) in den Puffer des DC.
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_set_pixel_v_impl(hdc: usize, x: i32, y: i32, color: u32) -> u32 {
    // Phase 1: Ziel-Objekt und Dimensionen ermitteln.
    let (target, w, h) = {
        let objects = match OBJECTS.lock() {
            Ok(o) => o,
            Err(_) => return 0,
        };
        let Some(GdiObject::Dc {
            bound_bitmap,
            width,
            height,
            ..
        }) = objects.get(&hdc)
        else {
            return 0;
        };
        if let Some(bmp) = bound_bitmap {
            let Some(GdiObject::Bitmap {
                width: bw,
                height: bh,
                ..
            }) = objects.get(bmp)
            else {
                return 0;
            };
            (*bmp, *bw, *bh)
        } else {
            (hdc, *width, *height)
        }
    };
    if x < 0 || y < 0 || (x as usize) >= w || (y as usize) >= h {
        return 0;
    }
    // Phase 2: Pixel schreiben.
    let idx = (y as usize * w + x as usize) * 4;
    let mut objects = match OBJECTS.lock() {
        Ok(o) => o,
        Err(_) => return 0,
    };
    let Some(obj) = objects.get_mut(&target) else {
        return 0;
    };
    let buf = match obj {
        GdiObject::Dc { own_pixels, .. } => own_pixels,
        GdiObject::Bitmap { pixels, .. } => pixels,
    };
    if idx + 4 <= buf.len() {
        buf[idx..idx + 4].copy_from_slice(&[
            (color & 0xFF) as u8,
            ((color >> 8) & 0xFF) as u8,
            ((color >> 16) & 0xFF) as u8,
            0xFF,
        ]);
        color
    } else {
        0
    }
}

/// Liest einen Pixel (COLORREF) aus dem Puffer des DC.
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_get_pixel_impl(hdc: usize, x: i32, y: i32) -> u32 {
    let objects = match OBJECTS.lock() {
        Ok(o) => o,
        Err(_) => return 0,
    };
    let Some(GdiObject::Dc {
        bound_bitmap,
        width,
        height,
        own_pixels,
    }) = objects.get(&hdc)
    else {
        return 0;
    };
    let (buf, w, h) = if let Some(bmp) = bound_bitmap {
        let Some(GdiObject::Bitmap {
            width,
            height,
            pixels,
        }) = objects.get(bmp)
        else {
            return 0;
        };
        (pixels, *width, *height)
    } else {
        (own_pixels, *width, *height)
    };
    if x < 0 || y < 0 || (x as usize) >= w || (y as usize) >= h {
        return 0;
    }
    let idx = (y as usize * w + x as usize) * 4;
    if idx + 4 <= buf.len() {
        let r = buf[idx] as u32;
        let g = buf[idx + 1] as u32;
        let b = buf[idx + 2] as u32;
        (b << 16) | (g << 8) | r
    } else {
        0
    }
}

/// GetDC(HWND) -> HDC (Bildschirm-DC, 1x1-Puffer)
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_get_dc_impl(_hwnd: i64) -> i64 {
    alloc(GdiObject::Dc {
        bound_bitmap: None,
        width: 1,
        height: 1,
        own_pixels: vec![0; 4],
    }) as i64
}

/// ReleaseDC(HWND, HDC) -> int
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_release_dc_impl(_hwnd: i64, hdc: i64) -> i32 {
    gdi_remove(hdc as usize) as i32
}

/// CreateCompatibleDC(HDC) -> HDC
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_create_compatible_dc_impl(_hdc: i64) -> i64 {
    alloc(GdiObject::Dc {
        bound_bitmap: None,
        width: 1,
        height: 1,
        own_pixels: vec![0; 4],
    }) as i64
}

/// CreateCompatibleBitmap(HDC, int, int) -> HBITMAP
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_create_compatible_bitmap_impl(_hdc: i64, w: i32, h: i32) -> i64 {
    if w <= 0 || h <= 0 {
        return 0;
    }
    alloc(GdiObject::Bitmap {
        width: w as usize,
        height: h as usize,
        pixels: vec![0; (w as usize) * (h as usize) * 4],
    }) as i64
}

/// SelectObject(HDC, HGDIOBJ) -> HGDIOBJ (vorheriges Objekt)
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_select_object_impl(hdc: usize, obj: usize) -> i64 {
    let mut objects = match OBJECTS.lock() {
        Ok(o) => o,
        Err(_) => return 0,
    };
    // Nur Bitmaps sind als GDI-Objekte wählbar.
    if !matches!(objects.get(&obj), Some(GdiObject::Bitmap { .. })) {
        return 0;
    }
    let Some(GdiObject::Dc { bound_bitmap, .. }) = objects.get_mut(&hdc) else {
        return 0;
    };
    let prev = bound_bitmap.take();
    *bound_bitmap = Some(obj);
    prev.unwrap_or(0) as i64
}

/// DeleteObject(HGDIOBJ) -> BOOL
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_delete_object_impl(obj: usize) -> i32 {
    gdi_remove(obj) as i32
}

/// DeleteDC(HDC) -> BOOL
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_delete_dc_impl(hdc: usize) -> i32 {
    gdi_remove(hdc) as i32
}

// ---------------------------------------------------------------------------
// Import-Auflösung
// ---------------------------------------------------------------------------

/// Löst einen gdi32-Import auf die passende Stub-Adresse auf.
///
/// Nicht implementierte Funktionen liefern `Err(UnimplementedApi)`.
pub fn resolve(imp: &crate::loader::imports::Import) -> Result<usize, ExeError> {
    let name = match imp {
        crate::loader::imports::Import::ByName { name, .. } => name.as_str(),
        crate::loader::imports::Import::ByOrdinal { ordinal } => {
            return Err(ExeError::UnimplementedApi(
                "gdi32.dll".into(),
                format!("#{ordinal}"),
            ))
        }
    };
    let stub = match name {
        "GetDC" => fn_addr(joys_win_get_dc_stub),
        "ReleaseDC" => fn_addr(joys_win_release_dc_stub),
        "CreateCompatibleDC" => fn_addr(joys_win_create_compatible_dc_stub),
        "CreateCompatibleBitmap" => fn_addr(joys_win_create_compatible_bitmap_stub),
        "SelectObject" => fn_addr(joys_win_select_object_stub),
        "DeleteObject" => fn_addr(joys_win_delete_object_stub),
        "DeleteDC" => fn_addr(joys_win_delete_dc_stub),
        "SetPixelV" => fn_addr(joys_win_set_pixel_v_stub),
        "GetPixel" => fn_addr(joys_win_get_pixel_stub),
        other => return Err(ExeError::UnimplementedApi("gdi32.dll".into(), other.into())),
    };
    Ok(stub)
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn fn_addr(f: unsafe extern "C" fn()) -> usize {
    f as usize
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
extern "C" {
    fn joys_win_get_dc_stub();
    fn joys_win_release_dc_stub();
    fn joys_win_create_compatible_dc_stub();
    fn joys_win_create_compatible_bitmap_stub();
    fn joys_win_select_object_stub();
    fn joys_win_delete_object_stub();
    fn joys_win_delete_dc_stub();
    fn joys_win_set_pixel_v_stub();
    fn joys_win_get_pixel_stub();
}

#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
fn fn_addr(_f: usize) -> usize {
    0
}

#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
macro_rules! stub_const {
    ($($name:ident),*) => {
        $(
            #[allow(non_upper_case_globals)]
            const $name: usize = 0;
        )*
    };
}

#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
stub_const!(
    joys_win_get_dc_stub,
    joys_win_release_dc_stub,
    joys_win_create_compatible_dc_stub,
    joys_win_create_compatible_bitmap_stub,
    joys_win_select_object_stub,
    joys_win_delete_object_stub,
    joys_win_delete_dc_stub,
    joys_win_set_pixel_v_stub,
    joys_win_get_pixel_stub
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmap_pixel_roundtrip() {
        let hdc = joys_win_create_compatible_dc_impl(0);
        assert_ne!(hdc, 0);
        let bmp = joys_win_create_compatible_bitmap_impl(0, 8, 8);
        assert_ne!(bmp, 0);
        joys_win_select_object_impl(hdc as usize, bmp as usize);
        // Rot setzen und zurücklesen.
        let c = joys_win_set_pixel_v_impl(hdc as usize, 1, 1, 0x00FF0000);
        assert_eq!(c, 0x00FF0000);
        let read = joys_win_get_pixel_impl(hdc as usize, 1, 1);
        assert_eq!(read, 0x00FF0000, "Pixel-Roundtrip muss die Farbe liefern");
        // Außerhalb der Bitmap -> 0.
        assert_eq!(joys_win_get_pixel_impl(hdc as usize, 100, 100), 0);
        joys_win_delete_object_impl(bmp as usize);
        joys_win_delete_dc_impl(hdc as usize);
    }
}
