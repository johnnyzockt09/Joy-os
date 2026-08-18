# PHASE 16 – Performance & Optimierung (Report)

**Status: Basis-Messung + erste Optimierungen.** Echte Messungen im
gebooteten Joys-Live-System (QEMU, TCG-Emulation; echte Hardware ist
deutlich schneller).

## Gemessene Werte (QEMU, 2 vCPU, 2 GB RAM, TCG)

```
ram_free_mb:   1699 MB  (von 1961 MB sichtbar) → sehr leicht
ram_total_mb:  1961 MB
processes:     109      (inkl. Dienste; Desktop-Basis leicht)
disk_used_mb:  12       (Live-Overlay)
uptime:        up 3 minutes (unter TCG langsam; Hardware deutlich schneller)
```

- Der Desktop (openbox + joys-shell + pcmanfm + picom) läuft neben 100+
  Systemprozessen mit **~300 MB verbrauchtem RAM** – im Zielbereich
  „1 GB RAM → bootet, 4 GB → komfortabel".

## Optimierungen in dieser Phase

- **apport deaktiviert** (`/etc/default/apport`, systemd unit disabled):
  spart CPU in QEMU/VMs (frisst sonst Ressourcen bei Xorg-Crashes).
- **Leichtgewichtige Desktop-Session**: openbox + eigene joys-shell
  (Python/GTK3) + pcmanfm statt schwerer Panels; picom nur mit
  Hardware-unterstützten Effekten, falls vorhanden.
- **Kein unnötiger Daemon**: weder LightDM noch GNOME/KDE-Dienste.
- Boot-Profile (minimal/desktop) bleiben für kleine Installationsgrößen.

## Ehrliche Einordnung

- Bootzeit/Benchmarks unter TCG-Emulation sind nicht auf echte Hardware
  übertragbar; `userspace_ms` ist in QEMU ohne `systemd-analyze`-Support
  nicht ermittelbar. Auf echter Hardware wäre `systemd-analyze time` nutzbar.
- Prozesszahl 109 ist eher niedrig für ein Live-System; weitere Reduktion
  (z. B. `systemd-udev`-Autostart, `console-kit`) ist TODO.

## Tests

- Desktop-Test (QEMU): openbox/joys-shell/pcmanfm/Netz/Screenshot/Shutdown
  – alle Checks grün.
- Rust: voller Workspace grün (inkl. update-Tests), Clippy 0.
