//! Windows-Dateisystem-API (kernel32-Dateifunktionen) mit Pfad-Abbildung
//! auf Linux.
//!
//! Konzept:
//! - Relative Windows-Pfade werden direkt auf das Linux-CWD abgebildet.
//! - `C:\...`-Pfade werden auf `~/.joys/windows/...` (virtuelles Laufwerk)
//!   abgebildet, damit Windows-Anwendungen nichts von Linux wissen müssen.
//! - Datei-Handles sind die Linux-fd-Werte (positiv), Pseudo-Handles
//!   (stdout/stderr) sind negativ -> keine Kollision.
//!
//! TODO(PHASE 8): CreateFileW, FindFirstFile/FindNextFile, GetFullPathName,
//! Directory-Aufzählung, Abbildung weiterer Windows-Pfade.

use std::path::PathBuf;

/// Windows-Datei-Konstanten.
pub const GENERIC_READ: u32 = 0x8000_0000;
pub const GENERIC_WRITE: u32 = 0x4000_0000;
pub const CREATE_NEW: u32 = 1;
pub const CREATE_ALWAYS: u32 = 2;
pub const OPEN_EXISTING: u32 = 3;
pub const OPEN_ALWAYS: u32 = 4;
pub const TRUNCATE_EXISTING: u32 = 5;

/// Basisverzeichnis für das virtuelle Windows-Laufwerk.
fn joys_windows_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".joys").join("windows")
    } else {
        PathBuf::from(".joys").join("windows")
    }
}

/// Bildet einen Windows-Pfad auf einen Linux-Pfad ab.
pub fn win_to_linux_path(win: &str) -> String {
    let s = win.trim_matches('"');
    // `C:\...` bzw. `C:/...` -> ~/.joys/windows/...
    if s.len() >= 3
        && s.as_bytes()[1] == b':'
        && (s.as_bytes()[2] == b'\\' || s.as_bytes()[2] == b'/')
    {
        let rest = &s[3..];
        let mut p = joys_windows_dir();
        for part in rest.split(['\\', '/']).filter(|p| !p.is_empty()) {
            p.push(part);
        }
        return p.to_string_lossy().into_owned();
    }
    s.replace('\\', "/")
}

/// Liest einen NUL-terminierten C-String (ANSI).
///
/// # Safety
/// `p` muss auf einen gültigen NUL-terminierten String zeigen.
pub unsafe fn cstr_a(p: *const u8) -> Option<String> {
    if p.is_null() {
        return None;
    }
    let mut end = 0usize;
    while unsafe { *p.add(end) } != 0 {
        end += 1;
        if end > 4096 {
            return None;
        }
    }
    let bytes = unsafe { std::slice::from_raw_parts(p, end) };
    Some(String::from_utf8_lossy(bytes).into_owned())
}
