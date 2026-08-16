//! kernel32.dll – Builtin-Implementierung von joys-win.
//!
//! Jede Funktion wird auf Linux abgebildet (echte Wirkung, keine Dummy-Werte):
//! - GetStdHandle  -> Pseudo-Handles wie bei Windows (-10/-11/-12) zurückgeben
//! - WriteFile     -> auf stdout/stderr (fd 1/2) schreiben
//! - ExitProcess   -> Prozess beenden
//!
//! Die von Windows-Code aufgerufenen Adressen sind Win64-ABI-Stubs
//! (siehe runtime/abi.rs), die Argumente von Win64 (RCX/RDX/R8/R9) auf das
//! SysV-ABI der Rust-Impls umsetzen. Die Rust-Impls sind bewusst simpel.

use crate::loader::imports::Import;
use crate::runtime::ExeError;

/// Windows-Konstanten.
pub const STD_INPUT_HANDLE: i32 = -10;
pub const STD_OUTPUT_HANDLE: i32 = -11;
pub const STD_ERROR_HANDLE: i32 = -12;

/// Impl von `GetStdHandle`. Liefert das Windows-Pseudo-Handle zurück
/// (wie Windows es tut); WriteFile erkennt diese Werte wieder.
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub mit dem Windows-Aufrufer aufgerufen.
#[no_mangle]
pub unsafe extern "C" fn joys_win_get_std_handle_impl(n_std_handle: u32) -> i64 {
    match n_std_handle as i32 {
        STD_INPUT_HANDLE => i64::from(STD_INPUT_HANDLE),
        STD_OUTPUT_HANDLE => i64::from(STD_OUTPUT_HANDLE),
        STD_ERROR_HANDLE => i64::from(STD_ERROR_HANDLE),
        _ => 0,
    }
}

/// Impl von `WriteFile`. Schreibt den Puffer auf den passenden Linux-fd.
///
/// # Safety
/// `lp_buffer` muss auf mindestens `n_bytes` gültige Bytes zeigen und
/// `written_out` auf ein gültiges `DWORD` (oder NULL), gemäß Win32-ABI.
#[allow(clippy::too_many_arguments)]
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_write_file_impl(
    h_file: i64,
    lp_buffer: *const u8,
    n_bytes: u32,
    written_out: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> i32 {
    let fd = match h_file {
        h if h == i64::from(STD_OUTPUT_HANDLE) => 1,
        h if h == i64::from(STD_ERROR_HANDLE) => 2,
        _ => return 0,
    };
    let len = n_bytes as usize;
    if lp_buffer.is_null() || len == 0 {
        if !written_out.is_null() {
            *written_out = 0;
        }
        return 1;
    }
    let buf = core::slice::from_raw_parts(lp_buffer, len);
    let n = libc::write(fd, buf.as_ptr() as *const libc::c_void, buf.len());
    if n < 0 {
        return 0;
    }
    if !written_out.is_null() {
        *written_out = n as u32;
    }
    1
}

/// Fallback für Nicht-Unix-Plattformen (Ausführung dort ohnehin unsupported).
///
/// # Safety
/// Siehe Unix-Variante; nur für Kompilierung auf Nicht-Linux-Zielen.
#[allow(clippy::too_many_arguments)]
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_write_file_impl(
    _h_file: i64,
    _lp_buffer: *const u8,
    _n_bytes: u32,
    written_out: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> i32 {
    if !written_out.is_null() {
        *written_out = 0;
    }
    0
}

/// Impl von `ExitProcess`. Beendet den Prozess mit dem Exit-Code.
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub mit dem Windows-Aufrufer aufgerufen; kehrt
/// nie zurück.
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_exit_process_impl(exit_code: u32) -> ! {
    libc::_exit(exit_code as i32);
}

/// Fallback für Nicht-Unix-Plattformen.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_exit_process_impl(_exit_code: u32) -> ! {
    std::process::abort()
}

/// Löst einen kernel32-Import auf die passende Stub-Adresse auf.
///
/// Nicht implementierte Funktionen liefern `Err(UnimplementedApi)` – es gibt
/// hier KEINE Dummy-Rückgabewerte.
pub fn resolve(imp: &Import) -> Result<usize, ExeError> {
    let name = match imp {
        Import::ByName { name, .. } => name.as_str(),
        Import::ByOrdinal { ordinal } => {
            return Err(ExeError::UnimplementedApi(
                "kernel32.dll".into(),
                format!("#{ordinal}"),
            ));
        }
    };
    match name {
        "GetStdHandle" => Ok(fn_addr(joys_win_get_std_handle_stub)),
        "WriteFile" => Ok(fn_addr(joys_win_write_file_stub)),
        "ExitProcess" => Ok(fn_addr(joys_win_exit_process_stub)),
        other => Err(ExeError::UnimplementedApi(
            "kernel32.dll".into(),
            other.into(),
        )),
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn fn_addr(f: unsafe extern "C" fn()) -> usize {
    f as usize
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
extern "C" {
    // Win64-ABI-Stubs aus runtime/abi.rs.
    fn joys_win_get_std_handle_stub();
    fn joys_win_write_file_stub();
    fn joys_win_exit_process_stub();
}

#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
fn fn_addr(_f: usize) -> usize {
    0
}

#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
#[allow(non_upper_case_globals)]
const joys_win_get_std_handle_stub: usize = 0;
#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
#[allow(non_upper_case_globals)]
const joys_win_write_file_stub: usize = 0;
#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
#[allow(non_upper_case_globals)]
const joys_win_exit_process_stub: usize = 0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_std_handle_returns_pseudo_handles() {
        let h = unsafe { joys_win_get_std_handle_impl(STD_OUTPUT_HANDLE as u32) };
        assert_eq!(h, i64::from(STD_OUTPUT_HANDLE));
        let h2 = unsafe { joys_win_get_std_handle_impl(0) };
        assert_eq!(h2, 0);
    }
}
