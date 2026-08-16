//! joys-core: zentrale System-API von Joys OS.
//!
//! Bietet die definierten Schnittstellen, über die andere Komponenten
//! (u. a. `joys-win`) auf Systemfunktionen zugreifen. `joys-win` darf nur
//! über diese Schnittstellen auf Joys Core zugreifen.
//!
//! Status: PHASE 0/3 - Grundgerüst. Konkrete Funktionen folgen in PHASE 3.

#![forbid(unsafe_code)]

pub mod system;

pub const JOYS_VERSION: &str = env!("CARGO_PKG_VERSION");
