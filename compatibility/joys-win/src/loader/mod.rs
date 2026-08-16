//! PE/COFF-Loader von joys-win.
//!
//! ```text
//! .exe/.dll
//!   ↓
//! pe/         PE-Header-Parsing (DOS, COFF, Optional, Data Directory)
//! sections/   Section-Modell + Berechtigungen
//! imports/    Import-Tabelle
//! exports/    Export-Tabelle
//! relocations/ Basis-Relocation-Blöcke
//! entrypoint/ Entry-Point-Ermittlung
//! ```
//!
//! Status: Header-, Section-, Import-, Export- und Relocation-Parsing sind
//! implementiert und getestet. Das eigentliche *Mapping* des Images in den
//! Speicher und das Ausführen folgt in PHASE 6.

pub mod entrypoint;
pub mod exports;
pub mod imports;
pub mod pe;
pub mod relocations;
pub mod sections;
