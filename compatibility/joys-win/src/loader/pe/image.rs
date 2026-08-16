//! Das geparste PE-Image: verbindet DOS-/COFF-/Optional-Header, Sections,
//! Import-, Export- und Relocation-Tabellen.

use crate::loader::entrypoint::EntryPoint;
use crate::loader::exports::{ExportTable, ExportTableParser};
use crate::loader::imports::{ImportDescriptor, ImportTableParser};
use crate::loader::pe::error::PeParseError;
use crate::loader::pe::reader::Reader;
use crate::loader::pe::{
    CoffHeader, DataDirectoryEntry, Directory, DosHeader, ImageArchitecture, OptionalHeader,
    OptionalHeader32, OptionalHeader64, PE_SIGNATURE,
};
use crate::loader::relocations::{parse_relocations, RelocationBlock};
use crate::loader::sections::Section;

/// Maximale Anzahl von Data-Directory-Einträgen (Windows-Definition).
pub const IMAGE_NUMBEROF_DIRECTORY_ENTRIES: usize = 16;

/// Ein vollständig geparstes PE-Image (aus einem Speicher-Buffer).
#[derive(Debug, Clone)]
pub struct PeImage {
    /// Rohdaten der Datei (für RVA->Offset-Zugriffe auf Tabellen).
    data: Vec<u8>,
    dos: DosHeader,
    coff: CoffHeader,
    optional: OptionalHeader,
    sections: Vec<Section>,
}

impl PeImage {
    /// Parst einen PE-Buffer (Dateiinhalt, z. B. `.exe` oder `.dll`).
    pub fn parse(data: &[u8]) -> Result<Self, PeParseError> {
        // --- DOS-Header ---
        if data.len() < 64 {
            return Err(PeParseError::NotEnoughData {
                offset: 0,
                needed: 64,
                size: data.len(),
            });
        }
        let mut r = Reader::new(data);
        let e_magic = r.read_u16()?;
        if e_magic != crate::loader::pe::DOS_MAGIC {
            return Err(PeParseError::InvalidDosSignature);
        }
        r.skip(58)?;
        let e_lfanew = r.read_u32()?;
        let dos = DosHeader {
            e_magic,
            e_cblp: 0,
            e_cp: 0,
            e_crlc: 0,
            e_cparhdr: 0,
            e_minalloc: 0,
            e_maxalloc: 0,
            e_ss: 0,
            e_sp: 0,
            e_csum: 0,
            e_ip: 0,
            e_cs: 0,
            e_lfarlc: 0,
            e_ovno: 0,
            e_res: [0; 4],
            e_oemid: 0,
            e_oeminfo: 0,
            e_res2: [0; 10],
            e_lfanew,
        };
        let _ = &dos;

        // --- PE-Signatur + COFF-Header ---
        let pe_off = e_lfanew as usize;
        if pe_off + 24 > data.len() {
            return Err(PeParseError::NotEnoughData {
                offset: pe_off,
                needed: 24,
                size: data.len(),
            });
        }
        r.seek(pe_off);
        let signature = r.read_u32()?;
        if signature != PE_SIGNATURE {
            return Err(PeParseError::InvalidPeSignature);
        }
        let machine = r.read_u16()?;
        let number_of_sections = r.read_u16()?;
        let time_date_stamp = r.read_u32()?;
        let pointer_to_symbol_table = r.read_u32()?;
        let number_of_symbols = r.read_u32()?;
        let size_of_optional_header = r.read_u16()?;
        let characteristics = r.read_u16()?;
        let coff = CoffHeader {
            machine,
            number_of_sections,
            time_date_stamp,
            pointer_to_symbol_table,
            number_of_symbols,
            size_of_optional_header,
            characteristics,
        };

        if number_of_sections == 0 {
            return Err(PeParseError::InvalidSectionCount(0));
        }

        // --- Optional-Header ---
        let opt_start = r.pos();
        let magic = r.read_u16()?;
        let optional = match magic {
            crate::loader::pe::OPTIONAL_MAGIC_PE32 => {
                let h = parse_optional_header32(&mut r, magic)?;
                OptionalHeader::Pe32(h)
            }
            crate::loader::pe::OPTIONAL_MAGIC_PE32PLUS => {
                let h = parse_optional_header64(&mut r, magic)?;
                OptionalHeader::Pe32Plus(h)
            }
            other => return Err(PeParseError::UnsupportedOptionalHeaderMagic(other)),
        };

        // --- Sections ---
        let sections_off = opt_start + size_of_optional_header as usize;
        r.seek(sections_off);
        let mut sections = Vec::with_capacity(number_of_sections as usize);
        for _ in 0..number_of_sections {
            sections.push(Section::parse(&mut r)?);
        }

        Ok(PeImage {
            data: data.to_vec(),
            dos,
            coff,
            optional,
            sections,
        })
    }

    /// Roher COFF-Header.
    pub fn coff(&self) -> &CoffHeader {
        &self.coff
    }

    /// DOS-Header (einschließlich e_lfanew).
    pub fn dos_header(&self) -> &DosHeader {
        &self.dos
    }

    /// Rohdaten der Datei.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Optional-Header (PE32 oder PE32+).
    pub fn optional_header(&self) -> &OptionalHeader {
        &self.optional
    }

    /// Alle Sections.
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// Zielarchitektur des Images.
    pub fn architecture(&self) -> ImageArchitecture {
        ImageArchitecture::from_machine(self.coff.machine)
    }

    /// Erkannte Architektur als Text.
    pub fn architecture_name(&self) -> &'static str {
        self.architecture().name()
    }

    /// Ist das Image eine DLL?
    pub fn is_dll(&self) -> bool {
        self.coff.characteristics & 0x2000 != 0
    }

    /// Ist das Image eine ausführbare Datei (.exe)?
    pub fn is_exe(&self) -> bool {
        !self.is_dll()
    }

    /// Entry-Point des Images.
    pub fn entry_point(&self) -> EntryPoint {
        EntryPoint::from_optional_header(&self.optional)
    }

    /// Subsystem (CUI/GUI/...).
    pub fn subsystem(&self) -> u16 {
        self.optional.subsystem()
    }

    /// Data-Directory-Eintrag per Typ.
    pub fn data_directory(&self, dir: Directory) -> DataDirectoryEntry {
        self.optional.data_directory(dir.index())
    }

    /// Rechnet eine RVA in einen Datei-Offset um (anhand der Sections).
    pub fn rva_to_offset(&self, rva: u32) -> Option<usize> {
        for s in &self.sections {
            let va = s.virtual_address;
            let vsize = s.virtual_size.max(s.size_of_raw_data);
            if rva >= va && rva < va.saturating_add(vsize) {
                let delta = rva - va;
                if delta < s.size_of_raw_data {
                    return Some(s.pointer_to_raw_data as usize + delta as usize);
                }
            }
        }
        // Headers abdecken (RVA < SizeOfHeaders).
        let size_of_headers = match &self.optional {
            OptionalHeader::Pe32(h) => h.size_of_headers,
            OptionalHeader::Pe32Plus(h) => h.size_of_headers,
        };
        if rva < size_of_headers {
            return Some(rva as usize);
        }
        None
    }

    fn pointer_size(&self) -> Option<usize> {
        self.architecture().pointer_size().map(usize::from)
    }

    /// Import-Tabelle als ausgewertete Deskriptoren.
    pub fn imports(&self) -> Result<Vec<ImportDescriptor>, PeParseError> {
        let dir = self.data_directory(Directory::Import);
        if dir.virtual_address == 0 || dir.size == 0 {
            return Ok(Vec::new());
        }
        let ps = self
            .pointer_size()
            .ok_or(PeParseError::InvalidImportTable)?;
        ImportTableParser::new(&self.data, &|rva| self.rva_to_offset(rva), ps)
            .parse(dir.virtual_address)
    }

    /// Export-Tabelle.
    pub fn exports(&self) -> Result<Option<ExportTable>, PeParseError> {
        let dir = self.data_directory(Directory::Export);
        if dir.virtual_address == 0 || dir.size == 0 {
            return Ok(None);
        }
        let table = ExportTableParser::new(&self.data, &|rva| self.rva_to_offset(rva))
            .parse(dir.virtual_address, dir.size)?;
        Ok(Some(table))
    }

    /// Basis-Relocation-Blöcke.
    pub fn relocations(&self) -> Result<Vec<RelocationBlock>, PeParseError> {
        let dir = self.data_directory(Directory::BaseRelocation);
        if dir.virtual_address == 0 || dir.size == 0 {
            return Ok(Vec::new());
        }
        parse_relocations(
            &self.data,
            &|rva| self.rva_to_offset(rva),
            dir.virtual_address,
            dir.size,
        )
    }

    /// Übersichtliche Beschreibung des Images.
    pub fn describe(&self) -> String {
        let ep = self.entry_point();
        format!(
            "{} ({}, {}-bit, image_base=0x{:x}, entry={}{})",
            if self.is_dll() { "DLL" } else { "EXE" },
            self.architecture_name(),
            self.architecture().pointer_size().unwrap_or(0) * 8,
            self.optional.image_base(),
            if ep.is_none() {
                "none".to_string()
            } else {
                format!("0x{:x}", ep.address)
            },
            match self.subsystem() {
                2 => ", GUI subsystem",
                3 => ", CUI subsystem",
                _ => "",
            }
        )
    }
}

// NOTE: Das `dos`-Feld wird aktuell nicht einzeln exponiert; die
// DOS-E_MAGIC-Prüfung ist der entscheidende Teil. Struktur ist für
// spätere Nutzung (DOS-Stub-Erkennung) vorbereitet.

fn parse_optional_header32(
    r: &mut Reader<'_>,
    magic: u16,
) -> Result<OptionalHeader32, PeParseError> {
    let major_linker_version = r.read_u8()?;
    let minor_linker_version = r.read_u8()?;
    let size_of_code = r.read_u32()?;
    let size_of_initialized_data = r.read_u32()?;
    let size_of_uninitialized_data = r.read_u32()?;
    let address_of_entry_point = r.read_u32()?;
    let base_of_code = r.read_u32()?;
    let base_of_data = r.read_u32()?;
    let image_base = r.read_u32()?;
    let section_alignment = r.read_u32()?;
    let file_alignment = r.read_u32()?;
    let major_operating_system_version = r.read_u16()?;
    let minor_operating_system_version = r.read_u16()?;
    let major_image_version = r.read_u16()?;
    let minor_image_version = r.read_u16()?;
    let major_subsystem_version = r.read_u16()?;
    let minor_subsystem_version = r.read_u16()?;
    let win32_version_value = r.read_u32()?;
    let size_of_image = r.read_u32()?;
    let size_of_headers = r.read_u32()?;
    let checksum = r.read_u32()?;
    let subsystem = r.read_u16()?;
    let dll_characteristics = r.read_u16()?;
    let size_of_stack_reserve = r.read_u32()?;
    let size_of_stack_commit = r.read_u32()?;
    let size_of_heap_reserve = r.read_u32()?;
    let size_of_heap_commit = r.read_u32()?;
    let loader_flags = r.read_u32()?;
    let number_of_rva_and_sizes = r.read_u32()?;
    let data_directory = read_data_directories(r, number_of_rva_and_sizes)?;

    Ok(OptionalHeader32 {
        magic,
        major_linker_version,
        minor_linker_version,
        size_of_code,
        size_of_initialized_data,
        size_of_uninitialized_data,
        address_of_entry_point,
        base_of_code,
        base_of_data,
        image_base,
        section_alignment,
        file_alignment,
        major_operating_system_version,
        minor_operating_system_version,
        major_image_version,
        minor_image_version,
        major_subsystem_version,
        minor_subsystem_version,
        win32_version_value,
        size_of_image,
        size_of_headers,
        checksum,
        subsystem,
        dll_characteristics,
        size_of_stack_reserve,
        size_of_stack_commit,
        size_of_heap_reserve,
        size_of_heap_commit,
        loader_flags,
        number_of_rva_and_sizes,
        data_directory,
    })
}

fn parse_optional_header64(
    r: &mut Reader<'_>,
    magic: u16,
) -> Result<OptionalHeader64, PeParseError> {
    let major_linker_version = r.read_u8()?;
    let minor_linker_version = r.read_u8()?;
    let size_of_code = r.read_u32()?;
    let size_of_initialized_data = r.read_u32()?;
    let size_of_uninitialized_data = r.read_u32()?;
    let address_of_entry_point = r.read_u32()?;
    let base_of_code = r.read_u32()?;
    let image_base = r.read_u64()?;
    let section_alignment = r.read_u32()?;
    let file_alignment = r.read_u32()?;
    let major_operating_system_version = r.read_u16()?;
    let minor_operating_system_version = r.read_u16()?;
    let major_image_version = r.read_u16()?;
    let minor_image_version = r.read_u16()?;
    let major_subsystem_version = r.read_u16()?;
    let minor_subsystem_version = r.read_u16()?;
    let win32_version_value = r.read_u32()?;
    let size_of_image = r.read_u32()?;
    let size_of_headers = r.read_u32()?;
    let checksum = r.read_u32()?;
    let subsystem = r.read_u16()?;
    let dll_characteristics = r.read_u16()?;
    let size_of_stack_reserve = r.read_u64()?;
    let size_of_stack_commit = r.read_u64()?;
    let size_of_heap_reserve = r.read_u64()?;
    let size_of_heap_commit = r.read_u64()?;
    let loader_flags = r.read_u32()?;
    let number_of_rva_and_sizes = r.read_u32()?;
    let data_directory = read_data_directories(r, number_of_rva_and_sizes)?;

    Ok(OptionalHeader64 {
        magic,
        major_linker_version,
        minor_linker_version,
        size_of_code,
        size_of_initialized_data,
        size_of_uninitialized_data,
        address_of_entry_point,
        base_of_code,
        image_base,
        section_alignment,
        file_alignment,
        major_operating_system_version,
        minor_operating_system_version,
        major_image_version,
        minor_image_version,
        major_subsystem_version,
        minor_subsystem_version,
        win32_version_value,
        size_of_image,
        size_of_headers,
        checksum,
        subsystem,
        dll_characteristics,
        size_of_stack_reserve,
        size_of_stack_commit,
        size_of_heap_reserve,
        size_of_heap_commit,
        loader_flags,
        number_of_rva_and_sizes,
        data_directory,
    })
}

fn read_data_directories(
    r: &mut Reader<'_>,
    count: u32,
) -> Result<Vec<DataDirectoryEntry>, PeParseError> {
    let n = (count as usize).min(IMAGE_NUMBEROF_DIRECTORY_ENTRIES);
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        let virtual_address = r.read_u32()?;
        let size = r.read_u32()?;
        v.push(DataDirectoryEntry {
            virtual_address,
            size,
        });
    }
    Ok(v)
}
