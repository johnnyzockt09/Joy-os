//! Export-Tabelle (IMAGE_EXPORT_DIRECTORY).

use crate::loader::pe::error::PeParseError;
use crate::loader::pe::reader::Reader;

/// Ein exportiertes Symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    /// Name des Exports (leer bei reinen Ordinal-Exports).
    pub name: String,
    /// Export-Ordinal (Base + Index).
    pub ordinal: u32,
    /// RVA des Export-Ziels.
    pub address_rva: u32,
    /// Forwarder-Ziel (z. B. "NTDLL.RtlAllocateHeap"), wenn gesetzt.
    pub forwarder: Option<String>,
}

/// Die vollständig ausgewertete Export-Tabelle eines PE-Images.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportTable {
    pub dll_name: String,
    pub base: u32,
    pub exports: Vec<Export>,
}

/// Parst die Export-Tabelle ab RVA `export_rva`.
pub struct ExportTableParser<'a> {
    data: &'a [u8],
    rva_to_offset: &'a dyn Fn(u32) -> Option<usize>,
}

impl<'a> ExportTableParser<'a> {
    pub fn new(data: &'a [u8], rva_to_offset: &'a dyn Fn(u32) -> Option<usize>) -> Self {
        ExportTableParser {
            data,
            rva_to_offset,
        }
    }

    /// Löst eine RVA in einen Datei-Offset auf (über das Image-Mapping).
    fn off(&self, rva: u32) -> Option<usize> {
        (self.rva_to_offset)(rva)
    }

    pub fn parse(&self, export_rva: u32, export_size: u32) -> Result<ExportTable, PeParseError> {
        let offset = self
            .off(export_rva)
            .ok_or(PeParseError::InvalidExportTable)?;
        if offset + 40 > self.data.len() {
            return Err(PeParseError::InvalidExportTable);
        }

        let mut r = Reader::new(&self.data[offset..]);
        let _characteristics = r.read_u32()?;
        let _time_date_stamp = r.read_u32()?;
        let _major_version = r.read_u16()?;
        let _minor_version = r.read_u16()?;
        let name_rva = r.read_u32()?;
        let base = r.read_u32()?;
        let number_of_functions = r.read_u32()?;
        let number_of_names = r.read_u32()?;
        let address_of_functions = r.read_u32()?;
        let address_of_names = r.read_u32()?;
        let address_of_name_ordinals = r.read_u32()?;

        let dll_name = self
            .read_cstring(name_rva)
            .ok_or(PeParseError::InvalidExportTable)?;

        let functions_off = self
            .off(address_of_functions)
            .ok_or(PeParseError::InvalidExportTable)?;
        let names_off = self
            .off(address_of_names)
            .ok_or(PeParseError::InvalidExportTable)?;
        let ordinals_off = self
            .off(address_of_name_ordinals)
            .ok_or(PeParseError::InvalidExportTable)?;

        let function_count = number_of_functions as usize;
        let name_count = number_of_names as usize;

        if functions_off
            .checked_add(function_count * 4)
            .is_none_or(|end| end > self.data.len())
        {
            return Err(PeParseError::InvalidExportTable);
        }
        if names_off
            .checked_add(name_count * 4)
            .is_none_or(|end| end > self.data.len())
        {
            return Err(PeParseError::InvalidExportTable);
        }
        if ordinals_off
            .checked_add(name_count * 2)
            .is_none_or(|end| end > self.data.len())
        {
            return Err(PeParseError::InvalidExportTable);
        }

        // Funktionen mit Namen versehen (Ordinal = base + index).
        let mut exports: Vec<Option<Export>> = (0..function_count)
            .map(|i| {
                let addr_rva = read_le_u32(self.data, functions_off + i * 4);
                Some(Export {
                    name: String::new(),
                    ordinal: base + i as u32,
                    address_rva: addr_rva,
                    forwarder: None,
                })
            })
            .collect();

        for i in 0..name_count {
            let name_rva_i = read_le_u32(self.data, names_off + i * 4);
            let name = self
                .read_cstring(name_rva_i)
                .ok_or(PeParseError::InvalidExportTable)?;
            let ordinal_idx = read_le_u16(self.data, ordinals_off + i * 2) as usize;
            if ordinal_idx >= function_count {
                return Err(PeParseError::InvalidExportTable);
            }
            if let Some(e) = exports.get_mut(ordinal_idx) {
                let e = e.as_mut().unwrap();
                e.name = name;
            }
        }

        // Forwarder auflösen: Eine Export-Adresse, die in den Bereich der
        // Export-Directory-Daten (RVA bis RVA+size) zeigt, ist ein
        // Forwarder-String (z. B. "NTDLL.RtlAllocateHeap").
        let export_dir_range = export_rva..export_rva.wrapping_add(export_size);
        let exports = exports
            .into_iter()
            .flatten()
            .map(|mut e| {
                if export_dir_range.contains(&e.address_rva) {
                    if let Some(fwd) = self.read_cstring(e.address_rva) {
                        e.forwarder = Some(fwd);
                    }
                }
                e
            })
            .collect();

        Ok(ExportTable {
            dll_name,
            base,
            exports,
        })
    }

    fn read_cstring(&self, rva: u32) -> Option<String> {
        let offset = self.off(rva)?;
        let slice = self.data.get(offset..)?;
        let term = slice.iter().position(|&b| b == 0)?;
        Some(String::from_utf8_lossy(&slice[..term]).into_owned())
    }
}

fn read_le_u32(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
}

fn read_le_u16(data: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([data[off], data[off + 1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Baut eine Export-Tabelle auf (Offset -> RVA: rva = 0x1000 + off).
    /// Der Funktions-Code liegt bewusst AUSSERHALB des Export-Directory-Bereichs
    /// (RVA 0x1300), der Forwarder-String INNERHALB (RVA 0x1047).
    /// 0x00: IMAGE_EXPORT_DIRECTORY (40 Bytes)
    /// 0x28: "test.dll\0"
    /// 0x31: AddressOfFunctions[2] (RVA 0x1300 Code, RVA 0x1047 Forwarder)
    /// 0x39: AddressOfNames[1] (RVA 0x103F)
    /// 0x3D: AddressOfNameOrdinals[1] (0x0000)
    /// 0x3F: "MyFunc\0"
    /// 0x46: (Reserve)
    /// 0x47: "OTHER.Sym\0" (Forwarder-String)
    fn build_export_table() -> Vec<u8> {
        const BASE: usize = 0x1000;
        let rva_of = |off: usize| -> u32 { (BASE + off) as u32 };

        let mut buf = Vec::new();
        buf.extend_from_slice(&[0u8; 40]); // dir (0x00)
        let dll_off = buf.len(); // 0x28
        buf.extend_from_slice(b"test.dll\0");
        let fns_off = buf.len(); // 0x31
        buf.extend_from_slice(&0x1300u32.to_le_bytes()); // Code-Ziel (außerhalb)
        buf.extend_from_slice(&rva_of(0x47).to_le_bytes()); // Forwarder-String
        let names_off = buf.len(); // 0x39
        buf.extend_from_slice(&rva_of(0x3f).to_le_bytes());
        let ordinals_off = buf.len(); // 0x3D
        buf.extend_from_slice(&0x0000u16.to_le_bytes());
        buf.extend_from_slice(b"MyFunc\0"); // 0x3F
        buf.extend_from_slice(&[0u8; 1]); // 0x46 Reserve
        buf.extend_from_slice(b"OTHER.Sym\0"); // 0x47

        // Directory-Felder füllen (IMAGE_EXPORT_DIRECTORY, Feldreihenfolge):
        // Characteristics(0) TimeDateStamp(4) Major(8) Minor(10)
        // Name(12) Base(16) NumberOfFunctions(20) NumberOfNames(24)
        // AddressOfFunctions(28) AddressOfNames(32) AddressOfNameOrdinals(36)
        buf[12..16].copy_from_slice(&rva_of(dll_off).to_le_bytes());
        buf[16..20].copy_from_slice(&1u32.to_le_bytes()); // Base
        buf[20..24].copy_from_slice(&2u32.to_le_bytes()); // NumberOfFunctions
        buf[24..28].copy_from_slice(&1u32.to_le_bytes()); // NumberOfNames
        buf[28..32].copy_from_slice(&rva_of(fns_off).to_le_bytes());
        buf[32..36].copy_from_slice(&rva_of(names_off).to_le_bytes());
        buf[36..40].copy_from_slice(&rva_of(ordinals_off).to_le_bytes());
        buf
    }

    fn rva_of(rva: u32) -> Option<usize> {
        if rva >= 0x1000 {
            Some((rva - 0x1000) as usize)
        } else {
            None
        }
    }

    #[test]
    fn parses_exports_and_forwarders() {
        let buf = build_export_table();
        let size = buf.len() as u32;
        let parser = ExportTableParser::new(&buf, &rva_of);
        let table = parser.parse(0x1000, size).unwrap();
        assert_eq!(table.dll_name, "test.dll");
        assert_eq!(table.base, 1);
        assert_eq!(table.exports.len(), 2);

        let named = table.exports.iter().find(|e| !e.name.is_empty()).unwrap();
        assert_eq!(named.name, "MyFunc");
        assert_eq!(named.ordinal, 1);
        assert!(named.forwarder.is_none());

        let fwd = table
            .exports
            .iter()
            .find(|e| e.forwarder.is_some())
            .unwrap();
        assert_eq!(fwd.forwarder.as_deref(), Some("OTHER.Sym"));
        assert_eq!(fwd.ordinal, 2);
    }
}
