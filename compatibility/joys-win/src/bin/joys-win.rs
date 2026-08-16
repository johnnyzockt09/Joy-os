//! joys-win CLI: erkennt und analysiert Windows-Programme (.exe/.dll)
//! über den PE/COFF-Loader.
//!
//! Status: Analyse-Modus (PHASE 5). Das tatsächliche Ausführen eines
//! Programms (PHASE 6) ist noch nicht implementiert.

use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Verwendung: joys-win <datei.exe|dll>");
        std::process::exit(2);
    }

    let path = Path::new(&args[1]);
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("joys-win: kann {} nicht lesen: {e}", path.display());
            std::process::exit(1);
        }
    };

    match joys_win::PeImage::parse(&data) {
        Ok(image) => {
            println!("Datei:     {}", path.display());
            println!("Bild:      {}", image.describe());
            println!("Architektur: {}", image.architecture_name());

            match image.imports() {
                Ok(imports) if !imports.is_empty() => {
                    println!("Imports:");
                    for d in &imports {
                        let names: Vec<String> = d
                            .imports
                            .iter()
                            .map(|i| match i {
                                joys_win::loader::imports::Import::ByName { name, .. } => {
                                    name.clone()
                                }
                                joys_win::loader::imports::Import::ByOrdinal { ordinal } => {
                                    format!("#{ordinal}")
                                }
                            })
                            .collect();
                        println!("  {} ({})", d.dll_name, names.join(", "));
                    }
                }
                _ => println!("Imports:   (keine)"),
            }

            match image.exports() {
                Ok(Some(t)) => println!("Exports:   {} ({})", t.dll_name, t.exports.len()),
                _ => println!("Exports:   (keine)"),
            }

            match image.relocations() {
                Ok(blocks) => println!("Relocations: {} Blöcke", blocks.len()),
                _ => println!("Relocations: (keine)"),
            }
        }
        Err(e) => {
            eprintln!("joys-win: kein valides PE-Image: {e}");
            std::process::exit(3);
        }
    }
}
