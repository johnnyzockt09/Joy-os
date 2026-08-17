//! Hardware-Informationen: CPU, Speicher.

/// Anzahl der logischen CPUs.
pub fn cpu_count() -> u32 {
    #[cfg(target_os = "linux")]
    {
        // Fallback auf sysconf, primär aus /proc/cpuinfo.
        let proc_count = std::fs::read_to_string("/proc/cpuinfo")
            .map(|s| s.lines().filter(|l| l.starts_with("processor")).count() as u32)
            .unwrap_or(0);
        if proc_count > 0 {
            return proc_count;
        }
    }
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1)
}

/// Gesamter physischer Speicher in Bytes.
pub fn total_memory_bytes() -> u64 {
    meminfo_value("MemTotal").unwrap_or(0)
}

/// Verfügbarer Speicher in Bytes.
pub fn available_memory_bytes() -> u64 {
    meminfo_value("MemAvailable").unwrap_or(0)
}

#[cfg(target_os = "linux")]
fn meminfo_value(key: &str) -> Option<u64> {
    let data = std::fs::read_to_string("/proc/meminfo").ok()?;
    let line = data.lines().find(|l| l.starts_with(key))?;
    let value = line.split_whitespace().nth(1)?;
    Some(value.parse::<u64>().ok()? * 1024)
}

#[cfg(not(target_os = "linux"))]
fn meminfo_value(_key: &str) -> Option<u64> {
    None
}

/// Übersichtliche Hardware-Beschreibung.
pub fn describe() -> String {
    format!(
        "{} CPUs, {:.0} MB RAM gesamt, {:.0} MB verfügbar",
        cpu_count(),
        total_memory_bytes() as f64 / (1024.0 * 1024.0),
        available_memory_bytes() as f64 / (1024.0 * 1024.0)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_count_is_positive() {
        assert!(cpu_count() >= 1);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn total_memory_is_positive() {
        assert!(total_memory_bytes() > 0, "MemTotal sollte > 0 sein");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn available_memory_is_positive() {
        assert!(available_memory_bytes() > 0, "MemAvailable sollte > 0 sein");
    }
}
