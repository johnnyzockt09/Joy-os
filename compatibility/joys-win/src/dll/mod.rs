//! DLL-System von joys-win.
//!
//! TODO(PHASE 5-6): DLL-Discovery, Import-/Export-Auflösung, Builtin-DLLs,
//! Suchpfade, Versionsbehandlung.
//!
//! Bisher: leer – es gibt hier bewusst KEINE Fake-Implementierung.
//! Das Verzeichnis existiert, damit die Zielarchitektur sichtbar ist.

/// Prioritätsreihenfolge der unterstützten Builtin-DLLs.
pub const BUILTIN_DLLS: &[&str] = &[
    "kernel32.dll",
    "ntdll.dll",
    "user32.dll",
    "advapi32.dll",
    "gdi32.dll",
    "shell32.dll",
    "ole32.dll",
    "ws2_32.dll",
];
