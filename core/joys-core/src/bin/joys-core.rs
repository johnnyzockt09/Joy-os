//! joys-core CLI: Systeminformationen von Joys OS.

use joys_core::system;

fn main() {
    println!("Joys Core {}", joys_core::JOYS_VERSION);
    println!("Host-Architektur: {}", system::host_arch());
}
