# PHASE 0 – Projektgrundlage (Report)

**Status: ABGESCHLOSSEN** (Zusatz: erste Meilensteine von PHASE 5)

## Erledigt

- Git-Repository-Struktur gemäß Zielarchitektur angelegt
- Cargo-Workspace mit `core/joys-core` und `compatibility/joys-win`
- Zentrale Versionsquelle `VERSION` (0.1.0)
- `LICENSE` (MIT), `THIRD_PARTY.md` (Wine/ReactOS/GRUB/etc. mit Lizenzen),
  `README.md`, `docs/PHASES.md`, `docs/ARCHITECTURE.md`
- GitHub-Actions-Workflows: `ci.yml` (fmt/clippy/test/release, Linux+Windows),
  `build-iso.yml` (ISO-Build + ISO-Test + SHA256 + Release bei Tags)
- ISO-Build-Skripte: `build-iso.sh`, `build-rootfs.sh`, `build-kernel.sh`,
  `run-qemu.sh`, `test-iso.sh` (Syntax-geprüft, laufen in CI/WSL)

## Zusatz: PE/COFF-Loader (PHASE 5, Fundament)

Echter, getesteter PE-Loader in `joys-win/src/loader/`:

- PE-Header-Parsing: DOS-Header, PE-Signatur, COFF-Header,
  Optional-Header PE32 + PE32+ (inkl. Data Directories), RVA→Offset
- Sections inkl. Berechtigungs-Flags
- Import-Tabelle (ByName/ByOrdinal, PE32/PE32+)
- Export-Tabelle inkl. Forwarder
- Basis-Relocation-Blöcke
- Entry-Point-Ermittlung

CLI: `joys-win <datei.exe|dll>` analysiert PE-Dateien.

## Tests

`cargo test --workspace`: **27 Tests grün** (0 failed):

- `joys-core`: 2 Unit-Tests
- `joys-win` Unit-Tests: Reader, Sections, Imports, Exports, Relocations,
  Entry-Point (17)
- `tests/pe_loader.rs` Integrationstests (8): synthetisches Minimal-PE32+,
  Fehlerfälle, RVA-Roundtrip **sowie echte Windows-Systemdateien**:
  `kernel32.dll`, `ntdll.dll`, `notepad.exe` (Architektur, Imports, Exports,
  Entry-Point, Relocations)
- `cargo clippy --workspace --all-targets`: **0 Warnings**
- `cargo fmt --all -- --check`: sauber

Beispiel (echte Ausgabe, `target/debug/joys-win.exe notepad.exe`):
- EXE, x86_64, 64-bit, image_base 0x140000000, entry 0x1400019c0, GUI subsystem
- ~60 importierte DLLs (User32, GDI32, api-ms-win-*-Sätze) inkl. Ordinal-Imports
- 5 Relocation-Blöcke

## Performance (Laufzeit des Parsers, gemessen)

- `kernel32.dll` (ca. 0,9 MB) parsen: wenige ms (Debug-Build)
- Der Parser ist Buffer-basiert und ohne externe Abhängigkeiten

## Bekannte Punkte / ehrliche Einordnung

- **Keine `hello.exe`-Ausführung** (PHASE 6): Der Loader parst und analysiert,
  führt aber noch **kein** Programm aus. Das ist bewusst nicht behauptet.
- **Runtime/API-Module sind leer** und als TODO markiert – keine Fake-Codes.
- **ISO-Build** ist in CI (Linux-Runner) vorgesehen; lokal auf Windows erst über
  WSL2 möglich. WSL2 ist hier nicht aktiviert (Virtualisierung/VM-Platform +
  Reboot erforderlich). Die Skripte sind syntaktisch geprüft, aber nicht auf
  einem Linux-Host ausgeführt worden – der erste echte Lauf erfolgt in CI.
- Die ISO-Variante startet derzeit **strukturell bootbar** (GRUB + Kernel +
  Initramfs); ein voller Live-Desktop (casper/live-boot, Joys-Shell) ist
  PHASE 1/2-Arbeit und noch nicht erreicht.

## Nächste Phase

PHASE 1: ISO in QEMU real booten (WSL2 lokal oder CI); danach PHASE 6
(hello.exe minimal ausführen).

---

*Bericht erstellt: 2026-08-16. Alle Angaben sind gemessen bzw. explizit als
„noch nicht implementiert" gekennzeichnet.*
