//! joys-core: zentrale System-API von Joys OS.
//!
//! Bietet die definierten Schnittstellen, über die andere Komponenten
//! (u. a. `joys-win`) auf Systemfunktionen zugreifen.
//!
//! Status: PHASE 3 – system, hardware, processes, files, network, user sind
//! mit echten, getesteten Implementierungen ausgebaut.
//!
//! Einige Funktionen nutzen gezielt `unsafe` für libc-Aufrufe (mit
//! `// SAFETY:`-Kommentaren dokumentiert); alles andere ist safe Rust.

pub mod files;
pub mod hardware;
pub mod network;
pub mod processes;
pub mod system;
pub mod update;
pub mod user;

pub const JOYS_VERSION: &str = env!("CARGO_PKG_VERSION");
