//! Kleiner, geprüfter Byte-Reader für den PE-Parser (Little-Endian).

use super::error::PeParseError;

/// Liest Bytes mit Little-Endian-Konvention aus einem Buffer.
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Reader { data, pos: 0 }
    }

    /// Aktuelle Position.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Zurückspringen an eine absolute Position.
    pub fn seek(&mut self, pos: usize) {
        self.pos = pos;
    }

    /// Offset um `n` Bytes überspringen.
    pub fn skip(&mut self, n: usize) -> Result<(), PeParseError> {
        self.read_bytes(n).map(|_| ())
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], PeParseError> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or(PeParseError::Malformed("offset overflow"))?;
        if end > self.data.len() {
            return Err(PeParseError::NotEnoughData {
                offset: self.pos,
                needed: n,
                size: self.data.len(),
            });
        }
        let slice = &self.data[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    pub fn read_u8(&mut self) -> Result<u8, PeParseError> {
        Ok(self.read_bytes(1)?[0])
    }

    pub fn read_u16(&mut self) -> Result<u16, PeParseError> {
        let b = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn read_u32(&mut self) -> Result<u32, PeParseError> {
        let b = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn read_u64(&mut self) -> Result<u64, PeParseError> {
        let b = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    /// Liest `n` NUL-terminierte Bytes ab der aktuellen Position und gibt den
    /// String ohne Terminator zurück (zusätzlich werden nachfolgende Daten
    /// bis zur nächsten 2-Byte-Grenze übersprungen, um Feldgrenzen zu wahren).
    pub fn read_cstring_max(&mut self, n: usize) -> Result<&'a [u8], PeParseError> {
        let start = self.pos;
        let end = std::cmp::min(start + n, self.data.len());
        let slice = &self.data[start..end];
        let term = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
        self.pos = end;
        Ok(&slice[..term])
    }

    /// Rohen Zugriff auf den Rest ab aktueller Position.
    pub fn rest(&self) -> &'a [u8] {
        &self.data[self.pos..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_le_u16() {
        let mut r = Reader::new(&[0x4d, 0x5a]);
        assert_eq!(r.read_u16().unwrap(), 0x5a4d);
    }

    #[test]
    fn reads_le_u32() {
        let mut r = Reader::new(&[0x78, 0x56, 0x34, 0x12]);
        assert_eq!(r.read_u32().unwrap(), 0x12345678);
    }

    #[test]
    fn reads_le_u64() {
        let mut r = Reader::new(&[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(r.read_u64().unwrap(), 0x0807060504030201);
    }

    #[test]
    fn not_enough_data() {
        let mut r = Reader::new(&[1, 2, 3]);
        let err = r.read_u32().unwrap_err();
        assert!(matches!(err, PeParseError::NotEnoughData { .. }));
    }

    #[test]
    fn cstring_stops_at_nul() {
        let mut r = Reader::new(b"kernel32.dll\0xyz");
        let s = r.read_cstring_max(64).unwrap();
        assert_eq!(s, b"kernel32.dll");
    }

    #[test]
    fn seek_and_skip() {
        let mut r = Reader::new(&[1, 2, 3, 4, 5]);
        r.skip(2).unwrap();
        assert_eq!(r.pos(), 2);
        r.seek(4);
        assert_eq!(r.read_u8().unwrap(), 5);
    }
}
