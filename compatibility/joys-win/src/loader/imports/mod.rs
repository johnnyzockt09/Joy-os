//! Import-Tabelle (IMAGE_IMPORT_DESCRIPTOR, IMAGE_THUNK_DATA).

use crate::loader::pe::error::PeParseError;
use crate::loader::pe::reader::Reader;

pub const SIZE_OF_IMPORT_DESCRIPTOR: usize = 20;

/// Ein Import einer DLL: DLL-Name + Liste der Import-Symbole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDescriptor {
    /// Name der DLL (z. B. "kernel32.dll").
    pub dll_name: String,
    /// OriginalFirstThunk (RVA der Import-Name-Tabelle).
    pub original_first_thunk: u32,
    /// FirstThunk (RVA der IAT).
    pub first_thunk: u32,
    /// Importierte Symbole.
    pub imports: Vec<Import>,
}

/// Ein importiertes Symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Import {
    /// Import per Name (Hint + Funktionsname).
    ByName { hint: u16, name: String },
    /// Import per Ordnungsnummer.
    ByOrdinal { ordinal: u16 },
}

/// Parst die Import-Tabelle ab der RVA `import_rva` (Datei-Offset wird von
/// der `rva_to_offset`-Funktion des Images aufgelöst).
///
/// `read` liefert die aufzulösenden RA/RVA-Werte der Thunk-Entries.
/// Für PE32+ sind Thunks 8 Byte, für PE32 4 Byte groß.
pub struct ImportTableParser<'a> {
    data: &'a [u8],
    rva_to_offset: &'a dyn Fn(u32) -> Option<usize>,
    pointer_size: usize,
}

impl<'a> ImportTableParser<'a> {
    pub fn new(
        data: &'a [u8],
        rva_to_offset: &'a dyn Fn(u32) -> Option<usize>,
        pointer_size: usize,
    ) -> Self {
        ImportTableParser {
            data,
            rva_to_offset,
            pointer_size,
        }
    }

    /// Löst eine RVA in einen Datei-Offset auf (über das Image-Mapping).
    fn off(&self, rva: u32) -> Option<usize> {
        (self.rva_to_offset)(rva)
    }

    /// Parst alle Import-Deskriptoren. Null-Deskriptor beendet die Tabelle.
    pub fn parse(&self, import_rva: u32) -> Result<Vec<ImportDescriptor>, PeParseError> {
        let mut offset = self
            .off(import_rva)
            .ok_or(PeParseError::InvalidImportTable)?;
        let mut result = Vec::new();

        loop {
            let mut r = Reader::new(&self.data[offset..]);
            let original_first_thunk = r.read_u32()?;
            let _time_date_stamp = r.read_u32()?;
            let _forwarder_chain = r.read_u32()?;
            let name_rva = r.read_u32()?;
            let first_thunk = r.read_u32()?;

            // Null-Deskriptor = Ende.
            if original_first_thunk == 0 && name_rva == 0 && first_thunk == 0 {
                break;
            }

            let name_offset = self.off(name_rva).ok_or(PeParseError::InvalidImportTable)?;
            let dll_name =
                cstring_at(self.data, name_offset, 256).ok_or(PeParseError::InvalidImportTable)?;

            let thunk_rva = if original_first_thunk != 0 {
                original_first_thunk
            } else {
                first_thunk
            };
            let imports = self.parse_thunks(thunk_rva)?;

            result.push(ImportDescriptor {
                dll_name,
                original_first_thunk,
                first_thunk,
                imports,
            });

            offset = offset
                .checked_add(SIZE_OF_IMPORT_DESCRIPTOR)
                .ok_or(PeParseError::InvalidImportTable)?;
        }

        Ok(result)
    }

    fn parse_thunks(&self, thunk_rva: u32) -> Result<Vec<Import>, PeParseError> {
        let mut thunk_offset = self
            .off(thunk_rva)
            .ok_or(PeParseError::InvalidImportTable)?;
        let mut imports = Vec::new();

        loop {
            let thunk = read_thunk(self.data, thunk_offset, self.pointer_size)
                .ok_or(PeParseError::InvalidImportTable)?;

            if thunk == 0 {
                break;
            }

            let ordinal_flag = if self.pointer_size == 8 {
                0x8000_0000_0000_0000u64
            } else {
                0x8000_0000u64
            };

            if thunk & ordinal_flag != 0 {
                let ordinal = (thunk & 0xffff) as u16;
                imports.push(Import::ByOrdinal { ordinal });
            } else {
                let name_rva = thunk as u32;
                let name_offset = self.off(name_rva).ok_or(PeParseError::InvalidImportTable)?;
                if name_offset + 2 > self.data.len() {
                    return Err(PeParseError::InvalidImportTable);
                }
                let hint = u16::from_le_bytes([self.data[name_offset], self.data[name_offset + 1]]);
                let name = cstring_at(self.data, name_offset + 2, 255)
                    .ok_or(PeParseError::InvalidImportTable)?;
                imports.push(Import::ByName { hint, name });
            }

            thunk_offset = thunk_offset
                .checked_add(self.pointer_size)
                .ok_or(PeParseError::InvalidImportTable)?;
        }

        Ok(imports)
    }
}

fn read_thunk(data: &[u8], offset: usize, size: usize) -> Option<u64> {
    match size {
        4 => {
            let b = data.get(offset..offset + 4)?;
            Some(u64::from(u32::from_le_bytes([b[0], b[1], b[2], b[3]])))
        }
        8 => {
            let b = data.get(offset..offset + 8)?;
            Some(u64::from_le_bytes([
                b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
            ]))
        }
        _ => None,
    }
}

fn cstring_at(data: &[u8], offset: usize, max: usize) -> Option<String> {
    let end = std::cmp::min(offset + max, data.len());
    let slice = data.get(offset..end)?;
    let term = slice.iter().position(|&b| b == 0)?;
    Some(String::from_utf8_lossy(&slice[..term]).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Baut einen korrekt kontiguierten Import-Bereich für PE32+ auf.
    ///
    /// Layout (RVA = 0x1000 + Offset):
    /// 0x00: Import-Descriptor (20 Bytes)
    /// 0x14: Null-Descriptor (Terminator)
    /// 0x28: "kernel32.dll\0"
    /// 0x35: IMAGE_IMPORT_BY_NAME (Hint 0 + "ExitProcess\0")
    /// 0x43: Thunk-Liste (8-Byte: RVA->ByName, 0)
    fn build_import_table() -> Vec<u8> {
        const BASE: usize = 0x1000;
        let rva_of = |off: usize| -> u32 { (BASE + off) as u32 };

        let mut buf = Vec::new();
        buf.extend_from_slice(&[0u8; SIZE_OF_IMPORT_DESCRIPTOR]); // 0x00
        buf.extend_from_slice(&[0u8; SIZE_OF_IMPORT_DESCRIPTOR]); // 0x14 Terminator
        let dll_name_off = buf.len(); // 0x28
        buf.extend_from_slice(b"kernel32.dll\0");
        let byname_off = buf.len(); // 0x35
        buf.extend_from_slice(&0x0000u16.to_le_bytes()); // Hint
        buf.extend_from_slice(b"ExitProcess\0");
        let thunk_off = buf.len(); // 0x43
        buf.extend_from_slice(&(rva_of(byname_off) as u64).to_le_bytes());
        buf.extend_from_slice(&0u64.to_le_bytes()); // Terminator

        let thunk_rva = rva_of(thunk_off);
        buf[0..4].copy_from_slice(&thunk_rva.to_le_bytes()); // OriginalFirstThunk
        buf[12..16].copy_from_slice(&rva_of(dll_name_off).to_le_bytes()); // Name
        buf[16..20].copy_from_slice(&thunk_rva.to_le_bytes()); // FirstThunk
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
    fn parses_import_by_name() {
        let buf = build_import_table();
        let parser = ImportTableParser::new(&buf, &rva_of, 8);
        let imports = parser.parse(0x1000).unwrap();
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].dll_name, "kernel32.dll");
        assert_eq!(imports[0].imports.len(), 1);
        match &imports[0].imports[0] {
            Import::ByName { hint, name } => {
                assert_eq!(*hint, 0);
                assert_eq!(name, "ExitProcess");
            }
            _ => panic!("expected by-name import"),
        }
    }

    #[test]
    fn stops_at_null_descriptor() {
        let buf = build_import_table();
        let parser = ImportTableParser::new(&buf, &rva_of, 8);
        let imports = parser.parse(0x1000).unwrap();
        assert_eq!(imports.len(), 1);
    }

    #[test]
    fn parses_import_by_ordinal() {
        let thunk: u64 = 0x8000_0000_0000_0007;
        let mut buf = Vec::new();
        buf.extend_from_slice(&0u32.to_le_bytes()); // 0x00
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0x1028u32.to_le_bytes()); // Name
        buf.extend_from_slice(&0x1030u32.to_le_bytes()); // FirstThunk
        buf.extend_from_slice(&[0u8; SIZE_OF_IMPORT_DESCRIPTOR]); // 0x14 Terminator
        buf.extend_from_slice(b"foo.dll\0"); // 0x28
        buf.extend_from_slice(&thunk.to_le_bytes()); // 0x30
        buf.extend_from_slice(&0u64.to_le_bytes());

        let parser = ImportTableParser::new(&buf, &rva_of, 8);
        let imports = parser.parse(0x1000).unwrap();
        assert_eq!(imports[0].dll_name, "foo.dll");
        assert_eq!(imports[0].imports.len(), 1);
        match &imports[0].imports[0] {
            Import::ByOrdinal { ordinal } => assert_eq!(*ordinal, 7),
            _ => panic!("expected by-ordinal import"),
        }
    }
}
