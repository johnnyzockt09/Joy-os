//! PE-Header-Parsing: DOS-Header, PE-Signatur, COFF-Header, Optional-Header,
//! Data Directories.

pub mod error;
pub mod image;
pub mod reader;

pub use image::PeImage;

/// 'MZ' – DOS-Signatur.
pub const DOS_MAGIC: u16 = 0x5a4d;
/// 'PE\0\0' – PE-Signatur.
pub const PE_SIGNATURE: u32 = 0x0000_4550;
/// Optional-Header-Magic für PE32 (32-bit).
pub const OPTIONAL_MAGIC_PE32: u16 = 0x010b;
/// Optional-Header-Magic für PE32+ (64-bit).
pub const OPTIONAL_MAGIC_PE32PLUS: u16 = 0x020b;

/// Von Windows definierte Architekturen (IMAGE_FILE_MACHINE_*).
pub mod machine {
    pub const UNKNOWN: u16 = 0x0000;
    pub const I386: u16 = 0x014c;
    pub const AMD64: u16 = 0x8664;
    pub const ARM64: u16 = 0xaa64;
}

/// Von Windows definierte Subsysteme (IMAGE_SUBSYSTEM_*).
pub mod subsystem {
    pub const UNKNOWN: u16 = 0;
    pub const NATIVE: u16 = 1;
    pub const WINDOWS_GUI: u16 = 2;
    pub const WINDOWS_CUI: u16 = 3;
}

/// Von Windows definierte Data-Directory-Indizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Directory {
    Export,
    Import,
    Resource,
    Exception,
    Security,
    BaseRelocation,
    Debug,
    Architecture,
    GlobalPtr,
    Tls,
    LoadConfig,
    BoundImport,
    Iat,
    DelayImport,
    ComDescriptor,
    Reserved,
}

impl Directory {
    pub fn index(&self) -> usize {
        match self {
            Directory::Export => 0,
            Directory::Import => 1,
            Directory::Resource => 2,
            Directory::Exception => 3,
            Directory::Security => 4,
            Directory::BaseRelocation => 5,
            Directory::Debug => 6,
            Directory::Architecture => 7,
            Directory::GlobalPtr => 8,
            Directory::Tls => 9,
            Directory::LoadConfig => 10,
            Directory::BoundImport => 11,
            Directory::Iat => 12,
            Directory::DelayImport => 13,
            Directory::ComDescriptor => 14,
            Directory::Reserved => 15,
        }
    }
}

/// Verwendete Zielarchitektur eines PE-Images.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageArchitecture {
    X86,
    X86_64,
    Arm64,
    Unknown(u16),
}

impl ImageArchitecture {
    pub fn from_machine(m: u16) -> Self {
        match m {
            machine::I386 => ImageArchitecture::X86,
            machine::AMD64 => ImageArchitecture::X86_64,
            machine::ARM64 => ImageArchitecture::Arm64,
            other => ImageArchitecture::Unknown(other),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ImageArchitecture::X86 => "x86",
            ImageArchitecture::X86_64 => "x86_64",
            ImageArchitecture::Arm64 => "aarch64",
            ImageArchitecture::Unknown(_) => "unknown",
        }
    }

    pub fn pointer_size(&self) -> Option<u8> {
        match self {
            ImageArchitecture::X86 => Some(4),
            ImageArchitecture::X86_64 | ImageArchitecture::Arm64 => Some(8),
            ImageArchitecture::Unknown(_) => None,
        }
    }
}

/// DOS-Header (IMAGE_DOS_HEADER).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DosHeader {
    pub e_magic: u16,
    pub e_cblp: u16,
    pub e_cp: u16,
    pub e_crlc: u16,
    pub e_cparhdr: u16,
    pub e_minalloc: u16,
    pub e_maxalloc: u16,
    pub e_ss: u16,
    pub e_sp: u16,
    pub e_csum: u16,
    pub e_ip: u16,
    pub e_cs: u16,
    pub e_lfarlc: u16,
    pub e_ovno: u16,
    pub e_res: [u16; 4],
    pub e_oemid: u16,
    pub e_oeminfo: u16,
    pub e_res2: [u16; 10],
    pub e_lfanew: u32,
}

/// COFF-File-Header (IMAGE_FILE_HEADER).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoffHeader {
    pub machine: u16,
    pub number_of_sections: u16,
    pub time_date_stamp: u32,
    pub pointer_to_symbol_table: u32,
    pub number_of_symbols: u32,
    pub size_of_optional_header: u16,
    pub characteristics: u16,
}

/// Ein Eintrag der Data-Directory-Tabelle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataDirectoryEntry {
    pub virtual_address: u32,
    pub size: u32,
}

impl DataDirectoryEntry {
    pub const EMPTY: DataDirectoryEntry = DataDirectoryEntry {
        virtual_address: 0,
        size: 0,
    };
}

/// Optional-Header, versioniert für PE32 und PE32+.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionalHeader {
    Pe32(OptionalHeader32),
    Pe32Plus(OptionalHeader64),
}

impl OptionalHeader {
    pub fn is_pe32_plus(&self) -> bool {
        matches!(self, OptionalHeader::Pe32Plus(_))
    }

    pub fn entry_point(&self) -> u32 {
        match self {
            OptionalHeader::Pe32(h) => h.address_of_entry_point,
            OptionalHeader::Pe32Plus(h) => h.address_of_entry_point,
        }
    }

    pub fn image_base(&self) -> u64 {
        match self {
            OptionalHeader::Pe32(h) => u64::from(h.image_base),
            OptionalHeader::Pe32Plus(h) => h.image_base,
        }
    }

    pub fn size_of_image(&self) -> u32 {
        match self {
            OptionalHeader::Pe32(h) => h.size_of_image,
            OptionalHeader::Pe32Plus(h) => h.size_of_image,
        }
    }

    pub fn section_alignment(&self) -> u32 {
        match self {
            OptionalHeader::Pe32(h) => h.section_alignment,
            OptionalHeader::Pe32Plus(h) => h.section_alignment,
        }
    }

    pub fn file_alignment(&self) -> u32 {
        match self {
            OptionalHeader::Pe32(h) => h.file_alignment,
            OptionalHeader::Pe32Plus(h) => h.file_alignment,
        }
    }

    pub fn subsystem(&self) -> u16 {
        match self {
            OptionalHeader::Pe32(h) => h.subsystem,
            OptionalHeader::Pe32Plus(h) => h.subsystem,
        }
    }

    /// Zugriff auf die Data-Directory-Einträge.
    pub fn data_directory(&self, index: usize) -> DataDirectoryEntry {
        match self {
            OptionalHeader::Pe32(h) => h
                .data_directory
                .get(index)
                .copied()
                .unwrap_or(DataDirectoryEntry::EMPTY),
            OptionalHeader::Pe32Plus(h) => h
                .data_directory
                .get(index)
                .copied()
                .unwrap_or(DataDirectoryEntry::EMPTY),
        }
    }
}

/// Optional-Header für PE32 (32-bit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionalHeader32 {
    pub magic: u16,
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,
    pub base_of_code: u32,
    pub base_of_data: u32,
    pub image_base: u32,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub major_operating_system_version: u16,
    pub minor_operating_system_version: u16,
    pub major_image_version: u16,
    pub minor_image_version: u16,
    pub major_subsystem_version: u16,
    pub minor_subsystem_version: u16,
    pub win32_version_value: u32,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub checksum: u32,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub size_of_stack_reserve: u32,
    pub size_of_stack_commit: u32,
    pub size_of_heap_reserve: u32,
    pub size_of_heap_commit: u32,
    pub loader_flags: u32,
    pub number_of_rva_and_sizes: u32,
    pub data_directory: Vec<DataDirectoryEntry>,
}

/// Optional-Header für PE32+ (64-bit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionalHeader64 {
    pub magic: u16,
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,
    pub base_of_code: u32,
    pub image_base: u64,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub major_operating_system_version: u16,
    pub minor_operating_system_version: u16,
    pub major_image_version: u16,
    pub minor_image_version: u16,
    pub major_subsystem_version: u16,
    pub minor_subsystem_version: u16,
    pub win32_version_value: u32,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub checksum: u32,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub size_of_stack_reserve: u64,
    pub size_of_stack_commit: u64,
    pub size_of_heap_reserve: u64,
    pub size_of_heap_commit: u64,
    pub loader_flags: u32,
    pub number_of_rva_and_sizes: u32,
    pub data_directory: Vec<DataDirectoryEntry>,
}
