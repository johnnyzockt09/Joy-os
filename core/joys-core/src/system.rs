//! System-Informationen: Architektur, Hostname, Kernel, Uptime.

#[cfg(target_os = "linux")]
use std::io::Read;

/// Zentrale Versionskonstante für Joys (aus dem Workspace).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Architektur, auf der Joys läuft (Host-Target).
pub fn host_arch() -> &'static str {
    std::env::consts::ARCH
}

/// Betriebssystem-Name (z. B. "linux").
pub fn os_name() -> &'static str {
    std::env::consts::OS
}

/// Hostname.
pub fn hostname() -> String {
    #[cfg(unix)]
    {
        let mut buf = [0u8; 256];
        // SAFETY: gethostname schreibt maximal buf.len() Bytes in buf.
        unsafe {
            if libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) == 0 {
                let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
                return String::from_utf8_lossy(&buf[..end]).into_owned();
            }
        }
    }
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

/// Linux-Kernel-Version (uname -r) bzw. Windows-Version.
pub fn kernel_release() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(mut f) = std::fs::File::open("/proc/sys/kernel/osrelease") {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                return s.trim().to_string();
            }
        }
    }
    std::env::consts::OS.to_string()
}

/// Uptime in Sekunden (seit Systemstart).
pub fn uptime_secs() -> u64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(mut f) = std::fs::File::open("/proc/uptime") {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                if let Some(secs) = s
                    .split_whitespace()
                    .next()
                    .and_then(|v| v.parse::<f64>().ok())
                {
                    return secs as u64;
                }
            }
        }
    }
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn host_arch_is_known() {
        let a = host_arch();
        assert!(a == "x86_64" || a == "aarch64", "unknown arch: {a}");
    }

    #[test]
    fn hostname_is_not_empty() {
        assert!(!hostname().is_empty());
    }

    #[test]
    fn kernel_release_is_not_empty() {
        assert!(!kernel_release().is_empty());
    }

    #[test]
    fn uptime_is_positive() {
        assert!(uptime_secs() > 0);
    }
}
