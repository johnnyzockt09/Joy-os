//! joys-win CLI.
//!
//!   joys-win <datei.exe|dll>        PE-Datei analysieren (PHASE 5)
//!   joys-win run <datei.exe>        PE-Datei ausführen (PHASE 6, x86_64-Linux)
//!   joys-win --version              Version anzeigen

use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Verwendung: joys-win <datei.exe|dll> | joys-win run <datei.exe>");
        std::process::exit(2);
    }

    if args[1] == "--version" || args[1] == "-V" {
        println!("joys-win {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if args[1] == "run" {
        if args.len() < 3 {
            eprintln!("Verwendung: joys-win run <datei.exe>");
            std::process::exit(2);
        }
        run_exe(&args[2]);
        return;
    }

    analyze(&args[1]);
}

fn read_file(path: &str) -> Vec<u8> {
    let p = Path::new(path);
    match std::fs::read(p) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("joys-win: kann {} nicht lesen: {e}", p.display());
            std::process::exit(1);
        }
    }
}

fn analyze(path: &str) {
    let data = read_file(path);
    match joys_win::PeImage::parse(&data) {
        Ok(image) => {
            println!("Datei:       {}", Path::new(path).display());
            println!("Bild:        {}", image.describe());

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
                _ => println!("Imports:     (keine)"),
            }

            match image.exports() {
                Ok(Some(t)) => println!("Exports:     {} ({})", t.dll_name, t.exports.len()),
                _ => println!("Exports:     (keine)"),
            }

            match image.relocations() {
                Ok(blocks) => println!("Relocations: {} Blöcke", blocks.len()),
                _ => println!("Relocations: (keine)"),
            }
            println!();
            println!("Ausführen mit: joys-win run {}", Path::new(path).display());
        }
        Err(e) => {
            eprintln!("joys-win: kein valides PE-Image: {e}");
            std::process::exit(3);
        }
    }
}

fn run_exe(path: &str) {
    let data = read_file(path);
    let image = match joys_win::PeImage::parse(&data) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("joys-win: kein valides PE-Image: {e}");
            std::process::exit(3);
        }
    };

    println!("joys-win: führe {} aus ...", Path::new(path).display());
    println!("{}", image.describe());

    // Sicherheitshinweis ausgeben, aber trotzdem ausführen (User hat `run` gewählt).
    match unsafe { joys_win::runtime::run(&image) } {
        Ok(code) => {
            println!("joys-win: Prozess beendet mit Exit-Code {code}");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("joys-win: Ausführung fehlgeschlagen: {e}");
            std::process::exit(4);
        }
    }
}
