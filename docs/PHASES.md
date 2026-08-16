# Joys OS – Phasenplan

Dieses Dokument definiert die verbindliche Entwicklungsreihenfolge. Jede Phase
gilt nur dann als abgeschlossen, wenn ihre Tests grün sind und der Abschluss in
`docs/PHASE_*_REPORT.md` dokumentiert ist.

## Phase 0 – Projektgrundlage ✅
- Git-Repository, Cargo-Workspace, Doku, Lizenz, CI, VERSION

## Phase 1 – Bootfähiges Joys Linux ✅
- UEFI → GRUB → Linux Kernel → casper-Live-Userspace → Login-Prompt
- Reproduzierbar: `./scripts/build-iso.sh` + `./scripts/test-system.sh`
  (siehe `docs/PHASE_1_REPORT.md`)

## Phase 2 – Joys Desktop
- Openbox (oder eigener WM) → Joys Shell → Desktop

## Phase 3 – Joys Core
- System-API, Hardware-Erkennung, Dienste

## Phase 4 – Joys Application Manager
- Linux-Apps, Windows-Apps, AppImages, Flatpaks

## Phase 5 – Joys-Win PE Loader ✅
- .exe erkennen, PE-Header lesen, Architektur erkennen, Sections laden,
  Entry-Point erkennen (Tests gegen echte Windows-Systemdateien)

## Phase 6 – erste .exe ausführen ✅
- `hello.exe` → gibt `Hello from Windows!` aus (kein Wine) – **gemessen**
  nativ und in der gebooteten Joys-Live-ISO
  (siehe `docs/PHASE_6_REPORT.md`)

## Phase 7 – Kernel32
- Fortsetzung: weitere kernel32-Funktionen über die Builtin-Architektur
## Phase 8 – Filesystem + Registry
## Phase 9 – User32
## Phase 10 – GDI32
## Phase 11 – Networking (ws2_32)
## Phase 12 – Audio
## Phase 13 – Graphics (Vulkan-Basis)
## Phase 14 – Installer
## Phase 15 – Update-System
## Phase 16 – Performance-Optimierung

## Definition of Done – Joys 0.1

- ISO baut, startet in QEMU, Kernel + Desktop laufen
- Taskbar, Startmenü, Terminal, File Manager, Settings funktionieren
- Netzwerk, Shutdown, Reboot funktionieren
- GitHub CI grün, Doku vorhanden
- `joys-win` lädt und führt mindestens eine selbst kompilierte `.exe` aus

## Definition of Done – Joys-Win 0.1

- PE32+ / x86_64, .exe-Loading, Imports, Exports, Relocations, Entrypoint
- Basis-Prozess, Basis-Speicher, Basis-Kernel32, Konsolenausgabe, Dateisystem
- Test: `hello.exe` funktioniert direkt unter Joys, ohne Wine
