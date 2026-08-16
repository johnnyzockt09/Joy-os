//! System-Modul von joys-core.
//!
//! TODO(PHASE 3): Hardware-Erkennung, Prozess-/Thread-Verwaltung,
//! Dateisystem, Netzwerk, Grafik, Audio, Benutzer, Permissions, Updates.

/// Zentrale Versionskonstante für Joys (wird aus dem Workspace gezogen).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Architektur, auf der Joys läuft (Host-Target).
pub fn host_arch() -> &'static str {
    std::env::consts::ARCH
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
}
