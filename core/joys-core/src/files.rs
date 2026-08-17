//! Dateisystem-Informationen (Speicherplatz).

/// Disk-Nutzung eines Pfades.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskUsage {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
}

impl DiskUsage {
    pub fn used_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.free_bytes)
    }
}

/// Ermittelt die Disk-Nutzung des Dateisystems, das `path` enthält.
#[cfg(unix)]
pub fn disk_usage(path: &str) -> Option<DiskUsage> {
    let c = std::ffi::CString::new(path).ok()?;
    unsafe {
        let mut st: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c.as_ptr(), &mut st) != 0 {
            return None;
        }
        let bsize = st.f_frsize as u64;
        Some(DiskUsage {
            total_bytes: st.f_blocks as u64 * bsize,
            free_bytes: st.f_bfree as u64 * bsize,
            available_bytes: st.f_bavail as u64 * bsize,
        })
    }
}

/// Nicht-Unix-Fallback.
#[cfg(not(unix))]
pub fn disk_usage(_path: &str) -> Option<DiskUsage> {
    None
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::*;

    #[test]
    #[cfg(unix)]
    fn disk_usage_works() {
        let du = disk_usage("/").expect("root fs");
        assert!(du.total_bytes > 0);
        assert!(du.available_bytes > 0);
    }

    #[test]
    #[cfg(unix)]
    fn used_is_within_total() {
        let du = disk_usage("/").expect("root fs");
        assert!(du.used_bytes() <= du.total_bytes);
    }
}
