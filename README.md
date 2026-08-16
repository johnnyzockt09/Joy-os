# Joys OS

Joys OS ist ein schnelles, leichtes und modernes Linux-Betriebssystem mit einem
Windows-artigen Desktop-Erlebnis und einer eigenen Windows-Kompatibilitäts­runtime
(`joys-win`), die **ohne Wine** auskommt.

> **Status: PHASE 0/1 – frühe Entwicklung.** Nichts in diesem Repository ist als
> „fertig" zu betrachten, solange es nicht getestet ist. Siehe `docs/`.

## Vision

```
                 JOYS OS

 Linux .deb ────────────────┐
 AppImage ──────────────────┤
 Flatpak ───────────────────┤
                            ▼
                         JOYS CORE
                            ▲
                            │
 Windows .exe ──→ JOYS-WIN ┘
                            │
                       Linux Kernel
```

Doppelklick auf eine `.exe` → `joys-win` erkennt das PE-File, lädt es über den
eigenen PE-Loader und führt es über eine Windows-API-Kompatibilitätsschicht aus.
Wine ist **keine** Laufzeitabhängigkeit.

## Repository-Struktur

```
Joys/
├── core/              → joys-core (System-API, Hardware, Prozesse, Dienste)
├── desktop/           → Joys Desktop (Shell, Taskbar, Startmenü) [geplant]
├── apps/              → native Joys-Anwendungen [geplant]
├── compatibility/     → joys-win (PE-Loader, Win32-API, DLL-System) [in Arbeit]
├── installer/         → UEFI-Installer [geplant]
├── updater/           → joys-update [geplant]
├── packages/          → Paketsystem [geplant]
├── kernel/            → Kernel-Konfiguration für den ISO-Build [geplant]
├── build/             → ISO-Build-Artefakte (lokal, nicht versioniert)
├── scripts/           → build-iso.sh, run-qemu.sh, test-iso.sh …
├── docs/              → Architektur- und Phasen-Dokumentation
├── tests/             → übergeordnete Tests
├── .github/workflows/ → CI (Test + ISO-Build)
├── LICENSE
├── THIRD_PARTY.md
├── README.md
└── VERSION
```

## Version

Die zentrale Versionsquelle ist [`VERSION`](VERSION) (aktuell: **0.1.0**).
Sie wird von ISO-Dateinamen, Build-System, CI und Update-System verwendet.

## Build (Linux / WSL)

```bash
./scripts/build-iso.sh      # baut dist/Joys-<version>-x86_64.iso
./scripts/run-qemu.sh       # startet die ISO in QEMU (UEFI)
./scripts/test-iso.sh       # testet die ISO-Artefakte
```

Der ISO-Build benötigt eine Linux-Umgebung (z. B. WSL2/Ubuntu) oder läuft in
GitHub Actions.

## Rust-Workspace

```bash
cargo build --workspace
cargo test  --workspace
```

- `core/joys-core` – zentrale System-API
- `compatibility/joys-win` – Windows-Kompatibilitäts­runtime, beginnend mit einem
  echten PE/COFF-Loader

## Entwicklung

Siehe [`docs/PHASES.md`](docs/PHASES.md) für die Phasenliste und
`docs/PHASE_0_REPORT.md` für den Stand der aktuell abgeschlossenen Phase.

## Lizenzen & Drittanbieter

Siehe [`LICENSE`](LICENSE) und [`THIRD_PARTY.md`](THIRD_PARTY.md).
