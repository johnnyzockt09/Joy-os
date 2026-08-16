//! Basis-Relocation-Tabelle (IMAGE_BASE_RELOCATION).

use crate::loader::pe::error::PeParseError;
use crate::loader::pe::reader::Reader;

/// Relocation-Typen (IMAGE_REL_BASED_*).
pub mod types {
    pub const ABSOLUTE: u16 = 0;
    pub const HIGH: u16 = 1;
    pub const LOW: u16 = 2;
    pub const HIGHLOW: u16 = 3;
    pub const HIGHADJ: u16 = 4;
    pub const MIPS_JMPADDR: u16 = 5;
    pub const ARM_MOV32: u16 = 5;
    pub const RISCV_HIGH20: u16 = 5;
    pub const THUMB_MOV32: u16 = 7;
    pub const RISCV_LOW12I: u16 = 7;
    pub const MIPS_JMPADDR16: u16 = 9;
    pub const DIR64: u16 = 10;
}

/// Ein einzelner Relocation-Eintrag (Typ + 12-bit-Offset im Block).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelocationEntry {
    pub kind: u16,
    pub offset_in_block: u16,
}

/// Ein Relocation-Block: alle Einträge gelten für die Seite `page_rva`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelocationBlock {
    pub page_rva: u32,
    pub entries: Vec<RelocationEntry>,
}

/// Parst die Basis-Relocation-Tabelle ab RVA `reloc_rva`.
pub fn parse_relocations(
    data: &[u8],
    rva_to_offset: &dyn Fn(u32) -> Option<usize>,
    reloc_rva: u32,
    size: u32,
) -> Result<Vec<RelocationBlock>, PeParseError> {
    let mut offset = rva_to_offset(reloc_rva).ok_or(PeParseError::InvalidRelocations)?;
    let start = offset;
    let end = start.saturating_add(size as usize);

    let mut blocks = Vec::new();
    while offset + 8 <= end && offset + 8 <= data.len() {
        let mut r = Reader::new(&data[offset..]);
        let page_rva = r.read_u32()?;
        let block_size = r.read_u32()?;

        // Null-Block = Ende der Tabelle.
        if page_rva == 0 && block_size == 0 {
            break;
        }
        if block_size < 8 {
            return Err(PeParseError::InvalidRelocations);
        }

        let entry_bytes = (block_size - 8) as usize;
        if offset + block_size as usize > end || offset + block_size as usize > data.len() {
            return Err(PeParseError::InvalidRelocations);
        }

        let mut entries = Vec::with_capacity(entry_bytes / 2);
        for _ in 0..entry_bytes / 2 {
            let raw = r.read_u16()?;
            entries.push(RelocationEntry {
                kind: raw >> 12,
                offset_in_block: raw & 0x0fff,
            });
        }

        blocks.push(RelocationBlock { page_rva, entries });
        offset += block_size as usize;
    }

    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rva_to_offset(rva: u32) -> Option<usize> {
        if rva >= 0x1000 {
            Some((rva - 0x1000) as usize)
        } else {
            None
        }
    }

    #[test]
    fn parses_blocks_and_types() {
        // Zwei Blöcke: Seite 0x1000 (ABSOLUTE, DIR64), Seite 0x2000 (HIGHLOW).
        let mut buf = Vec::new();
        let block1_size = 8 + 2 * 2;
        buf.extend_from_slice(&0x1000u32.to_le_bytes());
        buf.extend_from_slice(&(block1_size as u32).to_le_bytes());
        let e1: u16 = types::ABSOLUTE << 12;
        let e2: u16 = (types::DIR64 << 12) | 0x018;
        buf.extend_from_slice(&e1.to_le_bytes());
        buf.extend_from_slice(&e2.to_le_bytes());

        let block2_size = 8 + 2;
        buf.extend_from_slice(&0x2000u32.to_le_bytes());
        buf.extend_from_slice(&(block2_size as u32).to_le_bytes());
        let e3: u16 = (types::HIGHLOW << 12) | 0x0ff;
        buf.extend_from_slice(&e3.to_le_bytes());

        let blocks = parse_relocations(&buf, &rva_to_offset, 0x1000, buf.len() as u32).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].page_rva, 0x1000);
        assert_eq!(blocks[0].entries.len(), 2);
        assert_eq!(blocks[0].entries[0].kind, types::ABSOLUTE);
        assert_eq!(blocks[0].entries[1].kind, types::DIR64);
        assert_eq!(blocks[0].entries[1].offset_in_block, 0x018);
        assert_eq!(blocks[1].entries[0].kind, types::HIGHLOW);
    }

    #[test]
    fn stops_at_null_block() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        let blocks = parse_relocations(&buf, &rva_to_offset, 0x1000, buf.len() as u32).unwrap();
        assert!(blocks.is_empty());
    }

    #[test]
    fn rejects_truncated_block() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&0x1000u32.to_le_bytes());
        buf.extend_from_slice(&64u32.to_le_bytes()); // verspricht mehr als vorhanden
        let err = parse_relocations(&buf, &rva_to_offset, 0x1000, buf.len() as u32).unwrap_err();
        assert_eq!(err, PeParseError::InvalidRelocations);
    }
}
