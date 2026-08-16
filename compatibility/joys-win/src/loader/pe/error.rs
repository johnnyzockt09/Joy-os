//! Fehler- und Grundtypen für den PE-Parser.

use std::fmt;

/// Fehler beim Parsen einer PE-Datei.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeParseError {
    /// Nicht genug Bytes im Buffer.
    NotEnoughData {
        /// Offset, an dem gelesen wurde.
        offset: usize,
        /// Benötigte Bytes.
        needed: usize,
        /// Vorhandene Bytes.
        size: usize,
    },
    /// Ungültiger DOS-Header (kein 'MZ').
    InvalidDosSignature,
    /// Ungültige PE-Signatur (kein 'PE\\0\\0').
    InvalidPeSignature,
    /// Nicht unterstützte Optional-Header-Magic (weder PE32 noch PE32+).
    UnsupportedOptionalHeaderMagic(u16),
    /// Nicht unterstützte Machine-Architektur.
    UnsupportedMachine(u16),
    /// Ungültige Anzahl von Sections.
    InvalidSectionCount(u32),
    /// Kaputte Import-Tabelle.
    InvalidImportTable,
    /// Kaputte Export-Tabelle.
    InvalidExportTable,
    /// Kaputte Relocation-Tabelle.
    InvalidRelocations,
    /// Unerwartete Datenstruktur.
    Malformed(&'static str),
}

impl fmt::Display for PeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeParseError::NotEnoughData {
                offset,
                needed,
                size,
            } => write!(
                f,
                "PE: not enough data at offset {offset}: need {needed} bytes, have {size}"
            ),
            PeParseError::InvalidDosSignature => write!(f, "PE: invalid DOS signature (no 'MZ')"),
            PeParseError::InvalidPeSignature => write!(f, "PE: invalid PE signature"),
            PeParseError::UnsupportedOptionalHeaderMagic(m) => {
                write!(f, "PE: unsupported optional header magic 0x{m:04x}")
            }
            PeParseError::UnsupportedMachine(m) => {
                write!(f, "PE: unsupported machine 0x{m:04x}")
            }
            PeParseError::InvalidSectionCount(n) => {
                write!(f, "PE: invalid section count {n}")
            }
            PeParseError::InvalidImportTable => write!(f, "PE: invalid import table"),
            PeParseError::InvalidExportTable => write!(f, "PE: invalid export table"),
            PeParseError::InvalidRelocations => write!(f, "PE: invalid relocation table"),
            PeParseError::Malformed(m) => write!(f, "PE: malformed data: {m}"),
        }
    }
}

impl std::error::Error for PeParseError {}
