//! advapi32.dll – Registry-Builtin von joys-win.
//!
//! Joys besitzt eine EIGENE Registry-Struktur (keine Kopie der Windows-
//! Registry): abgelegt unter `~/.joys/windows/registry/`.
//! Schlüssel = Verzeichnisse, Werte = Dateien.
//!
//! HKEY_Pseudo-Handles werden über eine Handle-Tabelle auf Schlüsselpfade
//! abgebildet. Nicht vorhandene Schlüssel/Werte -> Windows-Fehlercodes
//! (ERROR_FILE_NOT_FOUND = 2), keine Dummies.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::runtime::ExeError;

/// Windows-Fehlercodes.
pub const ERROR_SUCCESS: i32 = 0;
pub const ERROR_FILE_NOT_FOUND: i32 = 2;

/// HKEY-Konstanten (wie Windows: sign-extended auf 64-bit, da HKEY = Zeiger).
pub const HKEY_CLASSES_ROOT: usize = 0xffff_ffff_8000_0000;
pub const HKEY_CURRENT_USER: usize = 0xffff_ffff_8000_0001;
pub const HKEY_LOCAL_MACHINE: usize = 0xffff_ffff_8000_0002;
pub const HKEY_USERS: usize = 0xffff_ffff_8000_0003;

/// Werttypen.
pub const REG_SZ: u32 = 1;
pub const REG_DWORD: u32 = 4;

fn root_name(hkey: usize) -> &'static str {
    match hkey {
        HKEY_CLASSES_ROOT => "HKCR",
        HKEY_CURRENT_USER => "HKCU",
        HKEY_LOCAL_MACHINE => "HKLM",
        HKEY_USERS => "HKU",
        _ => "UNKNOWN",
    }
}

fn registry_base() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".joys")
            .join("windows")
            .join("registry")
    } else {
        PathBuf::from(".joys").join("windows").join("registry")
    }
}

fn key_path(hkey: usize, subkey: &str) -> PathBuf {
    let mut p = registry_base().join(root_name(hkey));
    for part in subkey.split('\\').filter(|p| !p.is_empty()) {
        p.push(part);
    }
    p
}

/// Handle-Tabelle: HKEY -> Schlüsselpfad.
static HANDLES: std::sync::LazyLock<std::sync::Mutex<HashMap<usize, PathBuf>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
static NEXT_HANDLE: AtomicUsize = AtomicUsize::new(0x1000);

fn alloc_handle(path: PathBuf) -> usize {
    let h = NEXT_HANDLE.fetch_add(1, Ordering::SeqCst);
    if let Ok(mut m) = HANDLES.lock() {
        m.insert(h, path);
    }
    h
}

fn handle_path(h: usize) -> Option<PathBuf> {
    HANDLES.lock().ok()?.get(&h).cloned()
}

/// Löst einen Import der advapi32.dll auf.
///
/// Nicht implementierte Funktionen liefern `Err(UnimplementedApi)`.
pub fn resolve(imp: &crate::loader::imports::Import) -> Result<usize, ExeError> {
    let name = match imp {
        crate::loader::imports::Import::ByName { name, .. } => name.as_str(),
        crate::loader::imports::Import::ByOrdinal { ordinal } => {
            return Err(ExeError::UnimplementedApi(
                "advapi32.dll".into(),
                format!("#{ordinal}"),
            ))
        }
    };
    let stub = match name {
        "RegCreateKeyA" => fn_addr(joys_win_reg_create_key_a_stub),
        "RegOpenKeyExA" => fn_addr(joys_win_reg_open_key_ex_a_stub),
        "RegSetValueExA" => fn_addr(joys_win_reg_set_value_ex_a_stub),
        "RegQueryValueExA" => fn_addr(joys_win_reg_query_value_ex_a_stub),
        "RegDeleteKeyA" => fn_addr(joys_win_reg_delete_key_a_stub),
        "RegCloseKey" => fn_addr(joys_win_reg_close_key_stub),
        other => {
            return Err(ExeError::UnimplementedApi(
                "advapi32.dll".into(),
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
    fn joys_win_reg_create_key_a_stub();
    fn joys_win_reg_open_key_ex_a_stub();
    fn joys_win_reg_set_value_ex_a_stub();
    fn joys_win_reg_query_value_ex_a_stub();
    fn joys_win_reg_delete_key_a_stub();
    fn joys_win_reg_close_key_stub();
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
    joys_win_reg_create_key_a_stub,
    joys_win_reg_open_key_ex_a_stub,
    joys_win_reg_set_value_ex_a_stub,
    joys_win_reg_query_value_ex_a_stub,
    joys_win_reg_delete_key_a_stub,
    joys_win_reg_close_key_stub
);

// ---------------------------------------------------------------------------
// Impls (Win64-ABI, von den Stubs in runtime/abi.rs aufgerufen)
// ---------------------------------------------------------------------------

/// RegCreateKeyA(HKEY hKey, LPCSTR lpSubKey, PHKEY phkResult) -> LONG
///
/// # Safety
/// Zeiger-Argumente müssen gültig sein (Win32-ABI).
#[no_mangle]
pub unsafe extern "C" fn joys_win_reg_create_key_a_impl(
    hkey: usize,
    subkey: *const u8,
    out_handle: *mut usize,
) -> i32 {
    let Some(name) = crate::api::filesystem::cstr_a(subkey) else {
        return ERROR_FILE_NOT_FOUND;
    };
    let path = key_path(hkey, &name);
    if std::fs::create_dir_all(&path).is_err() {
        return 5; // ERROR_ACCESS_DENIED
    }
    let h = alloc_handle(path);
    if !out_handle.is_null() {
        *out_handle = h;
    }
    ERROR_SUCCESS
}

/// RegOpenKeyExA(HKEY hKey, LPCSTR lpSubKey, DWORD ulOptions,
///               REGSAM samDesired, PHKEY phkResult) -> LONG
///
/// # Safety
/// Zeiger-Argumente müssen gültig sein (Win32-ABI).
#[no_mangle]
pub unsafe extern "C" fn joys_win_reg_open_key_ex_a_impl(
    hkey: usize,
    subkey: *const u8,
    _ul_options: u32,
    _sam_desired: u32,
    out_handle: *mut usize,
) -> i32 {
    let Some(name) = crate::api::filesystem::cstr_a(subkey) else {
        return ERROR_FILE_NOT_FOUND;
    };
    let path = key_path(hkey, &name);
    if !path.is_dir() {
        return ERROR_FILE_NOT_FOUND;
    }
    let h = alloc_handle(path);
    if !out_handle.is_null() {
        *out_handle = h;
    }
    ERROR_SUCCESS
}

/// RegSetValueExA(HKEY hKey, LPCSTR lpValueName, DWORD Reserved, DWORD dwType,
///                const BYTE *lpData, DWORD cbData) -> LONG
///
/// # Safety
/// Zeiger-Argumente müssen gültig sein (Win32-ABI).
#[no_mangle]
pub unsafe extern "C" fn joys_win_reg_set_value_ex_a_impl(
    hkey: usize,
    value_name: *const u8,
    _reserved: u32,
    dw_type: u32,
    lp_data: *const u8,
    cb_data: u32,
) -> i32 {
    let Some(path) = handle_path(hkey) else {
        return ERROR_FILE_NOT_FOUND;
    };
    let name = crate::api::filesystem::cstr_a(value_name).unwrap_or_default();
    let value_file = path.join(format!("{name}@{dw_type}"));
    let data = if lp_data.is_null() {
        Vec::new()
    } else {
        std::slice::from_raw_parts(lp_data, cb_data as usize).to_vec()
    };
    if std::fs::write(&value_file, data).is_ok() {
        ERROR_SUCCESS
    } else {
        5
    }
}

/// RegQueryValueExA(HKEY hKey, LPCSTR lpValueName, LPDWORD lpReserved,
///                  LPDWORD lpType, LPBYTE lpData, LPDWORD lpcbData) -> LONG
///
/// # Safety
/// Zeiger-Argumente müssen gültig sein (Win32-ABI).
#[no_mangle]
pub unsafe extern "C" fn joys_win_reg_query_value_ex_a_impl(
    hkey: usize,
    value_name: *const u8,
    _reserved: *mut u32,
    out_type: *mut u32,
    out_data: *mut u8,
    inout_size: *mut u32,
) -> i32 {
    let Some(path) = handle_path(hkey) else {
        return ERROR_FILE_NOT_FOUND;
    };
    let name = crate::api::filesystem::cstr_a(value_name).unwrap_or_default();
    // Datei-Suffix @Typ ermitteln.
    let mut entry: Option<(String, Vec<u8>)> = None;
    if let Ok(rd) = std::fs::read_dir(&path) {
        for e in rd.flatten() {
            let fname = e.file_name().to_string_lossy().into_owned();
            if let Some(typ) = fname.strip_prefix(&name).and_then(|r| r.strip_prefix('@')) {
                entry = Some((typ.to_string(), std::fs::read(e.path()).unwrap_or_default()));
                break;
            }
        }
    }
    let Some((typ, data)) = entry else {
        return ERROR_FILE_NOT_FOUND;
    };
    let typ: u32 = typ.parse().unwrap_or(REG_SZ);
    if !out_type.is_null() {
        *out_type = typ;
    }
    let size_in = if inout_size.is_null() { 0 } else { *inout_size };
    if !out_data.is_null() {
        let n = size_in.min(data.len() as u32) as usize;
        std::ptr::copy_nonoverlapping(data.as_ptr(), out_data, n);
    }
    if !inout_size.is_null() {
        *inout_size = data.len() as u32;
    }
    ERROR_SUCCESS
}

/// RegDeleteKeyA(HKEY hKey, LPCSTR lpSubKey) -> LONG
///
/// # Safety
/// Zeiger-Argumente müssen gültig sein (Win32-ABI).
#[no_mangle]
pub unsafe extern "C" fn joys_win_reg_delete_key_a_impl(hkey: usize, subkey: *const u8) -> i32 {
    let Some(name) = crate::api::filesystem::cstr_a(subkey) else {
        return ERROR_FILE_NOT_FOUND;
    };
    let path = key_path(hkey, &name);
    if std::fs::remove_dir_all(&path).is_ok() {
        ERROR_SUCCESS
    } else {
        ERROR_FILE_NOT_FOUND
    }
}

/// RegCloseKey(HKEY hKey) -> LONG
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_reg_close_key_impl(hkey: usize) -> i32 {
    if let Ok(mut m) = HANDLES.lock() {
        m.remove(&hkey);
    }
    ERROR_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_roundtrip() {
        // Eigene Test-Registry unter einem Temp-Verzeichnis.
        std::env::set_var("HOME", std::env::temp_dir());
        unsafe {
            let mut hkey = 0usize;
            let subkey = c"Software\\Joys\\UnitTest";
            let rc = joys_win_reg_create_key_a_impl(
                HKEY_CURRENT_USER,
                subkey.as_ptr().cast::<u8>(),
                &mut hkey,
            );
            assert_eq!(rc, ERROR_SUCCESS);
            assert_ne!(hkey, 0);

            let name = c"Greeting";
            let val = b"Servus!\0";
            let rc = joys_win_reg_set_value_ex_a_impl(
                hkey,
                name.as_ptr().cast::<u8>(),
                0,
                REG_SZ,
                val.as_ptr(),
                val.len() as u32,
            );
            assert_eq!(rc, ERROR_SUCCESS);

            let mut typ = 0u32;
            let mut out = [0u8; 64];
            let mut size = out.len() as u32;
            let rc = joys_win_reg_query_value_ex_a_impl(
                hkey,
                name.as_ptr().cast::<u8>(),
                std::ptr::null_mut(),
                &mut typ,
                out.as_mut_ptr(),
                &mut size,
            );
            assert_eq!(rc, ERROR_SUCCESS);
            assert_eq!(typ, REG_SZ);
            assert_eq!(&out[..7], b"Servus!");

            joys_win_reg_close_key_impl(hkey);
        }
    }
}
