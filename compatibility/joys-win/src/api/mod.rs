//! API-Modul von joys-win: Builtin-Implementierungen der Windows-DLLs.
//!
//! Status: kernel32 mit einer minimalen, echt funktionierenden Basis
//! (GetStdHandle, WriteFile, ExitProcess). Alles andere ist bewusst NICHT
//! implementiert - Aufrufe scheitern mit einer klaren Meldung.

pub mod kernel32;
