//! Bildet ein PE-Image in den Speicher ab (Mapping, Relocations,
//! Section-Schutz).

use std::ffi::c_void;
use std::ptr;

use crate::loader::pe::OptionalHeader;
use crate::loader::pe::PeImage;
use crate::loader::relocations::types;
use crate::runtime::ExeError;

/// In den Speicher abgebildetes PE-Image.
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
pub struct MappedImage {
    base: *mut u8,
    size: usize,
}

unsafe impl Send for MappedImage {}
unsafe impl Sync for MappedImage {}

impl MappedImage {
    /// Mappt das Image (Headers + Sections) in den Speicher.
    ///
    /// Zuerst wird versucht, an der bevorzugten ImageBase zu laden. Schlägt
    /// das fehl, wird eine freie Adresse verwendet und die Basis-Relocations
    /// werden angewendet. Anfangs sind alle Seiten RW (damit die IAT
    /// beschrieben werden kann); `protect` setzt danach den echten Schutz.
    pub unsafe fn map(img: &PeImage) -> Result<MappedImage, ExeError> {
        let preferred = img.optional_header().image_base();
        let size = img.optional_header().size_of_image() as usize;
        if size == 0 {
            return Err(ExeError::MapFailed("size_of_image ist 0".into()));
        }

        let base = libc::mmap(
            preferred as *mut c_void,
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        );
        if base == libc::MAP_FAILED {
            return Err(ExeError::MapFailed(format!(
                "mmap an 0x{preferred:x} fehlgeschlagen"
            )));
        }
        let base_addr = base as usize;
        let delta = base_addr as isize - preferred as isize;
        if delta != 0 {
            // Ohne Relocations dürfen wir NICHT an anderer Adresse laden
            // (falsche absolute Adressen im Image würden crashen).
            if img.relocations().map_or(true, |b| b.is_empty()) {
                return Err(ExeError::MapFailed(format!(
                    "Image hat keine Relocations, konnte aber nicht an der \
                     bevorzugten ImageBase 0x{preferred:x} geladen werden \
                     (Speicher belegt)"
                )));
            }
            apply_relocations(img, base as *mut u8, delta)?;
        }

        // Headers kopieren.
        let headers_size = match &img.optional_header() {
            OptionalHeader::Pe32(h) => h.size_of_headers,
            OptionalHeader::Pe32Plus(h) => h.size_of_headers,
        } as usize;
        let data = img.data();
        ptr::copy_nonoverlapping(data.as_ptr(), base as *mut u8, headers_size);

        // Sections kopieren.
        for s in img.sections() {
            let dst = (base as *mut u8).add(s.virtual_address as usize);
            let src = data.as_ptr().add(s.pointer_to_raw_data as usize);
            if s.size_of_raw_data > 0 {
                ptr::copy_nonoverlapping(src, dst, s.size_of_raw_data as usize);
            }
        }

        Ok(MappedImage {
            base: base as *mut u8,
            size,
        })
    }

    /// Setzt die finalen Seitenrechte (read/write/exec) aus den Section-Flags.
    pub unsafe fn protect(&self, img: &PeImage) -> Result<(), ExeError> {
        let align = img.optional_header().section_alignment() as usize;
        for s in img.sections() {
            let mut prot = libc::PROT_NONE;
            if s.characteristics.is_read() {
                prot |= libc::PROT_READ;
            }
            if s.characteristics.is_write() {
                prot |= libc::PROT_WRITE;
            }
            if s.characteristics.is_execute() {
                prot |= libc::PROT_EXEC;
            }
            let len = align_up((s.virtual_size.max(1)) as usize, align);
            let r = libc::mprotect(
                self.base.add(s.virtual_address as usize) as *mut c_void,
                len,
                prot,
            );
            if r != 0 {
                return Err(ExeError::MapFailed(format!(
                    "mprotect für Section {} fehlgeschlagen",
                    s.name_str()
                )));
            }
        }
        Ok(())
    }

    /// Adresse bei gegebener RVA.
    pub fn ptr_at(&self, rva: u32) -> *mut u8 {
        unsafe { self.base.add(rva as usize) }
    }
}

impl Drop for MappedImage {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.base as *mut c_void, self.size);
        }
    }
}

/// Wendet die Basis-Relocations an, wenn das Image nicht an der
/// bevorzugten ImageBase geladen wurde.
unsafe fn apply_relocations(img: &PeImage, base: *mut u8, delta: isize) -> Result<(), ExeError> {
    let blocks = img
        .relocations()
        .map_err(|e| ExeError::MapFailed(format!("Relocation-Tabelle nicht lesbar: {e}")))?;
    for b in &blocks {
        for e in &b.entries {
            match e.kind {
                types::ABSOLUTE => {}
                types::DIR64 => {
                    let p = (base as usize + b.page_rva as usize + e.offset_in_block as usize)
                        as *mut u64;
                    *p = p.read().wrapping_add(delta as u64);
                }
                types::HIGHLOW => {
                    let p = (base as usize + b.page_rva as usize + e.offset_in_block as usize)
                        as *mut u32;
                    *p = p.read().wrapping_add(delta as u32);
                }
                other => return Err(ExeError::UnsupportedRelocation(other)),
            }
        }
    }
    Ok(())
}

fn align_up(n: usize, align: usize) -> usize {
    if align == 0 {
        return n;
    }
    n.div_ceil(align) * align
}
