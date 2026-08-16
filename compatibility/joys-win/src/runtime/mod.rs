//! Führt ein PE-Image aus: Mapping, Import-Auflösung, Entry-Point-Aufruf.
//!
//! Unterstützt aktuell: PE32+ / x86_64 auf Linux (mit Win64-ABI-Stubs).
//! Andere Plattformen liefern `ExeError::UnsupportedPlatform`.

mod error;

pub use error::ExeError;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
mod abi;
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
mod image;

use crate::loader::pe::PeImage;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
use image::MappedImage;

/// Führt ein geparstes PE-Image aus und liefert den Exit-Code.
///
/// # Safety
/// Springt in nicht verwalteten, aus dem PE gemappten Code.
pub unsafe fn run(img: &PeImage) -> Result<i32, ExeError> {
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    {
        let mapped = MappedImage::map(img)?;
        resolve_imports(img, &mapped)?;
        mapped.protect(img)?;

        let ep = img.entry_point();
        if ep.is_none() {
            return Err(ExeError::NoEntryPoint);
        }
        let entry: unsafe extern "C" fn() -> u32 = std::mem::transmute(mapped.ptr_at(ep.rva));
        let code = entry();
        Ok(code as i32)
    }

    #[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
    {
        let _ = img;
        Err(ExeError::UnsupportedPlatform)
    }
}

/// Füllt die IAT mit Adressen der joys-win-Builtins.
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
unsafe fn resolve_imports(img: &PeImage, mapped: &MappedImage) -> Result<(), ExeError> {
    let descriptors = img
        .imports()
        .map_err(|e| ExeError::MapFailed(format!("Import-Tabelle nicht lesbar: {e}")))?;
    let ps = img
        .architecture()
        .pointer_size()
        .expect("x86_64 hat 8-Byte-Zeiger") as u32;

    for d in &descriptors {
        let dll = d.dll_name.to_ascii_lowercase();
        for (i, imp) in d.imports.iter().enumerate() {
            let target = match dll.as_str() {
                "kernel32.dll" => crate::api::kernel32::resolve(imp)?,
                other => return Err(ExeError::UnimplementedApi(other.into(), import_name(imp))),
            };
            let iat_rva = d.first_thunk + i as u32 * ps;
            let slot = mapped.ptr_at(iat_rva) as *mut u64;
            *slot = target as u64;
        }
    }
    Ok(())
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn import_name(imp: &crate::loader::imports::Import) -> String {
    match imp {
        crate::loader::imports::Import::ByName { name, .. } => name.clone(),
        crate::loader::imports::Import::ByOrdinal { ordinal } => format!("#{ordinal}"),
    }
}
