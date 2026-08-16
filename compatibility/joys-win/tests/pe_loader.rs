//! Integrationstests für den PE/COFF-Loader.
//!
//! Zwei Arten:
//! 1. Synthetisches Minimal-PE32+ (auf allen Plattformen lauffähig).
//! 2. Echte Windows-Systemdateien (nur auf Windows-Hosts, z. B. beim
//!    lokalen Entwickeln; CI auf Linux nutzt den synthetischen Test).

use joys_win::loader::pe::ImageArchitecture;
use joys_win::PeImage;
use joys_win::PeParseError;

/// Baut ein minimales, strukturell gültiges PE32+ (x86_64) mit einer
/// `.text`-Section und Entry-Point RVA 0x1000 auf.
fn build_minimal_pe32plus() -> Vec<u8> {
    let mut buf = Vec::new();

    // DOS-Header (64 Bytes).
    buf.extend_from_slice(b"MZ");
    buf.extend_from_slice(&[0u8; 58]);
    buf.extend_from_slice(&0x80u32.to_le_bytes()); // e_lfanew
    while buf.len() < 0x80 {
        buf.push(0);
    }

    // PE-Signatur.
    buf.extend_from_slice(b"PE\0\0");

    // COFF-File-Header (20 Bytes).
    buf.extend_from_slice(&0x8664u16.to_le_bytes()); // machine: AMD64
    buf.extend_from_slice(&1u16.to_le_bytes()); // number of sections
    buf.extend_from_slice(&0u32.to_le_bytes()); // time/date
    buf.extend_from_slice(&0u32.to_le_bytes()); // ptr symbol table
    buf.extend_from_slice(&0u32.to_le_bytes()); // number of symbols
    buf.extend_from_slice(&240u16.to_le_bytes()); // size of optional header
    buf.extend_from_slice(&0x0022u16.to_le_bytes()); // EXECUTABLE_IMAGE|LARGE_ADDRESS_AWARE

    // Optional-Header PE32+ (240 Bytes).
    buf.extend_from_slice(&0x020bu16.to_le_bytes()); // magic PE32+
    buf.push(0); // linker major
    buf.push(0); // linker minor
    buf.extend_from_slice(&0x1000u32.to_le_bytes()); // size of code
    buf.extend_from_slice(&0u32.to_le_bytes()); // init data
    buf.extend_from_slice(&0u32.to_le_bytes()); // uninit data
    buf.extend_from_slice(&0x1000u32.to_le_bytes()); // address of entry point
    buf.extend_from_slice(&0x1000u32.to_le_bytes()); // base of code
    buf.extend_from_slice(&0x0000_0140_0000_0000u64.to_le_bytes()); // image base
    buf.extend_from_slice(&0x1000u32.to_le_bytes()); // section alignment
    buf.extend_from_slice(&0x200u32.to_le_bytes()); // file alignment
    buf.extend_from_slice(&6u16.to_le_bytes()); // os major
    buf.extend_from_slice(&0u16.to_le_bytes()); // os minor
    buf.extend_from_slice(&0u16.to_le_bytes()); // image major
    buf.extend_from_slice(&0u16.to_le_bytes()); // image minor
    buf.extend_from_slice(&6u16.to_le_bytes()); // subsystem major
    buf.extend_from_slice(&0u16.to_le_bytes()); // subsystem minor
    buf.extend_from_slice(&0u32.to_le_bytes()); // win32 version
    buf.extend_from_slice(&0x2000u32.to_le_bytes()); // size of image
    buf.extend_from_slice(&0x200u32.to_le_bytes()); // size of headers
    buf.extend_from_slice(&0u32.to_le_bytes()); // checksum
    buf.extend_from_slice(&3u16.to_le_bytes()); // subsystem: CUI
    buf.extend_from_slice(&0x8160u16.to_le_bytes()); // dll characteristics
    buf.extend_from_slice(&0x10_0000u64.to_le_bytes()); // stack reserve
    buf.extend_from_slice(&0x1000u64.to_le_bytes()); // stack commit
    buf.extend_from_slice(&0x10_0000u64.to_le_bytes()); // heap reserve
    buf.extend_from_slice(&0x1000u64.to_le_bytes()); // heap commit
    buf.extend_from_slice(&0u32.to_le_bytes()); // loader flags
    buf.extend_from_slice(&16u32.to_le_bytes()); // number of rva and sizes
    for _ in 0..16 {
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
    }
    debug_assert_eq!(buf.len(), 0x80 + 4 + 20 + 240);

    // Section-Header `.text` (40 Bytes).
    buf.extend_from_slice(b".text\0\0\0");
    buf.extend_from_slice(&0x1000u32.to_le_bytes()); // virtual size
    buf.extend_from_slice(&0x1000u32.to_le_bytes()); // virtual address
    buf.extend_from_slice(&0x200u32.to_le_bytes()); // size of raw data
    buf.extend_from_slice(&0x200u32.to_le_bytes()); // ptr to raw data
    buf.extend_from_slice(&0u32.to_le_bytes()); // ptr relocations
    buf.extend_from_slice(&0u32.to_le_bytes()); // ptr linenumbers
    buf.extend_from_slice(&0u16.to_le_bytes()); // #relocations
    buf.extend_from_slice(&0u16.to_le_bytes()); // #linenumbers
    buf.extend_from_slice(&0x6000_0020u32.to_le_bytes()); // CODE|EXECUTE|READ

    // Section-Daten ab 0x200.
    while buf.len() < 0x200 {
        buf.push(0);
    }
    buf.extend_from_slice(&[0xC3]); // `ret`

    buf
}

#[test]
fn parses_minimal_pe32plus() {
    let data = build_minimal_pe32plus();
    let img = PeImage::parse(&data).unwrap();
    assert_eq!(img.architecture(), ImageArchitecture::X86_64);
    assert_eq!(img.architecture_name(), "x86_64");
    assert!(img.is_exe());
    assert!(!img.is_dll());
    assert_eq!(img.entry_point().rva, 0x1000);
    assert_eq!(img.entry_point().address, 0x0000_0140_0000_1000);
    assert_eq!(img.optional_header().image_base(), 0x0000_0140_0000_0000);
    assert_eq!(img.sections().len(), 1);
    assert_eq!(img.sections()[0].name_str(), ".text");
    assert_eq!(img.sections()[0].virtual_address, 0x1000);
    assert_eq!(img.subsystem(), 3);
    // Keine Import-/Export-/Relocation-Tabellen vorhanden.
    assert!(img.imports().unwrap().is_empty());
    assert!(img.exports().unwrap().is_none());
    assert!(img.relocations().unwrap().is_empty());
}

#[test]
fn rejects_non_pe_data() {
    let mut data = b"Das ist kein PE-File, sondern ganz normaler Text.".to_vec();
    // Auf mindestens DOS-Header-Größe bringen, damit die Signaturprüfung greift.
    data.resize(128, b'x');
    let err = PeImage::parse(&data).unwrap_err();
    assert_eq!(err, PeParseError::InvalidDosSignature);
}

#[test]
fn rejects_truncated_dos_header() {
    let err = PeImage::parse(&[b'M', b'Z', 0x00]).unwrap_err();
    assert!(matches!(err, PeParseError::NotEnoughData { .. }));
}

#[test]
fn rejects_missing_pe_signature() {
    let mut data = build_minimal_pe32plus();
    // e_lfanew auf gültige Stelle, aber Signatur zerstören.
    let e_lfanew = u32::from_le_bytes([data[0x3c], data[0x3d], data[0x3e], data[0x3f]]) as usize;
    data[e_lfanew] = b'X';
    let err = PeImage::parse(&data).unwrap_err();
    assert_eq!(err, PeParseError::InvalidPeSignature);
}

#[test]
fn rva_to_offset_roundtrip() {
    let data = build_minimal_pe32plus();
    let img = PeImage::parse(&data).unwrap();
    // .text beginnt bei RVA 0x1000 und Raw-Offset 0x200.
    assert_eq!(img.rva_to_offset(0x1000), Some(0x200));
    assert_eq!(img.rva_to_offset(0x1010), Some(0x210));
    // Header-Bereich.
    assert_eq!(img.rva_to_offset(0x100), Some(0x100));
    // Außerhalb.
    assert_eq!(img.rva_to_offset(0x3000), None);
}

#[cfg(target_os = "windows")]
mod real_windows_binaries {
    use super::*;

    fn read_system(path: &str) -> Vec<u8> {
        let full = format!("C:\\Windows\\System32\\{path}");
        std::fs::read(&full).unwrap_or_else(|e| panic!("read {full}: {e}"))
    }

    #[test]
    fn parses_kernel32_dll() {
        let img = PeImage::parse(&read_system("kernel32.dll")).unwrap();
        assert!(img.is_dll());
        assert_eq!(img.architecture(), ImageArchitecture::X86_64);
        assert_eq!(img.architecture().pointer_size(), Some(8));
        // kernel32 exportiert zahlreiche Funktionen.
        let exports = img.exports().unwrap().expect("kernel32 exports");
        assert!(
            exports.exports.len() > 100,
            "exports = {}",
            exports.exports.len()
        );
        // Kernel32 importiert aus ntdll.
        let imports = img.imports().unwrap();
        assert!(!imports.is_empty());
    }

    #[test]
    fn parses_notepad_exe() {
        let img = PeImage::parse(&read_system("notepad.exe")).unwrap();
        assert!(img.is_exe());
        assert_eq!(img.architecture(), ImageArchitecture::X86_64);
        let ep = img.entry_point();
        assert!(!ep.is_none(), "notepad must have an entry point");
        assert!(ep.rva > 0);
    }

    #[test]
    fn parses_ntdll_exports() {
        let img = PeImage::parse(&read_system("ntdll.dll")).unwrap();
        let exports = img.exports().unwrap().expect("ntdll exports");
        let has_rtl = exports.exports.iter().any(|e| e.name.starts_with("Rtl"));
        assert!(has_rtl, "ntdll should export Rtl* functions");
    }
}
