//! joys-update – prüft Joys-OS-Updates via GitHub Releases.
//!
//!   joys-update                -> zeigt Report (lokal)
//!   joys-update --check <repo> -> prüft GitHub-Release (online)

use joys_core::update;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let repo = "JohnnyZockt09/Joy-os";
    if args.len() >= 3 && args[1] == "--check" {
        let repo = &args[2];
        match update::check_github_release(repo) {
            Ok(Some(remote)) => {
                println!("{}", update::report(update::CURRENT_VERSION, Some(&remote)));
            }
            Ok(None) => {
                println!("{}", update::report(update::CURRENT_VERSION, None));
                println!("(kein neues GitHub-Release gefunden oder offline)");
            }
            Err(e) => {
                eprintln!("joys-update: Fehler bei GitHub-Abfrage: {e}");
                std::process::exit(1);
            }
        }
    } else {
        println!("{}", update::report(update::CURRENT_VERSION, None));
        println!("(Mit --check {} online prüfen)", repo);
    }
}
