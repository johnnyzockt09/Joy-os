//! Fehler und Ablaufsteuerung für die Ausführung von Windows-Programmen.

use std::fmt;

/// Fehler beim Laden/Ausführen eines PE-Images.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExeError {
    /// Ausführung wird auf dieser Plattform noch nicht unterstützt.
    UnsupportedPlatform,
    /// Speicher-Mapping fehlgeschlagen (z. B. mmap/mprotect).
    MapFailed(String),
    /// Ein Import ist nicht auflösbar (keine Fake-Implementierung!).
    UnimplementedApi(String, String),
    /// Unbekannter Relocation-Typ.
    UnsupportedRelocation(u16),
    /// Das Image hat keinen ausführbaren Entry-Point.
    NoEntryPoint,
}

impl fmt::Display for ExeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExeError::UnsupportedPlatform => {
                write!(
                    f,
                    "Ausführung wird auf dieser Plattform noch nicht unterstützt"
                )
            }
            ExeError::MapFailed(m) => write!(f, "Mapping fehlgeschlagen: {m}"),
            ExeError::UnimplementedApi(dll, api) => write!(
                f,
                "API {dll}!{api} ist noch nicht implementiert (joys-win). Kein Dummy."
            ),
            ExeError::UnsupportedRelocation(t) => {
                write!(f, "Relocation-Typ {t} wird nicht unterstützt")
            }
            ExeError::NoEntryPoint => write!(f, "Kein ausführbarer Entry-Point"),
        }
    }
}

impl std::error::Error for ExeError {}
