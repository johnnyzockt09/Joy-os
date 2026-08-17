//! ws2_32.dll – Builtin-Implementierung von joys-win (PHASE 11).
//!
//! Abbildung der Windows-Socket-API auf Linux-Sockets:
//! - socket/bind/connect/listen/accept/send/recv/closesocket/getsockname
//! - WSAStartup/WSACleanup/WSAGetLastError
//! - htons/htonl/inet_addr (Netzwerk-Byteorder, echte Umrechnung)
//!
//! Socket-Handles sind die Linux-fds. Das Windows-sockaddr_in hat auf x86_64
//! dasselbe Speicher-Layout wie das Linux-sockaddr_in (family LE, port BE,
//! addr BE, zero[8]) – die Pointer werden daher direkt an libc übergeben.

use crate::runtime::ExeError;

thread_local! {
    static WS_LAST_ERROR: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

#[cfg(unix)]
fn set_last_errno() {
    WS_LAST_ERROR.with(|c| c.set(std::io::Error::last_os_error().raw_os_error().unwrap_or(0)));
}

#[cfg(not(unix))]
#[allow(dead_code)]
fn set_last_errno() {}

// ---------------------------------------------------------------------------
// Impls (Win64-ABI, von den Stubs in runtime/abi.rs aufgerufen)
// ---------------------------------------------------------------------------

/// WSAStartup(WORD, LPWSADATA) -> int
///
/// # Safety
/// `lp_wsadata` muss auf ein gültiges WSADATA (408 Bytes) zeigen.
#[no_mangle]
pub unsafe extern "C" fn joys_win_wsastartup_impl(
    _w_version_requested: u32,
    lp_wsadata: *mut u8,
) -> i32 {
    if lp_wsadata.is_null() {
        return 0;
    }
    // WSADATA (x64): wVersion@0, wHighVersion@2, ...
    *(lp_wsadata as *mut u16) = 0x0202; // Version 2.2
    *(lp_wsadata.add(2) as *mut u16) = 0x0202;
    WS_LAST_ERROR.with(|c| c.set(0));
    0
}

/// WSACleanup(void) -> int
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_wsacleanup_impl() -> i32 {
    0
}

/// socket(int, int, int) -> SOCKET
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[cfg(unix)]
#[no_mangle]
pub extern "C" fn joys_win_socket_impl(af: i32, ty: i32, proto: i32) -> i64 {
    let fd = unsafe { libc::socket(af, ty, proto) };
    if fd < 0 {
        set_last_errno();
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
#[no_mangle]
pub extern "C" fn joys_win_socket_impl(_af: i32, _ty: i32, _proto: i32) -> i64 {
    -1
}

/// bind(SOCKET, sockaddr*, int) -> int
///
/// # Safety
/// `addr` muss auf ein gültiges sockaddr zeigen (Win32-ABI).
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_bind_impl(
    sock: i64,
    addr: *const libc::sockaddr,
    len: i32,
) -> i32 {
    let r = libc::bind(sock as i32, addr, len as libc::socklen_t);
    if r != 0 {
        set_last_errno();
    }
    r
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_bind_impl(
    _sock: i64,
    _addr: *const libc::sockaddr,
    _len: i32,
) -> i32 {
    -1
}

/// connect(SOCKET, sockaddr*, int) -> int
///
/// # Safety
/// `addr` muss auf ein gültiges sockaddr zeigen (Win32-ABI).
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_connect_impl(
    sock: i64,
    addr: *const libc::sockaddr,
    len: i32,
) -> i32 {
    let r = libc::connect(sock as i32, addr, len as libc::socklen_t);
    if r != 0 {
        set_last_errno();
    }
    r
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_connect_impl(
    _sock: i64,
    _addr: *const libc::sockaddr,
    _len: i32,
) -> i32 {
    -1
}

/// listen(SOCKET, int) -> int
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[cfg(unix)]
#[no_mangle]
pub extern "C" fn joys_win_listen_impl(sock: i64, backlog: i32) -> i32 {
    let r = unsafe { libc::listen(sock as i32, backlog) };
    if r != 0 {
        set_last_errno();
    }
    r
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub extern "C" fn joys_win_listen_impl(_sock: i64, _backlog: i32) -> i32 {
    -1
}

/// accept(SOCKET, sockaddr*, int*) -> SOCKET
///
/// # Safety
/// `addr`/`len` müssen gültig oder NULL sein (Win32-ABI).
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_accept_impl(
    sock: i64,
    addr: *mut libc::sockaddr,
    len: *mut i32,
) -> i64 {
    let mut l: libc::socklen_t = len.as_ref().map_or(0, |&v| v as libc::socklen_t);
    let fd = libc::accept(
        sock as i32,
        addr,
        if len.is_null() {
            std::ptr::null_mut()
        } else {
            &mut l
        },
    );
    if fd < 0 {
        set_last_errno();
        -1
    } else {
        if !len.is_null() {
            *len = l as i32;
        }
        i64::from(fd)
    }
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_accept_impl(
    _sock: i64,
    _addr: *mut libc::sockaddr,
    _len: *mut i32,
) -> i64 {
    -1
}

/// send(SOCKET, const char*, int, int) -> int
///
/// # Safety
/// `buf` muss auf `len` gültige Bytes zeigen (Win32-ABI).
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_send_impl(
    sock: i64,
    buf: *const u8,
    len: i32,
    _flags: i32,
) -> i32 {
    let n = libc::send(sock as i32, buf as *const libc::c_void, len as usize, 0);
    if n < 0 {
        set_last_errno();
    }
    n as i32
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_send_impl(
    _sock: i64,
    _buf: *const u8,
    _len: i32,
    _flags: i32,
) -> i32 {
    -1
}

/// recv(SOCKET, char*, int, int) -> int
///
/// # Safety
/// `buf` muss auf `len` gültige Bytes zeigen (Win32-ABI).
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_recv_impl(sock: i64, buf: *mut u8, len: i32, _flags: i32) -> i32 {
    let n = libc::recv(sock as i32, buf as *mut libc::c_void, len as usize, 0);
    if n < 0 {
        set_last_errno();
    }
    n as i32
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_recv_impl(
    _sock: i64,
    _buf: *mut u8,
    _len: i32,
    _flags: i32,
) -> i32 {
    -1
}

/// closesocket(SOCKET) -> int
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[cfg(unix)]
#[no_mangle]
pub extern "C" fn joys_win_closesocket_impl(sock: i64) -> i32 {
    let r = unsafe { libc::close(sock as i32) };
    if r != 0 {
        set_last_errno();
    }
    r
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub extern "C" fn joys_win_closesocket_impl(_sock: i64) -> i32 {
    -1
}

/// getsockname(SOCKET, sockaddr*, int*) -> int
///
/// # Safety
/// `addr`/`len` müssen gültig sein (Win32-ABI).
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_getsockname_impl(
    sock: i64,
    addr: *mut libc::sockaddr,
    len: *mut i32,
) -> i32 {
    let mut l: libc::socklen_t = len.as_ref().map_or(0, |&v| v as libc::socklen_t);
    let r = libc::getsockname(sock as i32, addr, &mut l);
    if r == 0 && !len.is_null() {
        *len = l as i32;
    } else if r != 0 {
        set_last_errno();
    }
    r
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_getsockname_impl(
    _sock: i64,
    _addr: *mut libc::sockaddr,
    _len: *mut i32,
) -> i32 {
    -1
}

/// WSAGetLastError(void) -> int
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_wsagetlasterror_impl() -> i32 {
    WS_LAST_ERROR.with(|c| c.get())
}

/// htons(u16) -> u16
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_htons_impl(v: u16) -> u16 {
    v.to_be()
}

/// htonl(u32) -> u32
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[no_mangle]
pub extern "C" fn joys_win_htonl_impl(v: u32) -> u32 {
    v.to_be()
}

/// inet_addr(const char*) -> u32 (INADDR_NONE = u32::MAX bei Fehler)
///
/// # Safety
/// `cp` muss auf einen NUL-terminierten String zeigen.
#[no_mangle]
pub unsafe extern "C" fn joys_win_inet_addr_impl(cp: *const u8) -> u32 {
    let Some(s) = crate::api::filesystem::cstr_a(cp) else {
        return u32::MAX;
    };
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return u32::MAX;
    }
    let mut v: u32 = 0;
    for p in parts {
        let Ok(n) = p.parse::<u32>() else {
            return u32::MAX;
        };
        if n > 255 {
            return u32::MAX;
        }
        v = (v << 8) | n;
    }
    v.to_be() // Netzwerk-Byteorder
}

// ---------------------------------------------------------------------------
// Import-Auflösung
// ---------------------------------------------------------------------------

/// Löst einen ws2_32-Import auf die passende Stub-Adresse auf.
///
/// MSVC importiert ws2_32 meist per Ordinal; daher gibt es eine
/// Ordinal-Tabelle (aus C:\Windows\System32\ws2_32.dll übernommen).
/// Nicht implementierte Funktionen liefern `Err(UnimplementedApi)`.
pub fn resolve(imp: &crate::loader::imports::Import) -> Result<usize, ExeError> {
    let stub = match imp {
        crate::loader::imports::Import::ByName { name, .. } => match name.as_str() {
            "WSAStartup" => Some(fn_addr(joys_win_wsastartup_stub)),
            "WSACleanup" => Some(fn_addr(joys_win_wsacleanup_stub)),
            "WSAGetLastError" => Some(fn_addr(joys_win_wsagetlasterror_stub)),
            "socket" => Some(fn_addr(joys_win_socket_stub)),
            "bind" => Some(fn_addr(joys_win_bind_stub)),
            "connect" => Some(fn_addr(joys_win_connect_stub)),
            "listen" => Some(fn_addr(joys_win_listen_stub)),
            "accept" => Some(fn_addr(joys_win_accept_stub)),
            "send" => Some(fn_addr(joys_win_send_stub)),
            "recv" => Some(fn_addr(joys_win_recv_stub)),
            "closesocket" => Some(fn_addr(joys_win_closesocket_stub)),
            "getsockname" => Some(fn_addr(joys_win_getsockname_stub)),
            "htons" => Some(fn_addr(joys_win_htons_stub)),
            "htonl" => Some(fn_addr(joys_win_htonl_stub)),
            "inet_addr" => Some(fn_addr(joys_win_inet_addr_stub)),
            other => {
                return Err(ExeError::UnimplementedApi(
                    "ws2_32.dll".into(),
                    other.into(),
                ))
            }
        },
        crate::loader::imports::Import::ByOrdinal { ordinal } => match *ordinal {
            115 => Some(fn_addr(joys_win_wsastartup_stub)),
            116 => Some(fn_addr(joys_win_wsacleanup_stub)),
            111 => Some(fn_addr(joys_win_wsagetlasterror_stub)),
            23 => Some(fn_addr(joys_win_socket_stub)),
            2 => Some(fn_addr(joys_win_bind_stub)),
            4 => Some(fn_addr(joys_win_connect_stub)),
            13 => Some(fn_addr(joys_win_listen_stub)),
            1 => Some(fn_addr(joys_win_accept_stub)),
            19 => Some(fn_addr(joys_win_send_stub)),
            16 => Some(fn_addr(joys_win_recv_stub)),
            3 => Some(fn_addr(joys_win_closesocket_stub)),
            6 => Some(fn_addr(joys_win_getsockname_stub)),
            9 => Some(fn_addr(joys_win_htons_stub)),
            8 => Some(fn_addr(joys_win_htonl_stub)),
            11 => Some(fn_addr(joys_win_inet_addr_stub)),
            other => {
                return Err(ExeError::UnimplementedApi(
                    "ws2_32.dll".into(),
                    format!("#{other}"),
                ))
            }
        },
    };
    Ok(stub.expect("resolve liefert immer Some"))
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn fn_addr(f: unsafe extern "C" fn()) -> usize {
    f as usize
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
extern "C" {
    fn joys_win_wsastartup_stub();
    fn joys_win_wsacleanup_stub();
    fn joys_win_socket_stub();
    fn joys_win_bind_stub();
    fn joys_win_connect_stub();
    fn joys_win_listen_stub();
    fn joys_win_accept_stub();
    fn joys_win_send_stub();
    fn joys_win_recv_stub();
    fn joys_win_closesocket_stub();
    fn joys_win_getsockname_stub();
    fn joys_win_wsagetlasterror_stub();
    fn joys_win_htons_stub();
    fn joys_win_htonl_stub();
    fn joys_win_inet_addr_stub();
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
    joys_win_wsastartup_stub,
    joys_win_wsacleanup_stub,
    joys_win_socket_stub,
    joys_win_bind_stub,
    joys_win_connect_stub,
    joys_win_listen_stub,
    joys_win_accept_stub,
    joys_win_send_stub,
    joys_win_recv_stub,
    joys_win_closesocket_stub,
    joys_win_getsockname_stub,
    joys_win_wsagetlasterror_stub,
    joys_win_htons_stub,
    joys_win_htonl_stub,
    joys_win_inet_addr_stub
);
