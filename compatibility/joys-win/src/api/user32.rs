//! user32.dll – Builtin-Implementierung von joys-win (PHASE 9).
//!
//! Abbildung auf eine interne, in-process Fenster-/Nachrichten-Infrastruktur:
//! - Fensterklassen (RegisterClassExA) mit WndProc
//! - Fenster (CreateWindowExA) als Handle-Objekte
//! - Nachrichten-Queue pro Thread mit GetMessage/PostMessage
//! - DispatchMessage ruft die WndProc (Win64-Konvention) auf
//!
//! Es gibt (noch) KEINE Pixel-/Display-Ausgabe – das Fenstersystem ist real,
//! aber fensterlos. Texturen/Grafik folgen mit GDI32/User32-Zeichnen.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::runtime::ExeError;

// --- Windows-Konstanten ---
pub const WM_CREATE: u32 = 0x0001;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_QUIT: u32 = 0x0012;
pub const WM_APP: u32 = 0x8000;

// --- Interne Strukturen ---
struct WndClass {
    wndproc: usize,
}

struct Window {
    class: String,
    #[allow(dead_code)] // Stil-Info des Fensters (Modell), Auswertung folgt
    style: u32,
    visible: bool,
}

struct Msg {
    hwnd: usize,
    message: u32,
    wparam: u64,
    lparam: i64,
}

static CLASSES: std::sync::LazyLock<std::sync::Mutex<HashMap<String, WndClass>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
static WINDOWS: std::sync::LazyLock<std::sync::Mutex<HashMap<usize, Window>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
static NEXT_HWND: AtomicUsize = AtomicUsize::new(0x1000);

static QUEUE: std::sync::LazyLock<std::sync::Mutex<VecDeque<Msg>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(VecDeque::new()));
static QUEUE_COND: std::sync::Condvar = std::sync::Condvar::new();

/// Liest einen C-String ab einer Adresse (bis 255 Bytes).
unsafe fn read_cstr(p: *const u8) -> Option<String> {
    crate::api::filesystem::cstr_a(p)
}

// ---------------------------------------------------------------------------
// Impls (Win64-ABI, von den Stubs in runtime/abi.rs aufgerufen)
// ---------------------------------------------------------------------------

/// RegisterClassExA(const WNDCLASSEXA*) -> ATOM
///
/// # Safety
/// `lp_wnd_class` muss auf eine gültige WNDCLASSEXA-Struktur zeigen.
#[no_mangle]
pub unsafe extern "C" fn joys_win_register_class_ex_a_impl(lp_wnd_class: *const u8) -> u16 {
    if lp_wnd_class.is_null() {
        return 0;
    }
    // WNDCLASSEXA (x64, 80 Bytes): lpfnWndProc@8, lpszClassName@64.
    let wndproc = *(lp_wnd_class.add(8) as *const usize);
    let class_name_ptr = *(lp_wnd_class.add(64) as *const *const u8);
    let Some(class_name) = read_cstr(class_name_ptr) else {
        return 0;
    };
    if let Ok(mut classes) = CLASSES.lock() {
        classes.insert(class_name.clone(), WndClass { wndproc });
    }
    // Einfache ATOM = Index in der Klasse.
    1
}

/// CreateWindowExA(...) -> HWND
///
/// # Safety
/// `lp_class_name`/`lp_window_name` müssen gültige Strings sein.
#[allow(clippy::too_many_arguments)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_create_window_ex_a_impl(
    _dw_ex_style: u32,
    lp_class_name: *const u8,
    _lp_window_name: *const u8,
    dw_style: u32,
    _x: i32,
    _y: i32,
    _n_width: i32,
    _n_height: i32,
    _h_wnd_parent: i64,
    _h_menu: i64,
    _h_instance: i64,
    _lp_param: *mut core::ffi::c_void,
) -> i64 {
    let Some(class_name) = read_cstr(lp_class_name) else {
        return 0;
    };
    let wndproc = CLASSES
        .lock()
        .ok()
        .and_then(|c| c.get(&class_name).map(|w| w.wndproc))
        .unwrap_or(0);
    if wndproc == 0 {
        return 0; // Klasse nicht registriert
    }
    let hwnd = NEXT_HWND.fetch_add(1, Ordering::SeqCst);
    if let Ok(mut w) = WINDOWS.lock() {
        w.insert(
            hwnd,
            Window {
                class: class_name,
                style: dw_style,
                visible: false,
            },
        );
    }
    // WM_CREATE an die WndProc.
    let r = call_wndproc(hwnd, WM_CREATE, 0, 0, wndproc);
    if r == -1 {
        if let Ok(mut w) = WINDOWS.lock() {
            w.remove(&hwnd);
        }
        return 0;
    }
    hwnd as i64
}

/// ShowWindow(HWND, int) -> BOOL
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_show_window_impl(hwnd: usize, _n_cmd_show: i32) -> i32 {
    let mut was_visible = false;
    if let Ok(mut w) = WINDOWS.lock() {
        if let Some(win) = w.get_mut(&hwnd) {
            was_visible = win.visible;
            win.visible = true;
        }
    }
    i32::from(was_visible)
}

/// UpdateWindow(HWND) -> BOOL
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_update_window_impl(_hwnd: usize) -> i32 {
    1
}

/// DestroyWindow(HWND) -> BOOL
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_destroy_window_impl(hwnd: usize) -> i32 {
    let (wndproc, exists) = WINDOWS
        .lock()
        .ok()
        .map(|w| {
            w.get(&hwnd)
                .and_then(|win| {
                    CLASSES
                        .lock()
                        .ok()
                        .map(|c| c.get(&win.class).map(|k| k.wndproc))
                })
                .flatten()
                .map(|p| (p, true))
                .unwrap_or((0, true))
        })
        .unwrap_or((0, false));
    if !exists {
        return 0;
    }
    if wndproc != 0 {
        unsafe { call_wndproc(hwnd, WM_DESTROY, 0, 0, wndproc) };
    }
    if let Ok(mut w) = WINDOWS.lock() {
        w.remove(&hwnd);
    }
    1
}

/// GetMessageA(LPMSG, HWND, UINT, UINT) -> BOOL (0 bei WM_QUIT)
///
/// # Safety
/// `lp_msg` muss auf ein gültiges MSG (48 Bytes) zeigen.
#[no_mangle]
pub unsafe extern "C" fn joys_win_get_message_a_impl(
    lp_msg: *mut u8,
    _h_wnd: usize,
    _w_msg_filter_min: u32,
    _w_msg_filter_max: u32,
) -> i32 {
    let mut guard = QUEUE.lock().expect("queue lock");
    loop {
        if let Some(m) = guard.pop_front() {
            let is_quit = m.message == WM_QUIT;
            write_msg(lp_msg, &m);
            if is_quit {
                return 0;
            }
            return 1;
        }
        guard = QUEUE_COND.wait(guard).expect("cond wait");
    }
}

/// TranslateMessage(const MSG*) -> BOOL
///
/// # Safety
/// `lp_msg` muss auf ein gültiges MSG zeigen.
#[no_mangle]
pub unsafe extern "C" fn joys_win_translate_message_impl(_lp_msg: *const u8) -> i32 {
    0 // keine Tastatur-Übersetzung (noch)
}

/// DispatchMessageA(const MSG*) -> LRESULT
///
/// # Safety
/// `lp_msg` muss auf ein gültiges MSG zeigen.
#[no_mangle]
pub unsafe extern "C" fn joys_win_dispatch_message_a_impl(lp_msg: *const u8) -> i64 {
    let m = read_msg(lp_msg);
    let wndproc = WINDOWS
        .lock()
        .ok()
        .and_then(|w| {
            w.get(&m.hwnd).and_then(|win| {
                CLASSES
                    .lock()
                    .ok()
                    .map(|c| c.get(&win.class).map(|k| k.wndproc))
            })
        })
        .flatten()
        .unwrap_or(0);
    if wndproc == 0 {
        return 0;
    }
    call_wndproc(m.hwnd, m.message, m.wparam, m.lparam, wndproc)
}

/// PostMessageA(HWND, UINT, WPARAM, LPARAM) -> BOOL
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_post_message_a_impl(
    hwnd: usize,
    message: u32,
    wparam: u64,
    lparam: i64,
) -> i32 {
    push_msg(Msg {
        hwnd,
        message,
        wparam,
        lparam,
    });
    1
}

/// PostQuitMessage(int) -> void
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_post_quit_message_impl(exit_code: i32) {
    push_msg(Msg {
        hwnd: 0,
        message: WM_QUIT,
        wparam: exit_code as u32 as u64,
        lparam: 0,
    });
}

/// DefWindowProcA(HWND, UINT, WPARAM, LPARAM) -> LRESULT
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_def_window_proc_a_impl(
    hwnd: usize,
    msg: u32,
    _wparam: u64,
    _lparam: i64,
) -> i64 {
    if msg == WM_CLOSE {
        return i64::from(joys_win_destroy_window_impl(hwnd));
    }
    0
}

// ---------------------------------------------------------------------------
// Helfer
// ---------------------------------------------------------------------------

fn push_msg(m: Msg) {
    if let Ok(mut q) = QUEUE.lock() {
        q.push_back(m);
        QUEUE_COND.notify_all();
    }
}

/// Schreibt MSG (x64, 48 Bytes) an `dst`.
///
/// # Safety
/// `dst` muss auf 48 gültige Bytes zeigen.
unsafe fn write_msg(dst: *mut u8, m: &Msg) {
    *(dst as *mut u64) = m.hwnd as u64;
    *(dst.add(8) as *mut u32) = m.message;
    *(dst.add(16) as *mut u64) = m.wparam;
    *(dst.add(24) as *mut i64) = m.lparam;
    *(dst.add(32) as *mut u32) = 0; // time
    *(dst.add(36) as *mut i32) = 0; // pt.x
    *(dst.add(40) as *mut i32) = 0; // pt.y
}

/// Liest MSG (x64, 48 Bytes) von `src`.
///
/// # Safety
/// `src` muss auf 48 gültige Bytes zeigen.
unsafe fn read_msg(src: *const u8) -> Msg {
    Msg {
        hwnd: *(src as *const u64) as usize,
        message: *(src.add(8) as *const u32),
        wparam: *(src.add(16) as *const u64),
        lparam: *(src.add(24) as *const i64),
    }
}

/// Ruft die Windows-WndProc über das Win64-Trampoline auf.
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
unsafe fn call_wndproc(hwnd: usize, msg: u32, wparam: u64, lparam: i64, wndproc: usize) -> i64 {
    unsafe extern "C" {
        fn joys_win_call_wndproc(h: usize, m: u32, w: u64, l: i64, p: usize) -> i64;
    }
    // SAFETY: wndproc zeigt auf Code im gemappten Windows-Image (Win64-ABI).
    unsafe { joys_win_call_wndproc(hwnd, msg, wparam, lparam, wndproc) }
}

/// Fallback für Nicht-Linux-Ziele (dort ohnehin unsupported).
///
/// # Safety
/// Nur für die Kompilierung auf Nicht-Linux-Zielen.
#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
unsafe fn call_wndproc(
    _hwnd: usize,
    _msg: u32,
    _wparam: u64,
    _lparam: i64,
    _wndproc: usize,
) -> i64 {
    0
}

// ---------------------------------------------------------------------------
// Import-Auflösung
// ---------------------------------------------------------------------------

/// Löst einen user32-Import auf die passende Stub-Adresse auf.
///
/// Nicht implementierte Funktionen liefern `Err(UnimplementedApi)`.
pub fn resolve(imp: &crate::loader::imports::Import) -> Result<usize, ExeError> {
    let name = match imp {
        crate::loader::imports::Import::ByName { name, .. } => name.as_str(),
        crate::loader::imports::Import::ByOrdinal { ordinal } => {
            return Err(ExeError::UnimplementedApi(
                "user32.dll".into(),
                format!("#{ordinal}"),
            ))
        }
    };
    let stub = match name {
        "RegisterClassExA" => fn_addr(joys_win_register_class_ex_a_stub),
        "CreateWindowExA" => fn_addr(joys_win_create_window_ex_a_stub),
        "ShowWindow" => fn_addr(joys_win_show_window_stub),
        "UpdateWindow" => fn_addr(joys_win_update_window_stub),
        "DestroyWindow" => fn_addr(joys_win_destroy_window_stub),
        "GetMessageA" => fn_addr(joys_win_get_message_a_stub),
        "TranslateMessage" => fn_addr(joys_win_translate_message_stub),
        "DispatchMessageA" => fn_addr(joys_win_dispatch_message_a_stub),
        "PostMessageA" => fn_addr(joys_win_post_message_a_stub),
        "PostQuitMessage" => fn_addr(joys_win_post_quit_message_stub),
        "DefWindowProcA" => fn_addr(joys_win_def_window_proc_a_stub),
        // Im modernen Windows-SDK liegen GetDC/ReleaseDC in user32.dll.
        "GetDC" => fn_addr(joys_win_get_dc_stub),
        "ReleaseDC" => fn_addr(joys_win_release_dc_stub),
        other => {
            return Err(ExeError::UnimplementedApi(
                "user32.dll".into(),
                other.into(),
            ))
        }
    };
    Ok(stub)
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn fn_addr(f: unsafe extern "C" fn()) -> usize {
    f as usize
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
extern "C" {
    fn joys_win_register_class_ex_a_stub();
    fn joys_win_create_window_ex_a_stub();
    fn joys_win_show_window_stub();
    fn joys_win_update_window_stub();
    fn joys_win_destroy_window_stub();
    fn joys_win_get_message_a_stub();
    fn joys_win_translate_message_stub();
    fn joys_win_dispatch_message_a_stub();
    fn joys_win_post_message_a_stub();
    fn joys_win_post_quit_message_stub();
    fn joys_win_def_window_proc_a_stub();
    fn joys_win_get_dc_stub();
    fn joys_win_release_dc_stub();
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
    joys_win_register_class_ex_a_stub,
    joys_win_create_window_ex_a_stub,
    joys_win_show_window_stub,
    joys_win_update_window_stub,
    joys_win_destroy_window_stub,
    joys_win_get_message_a_stub,
    joys_win_translate_message_stub,
    joys_win_dispatch_message_a_stub,
    joys_win_post_message_a_stub,
    joys_win_post_quit_message_stub,
    joys_win_def_window_proc_a_stub,
    joys_win_get_dc_stub,
    joys_win_release_dc_stub
);
