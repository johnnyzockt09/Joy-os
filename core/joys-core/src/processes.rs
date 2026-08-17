//! Prozess-Informationen.

/// Anzahl laufender Prozesse (eigene Namen, ohne Threads).
pub fn process_count() -> usize {
    list_pids().len()
}

/// PIDs der laufenden Prozesse.
pub fn list_pids() -> Vec<u32> {
    #[cfg(target_os = "linux")]
    {
        let mut pids = Vec::new();
        if let Ok(rd) = std::fs::read_dir("/proc") {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.chars().all(|c| c.is_ascii_digit()) {
                    if let Ok(pid) = name.parse::<u32>() {
                        pids.push(pid);
                    }
                }
            }
        }
        pids.sort_unstable();
        pids
    }
    #[cfg(windows)]
    {
        let mut pids: Vec<u32> = std::process::Command::new("tasklist")
            .args(["/FO", "CSV", "/NH"])
            .output()
            .ok()
            .map(|out| {
                let text = String::from_utf8_lossy(&out.stdout);
                text.lines()
                    .filter_map(|l| {
                        let pid = l.split(',').nth(1)?.trim_matches('"');
                        pid.parse::<u32>().ok()
                    })
                    .collect()
            })
            .unwrap_or_default();
        pids.sort_unstable();
        pids
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_list_is_not_empty() {
        // Der eigene Testprozess existiert immer.
        assert!(process_count() >= 1);
    }

    #[test]
    fn pids_are_sorted_and_unique() {
        let pids = list_pids();
        let mut sorted = pids.clone();
        sorted.dedup();
        assert_eq!(pids.len(), sorted.len(), "PIDs sollten eindeutig sein");
        assert!(pids.windows(2).all(|w| w[0] <= w[1]), "PIDs sortiert");
    }
}
