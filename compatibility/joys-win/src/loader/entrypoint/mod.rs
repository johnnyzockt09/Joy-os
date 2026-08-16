//! Entry-Point-Ermittlung.

use crate::loader::pe::OptionalHeader;

/// Der Entry-Point eines PE-Images.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryPoint {
    /// RVA des Entry-Points (0 bei reinen Daten-DLLs ohne Code).
    pub rva: u32,
    /// Absolute Adresse (RVA + ImageBase).
    pub address: u64,
}

impl EntryPoint {
    /// Leerer Entry-Point (kein Code).
    pub const NONE: EntryPoint = EntryPoint { rva: 0, address: 0 };

    /// Ermittelt den Entry-Point aus dem Optional-Header.
    pub fn from_optional_header(opt: &OptionalHeader) -> Self {
        let rva = opt.entry_point();
        if rva == 0 {
            return EntryPoint::NONE;
        }
        EntryPoint {
            rva,
            address: opt.image_base().wrapping_add(u64::from(rva)),
        }
    }

    pub fn is_none(&self) -> bool {
        self.rva == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::pe::OptionalHeader32;

    #[test]
    fn computes_absolute_address() {
        let opt = OptionalHeader::Pe32(OptionalHeader32 {
            magic: 0x010b,
            major_linker_version: 0,
            minor_linker_version: 0,
            size_of_code: 0,
            size_of_initialized_data: 0,
            size_of_uninitialized_data: 0,
            address_of_entry_point: 0x1234,
            base_of_code: 0x1000,
            base_of_data: 0,
            image_base: 0x0040_0000,
            section_alignment: 0x1000,
            file_alignment: 0x200,
            major_operating_system_version: 6,
            minor_operating_system_version: 0,
            major_image_version: 0,
            minor_image_version: 0,
            major_subsystem_version: 6,
            minor_subsystem_version: 0,
            win32_version_value: 0,
            size_of_image: 0x2000,
            size_of_headers: 0x200,
            checksum: 0,
            subsystem: 3,
            dll_characteristics: 0,
            size_of_stack_reserve: 0x100000,
            size_of_stack_commit: 0x1000,
            size_of_heap_reserve: 0x100000,
            size_of_heap_commit: 0x1000,
            loader_flags: 0,
            number_of_rva_and_sizes: 0,
            data_directory: vec![],
        });
        let ep = EntryPoint::from_optional_header(&opt);
        assert_eq!(ep.rva, 0x1234);
        assert_eq!(ep.address, 0x0040_1234);
        assert!(!ep.is_none());
    }

    #[test]
    fn none_for_zero_entry() {
        let opt = OptionalHeader::Pe32(OptionalHeader32 {
            address_of_entry_point: 0,
            ..OptionalHeader32 {
                magic: 0x010b,
                major_linker_version: 0,
                minor_linker_version: 0,
                size_of_code: 0,
                size_of_initialized_data: 0,
                size_of_uninitialized_data: 0,
                address_of_entry_point: 0,
                base_of_code: 0,
                base_of_data: 0,
                image_base: 0x0040_0000,
                section_alignment: 0x1000,
                file_alignment: 0x200,
                major_operating_system_version: 6,
                minor_operating_system_version: 0,
                major_image_version: 0,
                minor_image_version: 0,
                major_subsystem_version: 6,
                minor_subsystem_version: 0,
                win32_version_value: 0,
                size_of_image: 0x2000,
                size_of_headers: 0x200,
                checksum: 0,
                subsystem: 3,
                dll_characteristics: 0,
                size_of_stack_reserve: 0x100000,
                size_of_stack_commit: 0x1000,
                size_of_heap_reserve: 0x100000,
                size_of_heap_commit: 0x1000,
                loader_flags: 0,
                number_of_rva_and_sizes: 0,
                data_directory: vec![],
            }
        });
        assert!(EntryPoint::from_optional_header(&opt).is_none());
    }
}
