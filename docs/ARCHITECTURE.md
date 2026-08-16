# Joys OS – Architektur

## Übersicht

```
                         JOYS OS
                            │
              ┌─────────────┴─────────────┐
              │                           │
        Linux Applications          Windows Applications
              │                           │
       .deb / AppImage                  .exe
              │                           │
              │                     ┌─────▼─────┐
              │                     │ joys-win  │
              │                     │            │
              │                     │ PE Loader  │
              │                     │ Win32 API  │
              │                     │ Win64 API  │
              │                     │ DLL Loader │
              │                     │ Registry   │
              │                     │ Processes  │
              │                     │ Threads    │
              │                     │ User32     │
              │                     │ Kernel32   │
              │                     └─────┬──────┘
              │                           │
              └──────────────┬────────────┘
                             │
                       Joys Core
                             │
                       Linux Kernel
```

## Komponenten

### joys-core (`core/joys-core`)
Zentrale System-API: Hardware, Prozesse, Dateien, Netzwerk, Grafik, Audio,
Benutzer, Permissions, Updates. `joys-win` greift ausschließlich über diese
definierten Schnittstellen zu. Status: Grundgerüst (PHASE 3 offen).

### joys-win (`compatibility/joys-win`)
Eigene Windows-Kompatibilitätsruntime – **ohne Wine als Laufzeitabhängigkeit**.

```
.exe/.dll
   ↓
PE-Loader (loader/)
   ↓
Runtime (runtime/)          [geplant: Prozess-, Thread-, Speicher-, Handle-Modell]
   ↓
Win32-API (api/)            [geplant: kernel32, ntdll, user32, gdi32, …]
   ↓
DLL-System (dll/)           [geplant: Discovery, Resolver, Builtins]
   ↓
Registry / Filesystem       [geplant: ~/.joys/windows/…]
   ↓
joys-core
   ↓
Linux Kernel
```

### Implementierter Stand (joys-win loader/)
| Modul | Inhalt | Status |
|-------|--------|--------|
| `loader/pe` | DOS-, COFF-, Optional-Header (PE32/PE32+), Data Directories, RVA→Offset | ✅ getestet |
| `loader/sections` | Section-Modell + Berechtigungen | ✅ getestet |
| `loader/imports` | Import-Tabelle (ByName/ByOrdinal, PE32/PE32+) | ✅ getestet |
| `loader/exports` | Export-Tabelle inkl. Forwarder | ✅ getestet |
| `loader/relocations` | Basis-Relocation-Blöcke | ✅ getestet |
| `loader/entrypoint` | Entry-Point (RVA + absolute Adresse) | ✅ getestet |
| `runtime/` | Prozesse/Threads/Speicher/Handles | ⏳ TODO |
| `api/` | Win32-API | ⏳ TODO |
| `dll/` | DLL-System | ⏳ TODO |
| `registry/` | Registry | ⏳ TODO |

## Windows→Linux Abbildung (Konzept)

- **Prozesse**: Windows-Prozessmodell → Linux-Prozesse (`CreateProcess` →
  Joys Process Manager → Linux-Prozess)
- **Threads**: Windows-Threads → pthreads/native Mechanismen; Verwaltung von
  Windows Thread IDs/Handles
- **Speicher**: `VirtualAlloc/VirtualFree` → mmap
- **Handles**: eigene Handle-Tabelle in joys-win
- **Registry**: Joys-eigene Struktur unter `~/.joys/windows/registry/`
- **Dateisystem**: virtuelle Windows-Pfade (`C:\Program Files`, `C:\Users`) →
  `~/.joys/windows/`
- **Grafik**: User32/GDI → Joys Desktop-/Window-System (GTK)
- **Netzwerk**: ws2_32 → Linux sockets
- **Audio**: Windows-Audio → PipeWire/ALSA

## Definition of Done (Kurzform)

Siehe [`PHASES.md`](PHASES.md). Joys 0.1: bootfähige ISO + Desktop-Grundfunktion
+ `joys-win` lädt/führt mindestens `hello.exe` aus. Kein Wine.
