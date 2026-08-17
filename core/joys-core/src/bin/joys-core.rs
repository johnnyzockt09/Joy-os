//! joys-core CLI: Systeminformationen von Joys OS.

use joys_core::{files, hardware, network, processes, system, user};

fn main() {
    println!("Joys Core {}", joys_core::JOYS_VERSION);
    println!(
        "Host:      {} ({})",
        system::hostname(),
        system::host_arch()
    );
    println!("Kernel:    {}", system::kernel_release());
    println!("Uptime:    {} s", system::uptime_secs());
    println!("Hardware:  {}", hardware::describe());
    println!("Prozesse:  {}", processes::process_count());
    println!("Benutzer:  {}", user::username());
    println!(
        "Home:      {}",
        user::home_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    );
    if let Some(du) = files::disk_usage("/") {
        println!(
            "Disk /:    {:.0} MB belegt / {:.0} MB frei",
            du.used_bytes() as f64 / (1024.0 * 1024.0),
            du.available_bytes as f64 / (1024.0 * 1024.0)
        );
    }
    if let Some(ip) = network::primary_ipv4() {
        println!("Netz:      {ip}");
    }
}
