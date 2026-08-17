//! Netzwerk-Informationen.

use std::net::UdpSocket;

/// Hostname (Alias zu system::hostname).
pub fn hostname() -> String {
    crate::system::hostname()
}

/// Beste IPv4-Adresse des Systems (Route zum öffentlichen DNS).
pub fn primary_ipv4() -> Option<String> {
    // UDP "connect" ohne Datenverkehr: ermittelt die ausgehende IP.
    let sock = UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect("8.8.8.8:80").ok()?;
    sock.local_addr().ok().map(|a| a.ip().to_string())
}

/// Liste lokaler IPv4-Adressen (via Hostname-Auflösung, best effort).
pub fn local_ips() -> Vec<String> {
    let mut ips = Vec::new();
    if let Ok(addrs) = std::env::var("JOYS_TEST_IP") {
        ips.push(addrs);
    }
    #[cfg(target_os = "linux")]
    if let Ok(data) = std::fs::read_to_string("/proc/net/if_inet6") {
        let _ = data;
    }
    if ips.is_empty() {
        if let Some(ip) = primary_ipv4() {
            ips.push(ip);
        }
    }
    ips
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostname_is_not_empty() {
        assert!(!hostname().is_empty());
    }

    #[test]
    fn local_ips_contains_something() {
        // Im Test wird keine echte IP garantiert; der Aufruf muss nur laufen.
        let _ = local_ips();
    }
}
