//! API-Modul von joys-win: Builtin-Implementierungen der Windows-DLLs.
//!
//! Status: kernel32 (Konsolen-/Prozess-/Speicher-/Datei-Grundfunktionen) und
//! advapi32 (Registry) mit echten, auf Linux abgebildeten Implementierungen.
//! Nicht implementierte APIs scheitern mit einer klaren Meldung.

pub mod advapi32;
pub mod filesystem;
pub mod gdi32;
pub mod kernel32;
pub mod user32;
pub mod ws2_32;
