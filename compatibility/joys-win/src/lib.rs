//! joys-win: Windows-Kompatibilitätsruntime von Joys OS.
//!
//! Ziel: Windows-Programme (.exe/.dll) direkt über eine eigene Runtime
//! ausführen - OHNE Wine als verpflichtende Laufzeit.
//!
//! ```text
//! .exe
//!   ↓
//! PE Loader (loader/)
//!   ↓
//! Runtime (runtime/)          [geplant]
//!   ↓
//! Win32-API (api/)            [geplant]
//!   ↓
//! Joys System API (joys-core)
//!   ↓
//! Linux Kernel
//! ```
//!
//! Status: PHASE 0/5/6 - der PE/COFF-Loader (loader/) ist als erster echter
//! Baustein implementiert und getestet. Das Ausführen einfacher PE32+-Programme
//! (PHASE 6) ist über runtime/ + api/ begonnen (x86_64 Linux). Alle anderen
//! Module sind deklariert, aber leer und ausdrücklich als TODO markiert -
//! keine Fake-Implementierung.

pub mod api;
pub mod dll;
pub mod loader;
pub mod registry;
pub mod runtime;

/// Von joys-win unterstützte PE-Architekturen.
pub mod arch {
    /// x86 (32-bit, PE32). Erkennung implementiert, Ausführung geplant.
    pub const X86: u16 = 0x014c;
    /// x86_64 (64-bit, PE32+). Primärziel (Definition of Done 0.1).
    pub const X86_64: u16 = 0x8664;
    /// ARM64EC / ARM64. Später.
    pub const ARM64: u16 = 0xaa64;
}

pub use loader::pe::error::PeParseError;
pub use loader::pe::image::PeImage;
pub use loader::pe::ImageArchitecture;
