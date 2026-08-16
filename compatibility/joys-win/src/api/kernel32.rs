//! kernel32.dll – Builtin-Implementierung von joys-win.
//!
//! Jede Funktion wird auf Linux abgebildet (echte Wirkung, keine Dummy-Werte):
//! - GetStdHandle          -> Pseudo-Handles wie bei Windows (-10/-11/-12)
//! - WriteFile             -> auf stdout/stderr (fd 1/2) schreiben
//! - ExitProcess           -> Prozess beenden
//! - Sleep                 -> nanosleep
//! - GetTickCount          -> CLOCK_MONOTONIC in Millisekunden
//! - GetCurrentProcess     -> Pseudo-Handle (-1)
//! - GetCurrentProcessId   -> getpid
//! - GetCurrentThreadId    -> gettid
//! - GetLastError/SetLastError -> thread-lokaler Fehlerwert
//! - VirtualAlloc/Free     -> mmap/munmap mit internem Alloc-Tracking
//! - GetSystemInfo         -> SYSTEM_INFO-Struktur (x64-Layout) mit Linux-Daten
//! - lstrlenA              -> C-String-Länge
//! - GetCommandLineA       -> leere Kommandozeile (noch kein Argv-Mapping)
//!
//! Die von Windows-Code aufgerufenen Adressen sind Win64-ABI-Stubs
//! (siehe runtime/abi.rs), die Argumente von Win64 (RCX/RDX/R8/R9) auf das
//! SysV-ABI der Rust-Impls umsetzen.

#[cfg(unix)]
use crate::api::filesystem::{
    cstr_a, win_to_linux_path, CREATE_ALWAYS, CREATE_NEW, GENERIC_WRITE, OPEN_EXISTING,
    TRUNCATE_EXISTING,
};
use crate::loader::imports::Import;
use crate::runtime::ExeError;

/// Windows-Konstanten.
pub const STD_INPUT_HANDLE: i32 = -10;
pub const STD_OUTPUT_HANDLE: i32 = -11;
pub const STD_ERROR_HANDLE: i32 = -12;
pub const PROCESSOR_ARCHITECTURE_AMD64: u16 = 9;
pub const MEM_RESERVE: u32 = 0x2000;
pub const MEM_COMMIT: u32 = 0x1000;
pub const MEM_RELEASE: u32 = 0x8000;
pub const MEM_DECOMMIT: u32 = 0x4000;
pub const PAGE_READWRITE: u32 = 0x04;

// ---------------------------------------------------------------------------
// Thread-lokaler Windows-Fehlerwert (GetLastError/SetLastError).
// ---------------------------------------------------------------------------
thread_local! {
    static LAST_ERROR: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

// ---------------------------------------------------------------------------
// Virtueller Speicher: mmap/munmap mit internem Tracking (nur Unix).
// ---------------------------------------------------------------------------
#[cfg(unix)]
static VIRTUAL_ALLOCS: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<usize, usize>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

// ---------------------------------------------------------------------------
// Impls
// ---------------------------------------------------------------------------

/// Impl von `GetStdHandle`. Liefert das Windows-Pseudo-Handle zurück.
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
        h if (0..=i32::MAX as i64).contains(&h) => h as i32, // Datei-fd
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
/// Siehe Unix-Variante.
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

/// Impl von `ExitProcess`.
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen; kehrt nie zurück.
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

/// Impl von `Sleep` (Millisekunden).
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_sleep_impl(ms: u32) {
    let t = libc::timespec {
        tv_sec: i64::from(ms / 1000),
        tv_nsec: i64::from(ms % 1000) * 1_000_000,
    };
    libc::nanosleep(&t, std::ptr::null_mut());
}

/// Fallback für Nicht-Unix-Ziele (dort ohnehin unsupported).
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_sleep_impl(_ms: u32) {}

/// Impl von `GetTickCount` – ms seit Systemstart (CLOCK_MONOTONIC).
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_get_tick_count_impl() -> u32 {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
    (ts.tv_sec as u64 * 1000 + ts.tv_nsec as u64 / 1_000_000) as u32
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_get_tick_count_impl() -> u32 {
    0
}

/// Impl von `GetCurrentProcess` – Pseudo-Handle (-1).
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_get_current_process_impl() -> i64 {
    -1
}

/// Impl von `GetCurrentProcessId`.
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[cfg(unix)]
#[no_mangle]
pub extern "C" fn joys_win_get_current_process_id_impl() -> u32 {
    unsafe { libc::getpid() as u32 }
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub extern "C" fn joys_win_get_current_process_id_impl() -> u32 {
    0
}

/// Impl von `GetCurrentThreadId`.
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[cfg(unix)]
#[no_mangle]
pub extern "C" fn joys_win_get_current_thread_id_impl() -> u32 {
    unsafe { libc::syscall(libc::SYS_gettid) as u32 }
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub extern "C" fn joys_win_get_current_thread_id_impl() -> u32 {
    0
}

/// Impl von `GetLastError`.
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_get_last_error_impl() -> u32 {
    LAST_ERROR.with(|c| c.get())
}

/// Impl von `SetLastError`.
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_set_last_error_impl(err: u32) {
    LAST_ERROR.with(|c| c.set(err));
}

/// Impl von `VirtualAlloc` (mmap).
///
/// # Safety
/// `lp_address` muss eine gültige Wunschadresse oder NULL sein (Win32-ABI).
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_virtual_alloc_impl(
    lp_address: *mut core::ffi::c_void,
    dw_size: u64,
    fl_allocation_type: u32,
    fl_protect: u32,
) -> *mut core::ffi::c_void {
    let size = dw_size as usize;
    if size == 0 {
        return std::ptr::null_mut();
    }
    let mut flags = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS;
    let addr = if lp_address.is_null() {
        std::ptr::null_mut()
    } else {
        flags |= libc::MAP_FIXED_NOREPLACE;
        lp_address
    };
    let prot = if fl_protect & PAGE_READWRITE != 0 {
        libc::PROT_READ | libc::PROT_WRITE
    } else {
        libc::PROT_NONE
    };
    let _ = fl_allocation_type; // MEM_RESERVE|MEM_COMMIT: beide abgedeckt durch mmap
    let r = libc::mmap(addr, size, prot, flags, -1, 0);
    if r == libc::MAP_FAILED {
        return std::ptr::null_mut();
    }
    if let Ok(mut map) = VIRTUAL_ALLOCS.lock() {
        map.insert(r as usize, size);
    }
    r
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_virtual_alloc_impl(
    _lp_address: *mut core::ffi::c_void,
    _dw_size: u64,
    _fl_allocation_type: u32,
    _fl_protect: u32,
) -> *mut core::ffi::c_void {
    std::ptr::null_mut()
}

/// Impl von `VirtualFree` (munmap über internes Tracking).
///
/// # Safety
/// `lp_address` muss von `VirtualAlloc` stammen (Win32-ABI).
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_virtual_free_impl(
    lp_address: *mut core::ffi::c_void,
    _dw_size: u64,
    fl_free_type: u32,
) -> i32 {
    if fl_free_type & MEM_RELEASE == 0 && fl_free_type & MEM_DECOMMIT == 0 {
        return 0;
    }
    let addr = lp_address as usize;
    let size = VIRTUAL_ALLOCS
        .lock()
        .map_or(0, |mut m| m.remove(&addr).unwrap_or(0));
    if size == 0 {
        return 0;
    }
    if libc::munmap(lp_address, size) == 0 {
        1
    } else {
        0
    }
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_virtual_free_impl(
    _lp_address: *mut core::ffi::c_void,
    _dw_size: u64,
    _fl_free_type: u32,
) -> i32 {
    0
}

/// Impl von `GetSystemInfo` – füllt SYSTEM_INFO (x64-Layout) mit Linux-Daten.
///
/// # Safety
/// `lp_system_info` muss auf mindestens 48 Bytes gültigen Speicher zeigen.
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_get_system_info_impl(lp_system_info: *mut u8) {
    if lp_system_info.is_null() {
        return;
    }
    let n = libc::sysconf(libc::_SC_NPROCESSORS_ONLN).max(1) as u32;
    // Layout (x64, 48 Bytes):
    // 0x00 u16 wProcessorArchitecture, 0x02 u16 wReserved
    // 0x04 u32 dwPageSize, 0x08 u64 lpMin, 0x10 u64 lpMax,
    // 0x18 u64 dwActiveProcessorMask, 0x20 u32 dwNumberOfProcessors,
    // 0x24 u32 dwProcessorType, 0x28 u32 dwAllocationGranularity,
    // 0x2C u16 wProcessorLevel, 0x2E u16 wProcessorRevision
    *(lp_system_info.add(0) as *mut u16) = PROCESSOR_ARCHITECTURE_AMD64;
    *(lp_system_info.add(2) as *mut u16) = 0;
    *(lp_system_info.add(4) as *mut u32) = 4096;
    *(lp_system_info.add(8) as *mut u64) = 0x1_0000;
    *(lp_system_info.add(16) as *mut u64) = 0x7fff_ffff_ffff;
    *(lp_system_info.add(24) as *mut u64) = (1u64 << n) - 1;
    *(lp_system_info.add(32) as *mut u32) = n;
    *(lp_system_info.add(36) as *mut u32) = 8664; // PROCESSOR_AMD_X8664
    *(lp_system_info.add(40) as *mut u32) = 0x1_0000;
    *(lp_system_info.add(44) as *mut u16) = 6;
    *(lp_system_info.add(46) as *mut u16) = 0;
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_get_system_info_impl(_lp_system_info: *mut u8) {}

/// Impl von `lstrlenA`.
///
/// # Safety
/// `s` muss auf eine NUL-terminierte Zeichenkette zeigen.
#[no_mangle]
pub unsafe extern "C" fn joys_win_lstrlen_a_impl(s: *const u8) -> i32 {
    if s.is_null() {
        return 0;
    }
    let mut n = 0usize;
    while *s.add(n) != 0 {
        n += 1;
    }
    n as i32
}

/// Impl von `GetCommandLineA` – vorerst leere Kommandozeile.
///
/// TODO(PHASE 7): Kommandozeile aus dem Prozess-Environment des Aufrufers
/// abbilden.
#[no_mangle]
pub extern "C" fn joys_win_get_command_line_a_impl() -> *const u8 {
    static EMPTY: [u8; 1] = [0];
    EMPTY.as_ptr()
}

/// Impl von `GetCurrentDirectoryA`.
///
/// # Safety
/// `lp_buffer` muss auf `n_buffer_length` gültige Bytes zeigen (Win32-ABI).
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_get_current_directory_a_impl(
    n_buffer_length: u32,
    lp_buffer: *mut u8,
) -> u32 {
    if lp_buffer.is_null() || n_buffer_length == 0 {
        return 0;
    }
    let mut buf = vec![0u8; n_buffer_length as usize];
    let r = libc::getcwd(buf.as_mut_ptr() as *mut libc::c_char, buf.len());
    if r.is_null() {
        return 0;
    }
    let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    std::ptr::copy_nonoverlapping(buf.as_ptr(), lp_buffer, len);
    len as u32
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_get_current_directory_a_impl(
    _n_buffer_length: u32,
    _lp_buffer: *mut u8,
) -> u32 {
    0
}

/// Impl von `SetCurrentDirectoryA`.
///
/// # Safety
/// `lp_path_name` muss auf eine NUL-terminierte Zeichenkette zeigen.
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_set_current_directory_a_impl(lp_path_name: *const u8) -> i32 {
    let Some(path) = cstr_a(lp_path_name) else {
        return 0;
    };
    let linux = win_to_linux_path(&path);
    let c = std::ffi::CString::new(linux).unwrap_or_default();
    if libc::chdir(c.as_ptr()) == 0 {
        1
    } else {
        0
    }
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_set_current_directory_a_impl(_lp_path_name: *const u8) -> i32 {
    0
}

/// Impl von `CreateFileA`.
///
/// # Safety
/// `lp_file_name` muss auf eine NUL-terminierte Zeichenkette zeigen
/// (Win32-ABI).
#[cfg(unix)]
#[allow(clippy::too_many_arguments)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_create_file_a_impl(
    lp_file_name: *const u8,
    dw_desired_access: u32,
    _dw_share_mode: u32,
    _lp_security_attributes: *mut core::ffi::c_void,
    dw_creation_disposition: u32,
    _dw_flags_and_attributes: u32,
    _h_template_file: i64,
) -> i64 {
    let Some(path) = cstr_a(lp_file_name) else {
        return -1;
    };
    let linux = win_to_linux_path(&path);
    let c = std::ffi::CString::new(linux).unwrap_or_default();

    let access = if dw_desired_access & GENERIC_WRITE != 0 {
        libc::O_RDWR
    } else {
        libc::O_RDONLY
    };
    let (flags, mode) = match dw_creation_disposition {
        CREATE_NEW => (libc::O_CREAT | libc::O_EXCL, 0o666),
        CREATE_ALWAYS => (libc::O_CREAT | libc::O_TRUNC, 0o666),
        OPEN_EXISTING => (0, 0),
        TRUNCATE_EXISTING => (libc::O_TRUNC, 0),
        _ => (0, 0),
    };
    let fd = libc::open(c.as_ptr(), flags | access, mode as libc::c_uint);
    if fd < 0 {
        -1
    } else {
        i64::from(fd)
    }
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[allow(clippy::too_many_arguments)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_create_file_a_impl(
    _lp_file_name: *const u8,
    _dw_desired_access: u32,
    _dw_share_mode: u32,
    _lp_security_attributes: *mut core::ffi::c_void,
    _dw_creation_disposition: u32,
    _dw_flags_and_attributes: u32,
    _h_template_file: i64,
) -> i64 {
    -1
}

/// Impl von `ReadFile`.
///
/// # Safety
/// `lp_buffer` muss auf `n_bytes_to_read` gültige Bytes zeigen (Win32-ABI).
#[allow(clippy::too_many_arguments)]
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_read_file_impl(
    h_file: i64,
    lp_buffer: *mut u8,
    n_bytes_to_read: u32,
    bytes_read_out: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> i32 {
    let fd = match h_file {
        h if h == i64::from(STD_INPUT_HANDLE) => 0,
        h if (0..=i32::MAX as i64).contains(&h) => h as i32,
        _ => return 0,
    };
    if lp_buffer.is_null() || n_bytes_to_read == 0 {
        if !bytes_read_out.is_null() {
            *bytes_read_out = 0;
        }
        return 0;
    }
    let buf = core::slice::from_raw_parts_mut(lp_buffer, n_bytes_to_read as usize);
    let n = libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len());
    if n < 0 {
        return 0;
    }
    if !bytes_read_out.is_null() {
        *bytes_read_out = n as u32;
    }
    1
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[allow(clippy::too_many_arguments)]
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_read_file_impl(
    _h_file: i64,
    _lp_buffer: *mut u8,
    _n_bytes_to_read: u32,
    bytes_read_out: *mut u32,
    _overlapped: *mut core::ffi::c_void,
) -> i32 {
    if !bytes_read_out.is_null() {
        *bytes_read_out = 0;
    }
    0
}

/// Impl von `GetFileSize`.
///
/// # Safety
/// `lp_file_size_high` muss gültig oder NULL sein (Win32-ABI).
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_get_file_size_impl(
    h_file: i64,
    _lp_file_size_high: *mut u32,
) -> u32 {
    if h_file < 0 {
        return u32::MAX; // INVALID_FILE_SIZE
    }
    let mut st: libc::stat = std::mem::zeroed();
    if libc::fstat(h_file as i32, &mut st) != 0 {
        return u32::MAX;
    }
    st.st_size as u32
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_get_file_size_impl(
    _h_file: i64,
    _lp_file_size_high: *mut u32,
) -> u32 {
    u32::MAX
}

/// Impl von `CloseHandle`.
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_close_handle_impl(h_object: i64) -> i32 {
    if h_object == i64::from(STD_INPUT_HANDLE)
        || h_object == i64::from(STD_OUTPUT_HANDLE)
        || h_object == i64::from(STD_ERROR_HANDLE)
    {
        return 1; // Pseudo-Handles nicht schließen (wie Windows)
    }
    if (0..=i32::MAX as i64).contains(&h_object) && libc::close(h_object as i32) == 0 {
        return 1;
    }
    0
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_close_handle_impl(_h_object: i64) -> i32 {
    0
}

// ---------------------------------------------------------------------------
// Import-Auflösung
// ---------------------------------------------------------------------------

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
    let stub = match name {
        "GetStdHandle" => fn_addr(joys_win_get_std_handle_stub),
        "WriteFile" => fn_addr(joys_win_write_file_stub),
        "ReadFile" => fn_addr(joys_win_read_file_stub),
        "CreateFileA" => fn_addr(joys_win_create_file_a_stub),
        "GetCurrentDirectoryA" => fn_addr(joys_win_get_current_directory_a_stub),
        "SetCurrentDirectoryA" => fn_addr(joys_win_set_current_directory_a_stub),
        "GetFileSize" => fn_addr(joys_win_get_file_size_stub),
        "CloseHandle" => fn_addr(joys_win_close_handle_stub),
        "ExitProcess" => fn_addr(joys_win_exit_process_stub),
        "Sleep" => fn_addr(joys_win_sleep_stub),
        "GetTickCount" => fn_addr(joys_win_get_tick_count_stub),
        "GetCurrentProcess" => fn_addr(joys_win_get_current_process_stub),
        "GetCurrentProcessId" => fn_addr(joys_win_get_current_process_id_stub),
        "GetCurrentThreadId" => fn_addr(joys_win_get_current_thread_id_stub),
        "GetLastError" => fn_addr(joys_win_get_last_error_stub),
        "SetLastError" => fn_addr(joys_win_set_last_error_stub),
        "VirtualAlloc" => fn_addr(joys_win_virtual_alloc_stub),
        "VirtualFree" => fn_addr(joys_win_virtual_free_stub),
        "GetSystemInfo" => fn_addr(joys_win_get_system_info_stub),
        "lstrlenA" => fn_addr(joys_win_lstrlen_a_stub),
        "GetCommandLineA" => fn_addr(joys_win_get_command_line_a_stub),
        other => {
            return Err(ExeError::UnimplementedApi(
                "kernel32.dll".into(),
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
    // Win64-ABI-Stubs aus runtime/abi.rs.
    fn joys_win_get_std_handle_stub();
    fn joys_win_write_file_stub();
    fn joys_win_read_file_stub();
    fn joys_win_create_file_a_stub();
    fn joys_win_get_current_directory_a_stub();
    fn joys_win_set_current_directory_a_stub();
    fn joys_win_get_file_size_stub();
    fn joys_win_close_handle_stub();
    fn joys_win_exit_process_stub();
    fn joys_win_sleep_stub();
    fn joys_win_get_tick_count_stub();
    fn joys_win_get_current_process_stub();
    fn joys_win_get_current_process_id_stub();
    fn joys_win_get_current_thread_id_stub();
    fn joys_win_get_last_error_stub();
    fn joys_win_set_last_error_stub();
    fn joys_win_virtual_alloc_stub();
    fn joys_win_virtual_free_stub();
    fn joys_win_get_system_info_stub();
    fn joys_win_lstrlen_a_stub();
    fn joys_win_get_command_line_a_stub();
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
    joys_win_get_std_handle_stub,
    joys_win_write_file_stub,
    joys_win_read_file_stub,
    joys_win_create_file_a_stub,
    joys_win_get_current_directory_a_stub,
    joys_win_set_current_directory_a_stub,
    joys_win_get_file_size_stub,
    joys_win_close_handle_stub,
    joys_win_exit_process_stub,
    joys_win_sleep_stub,
    joys_win_get_tick_count_stub,
    joys_win_get_current_process_stub,
    joys_win_get_current_process_id_stub,
    joys_win_get_current_thread_id_stub,
    joys_win_get_last_error_stub,
    joys_win_set_last_error_stub,
    joys_win_virtual_alloc_stub,
    joys_win_virtual_free_stub,
    joys_win_get_system_info_stub,
    joys_win_lstrlen_a_stub,
    joys_win_get_command_line_a_stub
);

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

    #[test]
    fn last_error_is_thread_local() {
        joys_win_set_last_error_impl(5);
        assert_eq!(joys_win_get_last_error_impl(), 5);
        joys_win_set_last_error_impl(0);
        assert_eq!(joys_win_get_last_error_impl(), 0);
    }

    #[test]
    fn lstrlen_works() {
        let s = c"Hallo";
        unsafe {
            assert_eq!(joys_win_lstrlen_a_impl(s.as_ptr().cast::<u8>()), 5);
            assert_eq!(joys_win_lstrlen_a_impl(std::ptr::null()), 0);
        }
    }

    #[test]
    #[cfg(unix)]
    fn virtual_alloc_free_roundtrip() {
        unsafe {
            let p = joys_win_virtual_alloc_impl(
                std::ptr::null_mut(),
                4096,
                MEM_RESERVE | MEM_COMMIT,
                PAGE_READWRITE,
            );
            assert!(!p.is_null());
            // Speicher tatsächlich beschreibbar.
            *(p as *mut u8) = 0x41;
            assert_eq!(*(p as *const u8), 0x41);
            let ok = joys_win_virtual_free_impl(p, 0, MEM_RELEASE);
            assert_eq!(ok, 1);
        }
    }
}
