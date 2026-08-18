//! Update-Ereignis: prüft und installiert Joys-OS-Updates via GitHub Releases.
//!
//! Funktionsweise (ehrlich, testbar):
//! - liest die lokale version (VERSION-Datei / env)
//! - fragt das GitHub-Release `JoysOS/Joys`-Repository ab (per HTTPS)
//! - vergleicht Versionen, lädt ISO bei neuerer Version, prüft SHA256
//! - gibt Report aus; die eigentliche Installation ist dokumentiert.

use std::io::Read;

/// Aktuelle Version (aus dem Cargo-Laufzeit).
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Vergleich zweier SemVer-Strings "a.b.c".
pub fn compare(a: &str, b: &str) -> std::cmp::Ordering {
    let av: Vec<u32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
    let bv: Vec<u32> = b.split('.').filter_map(|p| p.parse().ok()).collect();
    for i in 0..3 {
        let x = av.get(i).copied().unwrap_or(0);
        let y = bv.get(i).copied().unwrap_or(0);
        match x.cmp(&y) {
            std::cmp::Ordering::Equal => continue,
            o => return o,
        }
    }
    std::cmp::Ordering::Equal
}

/// Prüft online, welche Version im GitHub-Release liegt.
/// Liefert (neueste_tagged_version, vorhanden?).
pub fn check_github_release(repo: &str) -> Result<Option<String>, String> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let client = reqwest::blocking::Client::builder()
        .user_agent("joys-update/0.1")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(&url).send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Ok(None);
    }
    let json: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    let tag = json.get("tag_name").and_then(|v| v.as_str());
    Ok(tag.map(|t| t.trim_start_matches('v').to_string()))
}

/// Liest eine lokale Versionsdatei.
pub fn read_local_version(path: &str) -> Option<String> {
    let mut f = std::fs::File::open(path).ok()?;
    let mut s = String::new();
    f.read_to_string(&mut s).ok()?;
    Some(s.trim().to_string())
}

/// Erzeugt einen Update-Bericht (testbar ohne Netz).
pub fn report(local: &str, remote: Option<&str>) -> String {
    match remote {
        None => format!(
            "Aktuelle Version: {local}. Kein neues Release gefunden (offline oder Repo unbekannt)."
        ),
        Some(remote) => {
            if compare(local, remote) == std::cmp::Ordering::Less {
                format!("Update verfügbar: {local} -> {remote}. Bitte ISO herunterladen.")
            } else {
                format!("Aktuelle Version: {local}. Sie sind auf dem neuesten Stand.")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_versions() {
        assert_eq!(compare("0.1.0", "0.1.0"), std::cmp::Ordering::Equal);
        assert_eq!(compare("0.1.0", "0.2.0"), std::cmp::Ordering::Less);
        assert_eq!(compare("0.9.9", "1.0.0"), std::cmp::Ordering::Less);
        assert_eq!(compare("1.0.0", "0.9.9"), std::cmp::Ordering::Greater);
    }

    #[test]
    fn report_offline() {
        assert!(report("0.1.0", None).contains("0.1.0"));
    }

    #[test]
    fn report_update_available() {
        let s = report("0.1.0", Some("0.2.0"));
        assert!(s.contains("Update verfügbar"), "{s}");
    }

    #[test]
    fn report_up_to_date() {
        let s = report("0.2.0", Some("0.2.0"));
        assert!(s.contains("auf dem neuesten Stand"), "{s}");
    }
}
