//! Section-Modell und Section-Parsing (IMAGE_SECTION_HEADER).

use crate::loader::pe::error::PeParseError;
use crate::loader::pe::reader::Reader;

pub const SIZE_OF_SECTION_HEADER: usize = 40;

/// Berechtigungs-Flags einer Section (IMAGE_SCN_*).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionCharacteristics {
    pub raw: u32,
}

impl SectionCharacteristics {
    pub const EXECUTE: u32 = 0x2000_0000;
    pub const READ: u32 = 0x4000_0000;
    pub const WRITE: u32 = 0x8000_0000;

    pub const CNT_CODE: u32 = 0x0000_0020;
    pub const CNT_INITIALIZED_DATA: u32 = 0x0000_0040;
    pub const CNT_UNINITIALIZED_DATA: u32 = 0x0000_0080;

    pub fn is_execute(&self) -> bool {
        self.raw & Self::EXECUTE != 0
    }

    pub fn is_read(&self) -> bool {
        self.raw & Self::READ != 0
    }

    pub fn is_write(&self) -> bool {
        self.raw & Self::WRITE != 0
    }

    pub fn is_code(&self) -> bool {
        self.raw & Self::CNT_CODE != 0
    }
}

/// Eine Section eines PE-Images.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub name: [u8; 8],
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub size_of_raw_data: u32,
    pub pointer_to_raw_data: u32,
    pub characteristics: SectionCharacteristics,
}

impl Section {
    /// Name als String (bis zum ersten NUL-Byte).
    pub fn name_str(&self) -> String {
        let end = self
            .name
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(self.name.len());
        String::from_utf8_lossy(&self.name[..end]).into_owned()
    }

    /// Liest eine Section aus dem Reader.
    pub fn parse(r: &mut Reader<'_>) -> Result<Self, PeParseError> {
        let name_bytes = r.read_bytes(8)?;
        let mut name = [0u8; 8];
        name.copy_from_slice(name_bytes);
        Ok(Section {
            name,
            virtual_size: r.read_u32()?,
            virtual_address: r.read_u32()?,
            size_of_raw_data: r.read_u32()?,
            pointer_to_raw_data: r.read_u32()?,
            characteristics: SectionCharacteristics {
                raw: {
                    r.skip(12)?;
                    r.read_u32()?
                },
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(
        name: &[u8; 8],
        vsize: u32,
        vaddr: u32,
        raw_size: u32,
        raw_ptr: u32,
        chars: u32,
    ) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(name);
        v.extend_from_slice(&vsize.to_le_bytes());
        v.extend_from_slice(&vaddr.to_le_bytes());
        v.extend_from_slice(&raw_size.to_le_bytes());
        v.extend_from_slice(&raw_ptr.to_le_bytes());
        v.extend_from_slice(&0u32.to_le_bytes()); // PointerToRelocations
        v.extend_from_slice(&0u32.to_le_bytes()); // PointerToLinenumbers
        v.extend_from_slice(&0u16.to_le_bytes()); // NumberOfRelocations
        v.extend_from_slice(&0u16.to_le_bytes()); // NumberOfLinenumbers
        v.extend_from_slice(&chars.to_le_bytes());
        v
    }

    #[test]
    fn parses_section() {
        let buf = header(b".text\0\0\0", 0x1000, 0x1000, 0x200, 0x400, 0x6000_0020);
        let mut r = Reader::new(&buf);
        let s = Section::parse(&mut r).unwrap();
        assert_eq!(s.name_str(), ".text");
        assert_eq!(s.virtual_size, 0x1000);
        assert_eq!(s.virtual_address, 0x1000);
        assert!(s.characteristics.is_code());
        assert!(!s.characteristics.is_write());
    }

    #[test]
    fn detects_write_permission() {
        let buf = header(b".data\0\0\0", 0x800, 0x2000, 0x100, 0x600, 0xC000_0040);
        let mut r = Reader::new(&buf);
        let s = Section::parse(&mut r).unwrap();
        assert!(s.characteristics.is_write());
        assert!(s.characteristics.is_read());
    }
}
