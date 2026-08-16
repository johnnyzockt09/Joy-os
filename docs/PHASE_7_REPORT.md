# PHASE 7 – Kernel32-Erweiterung (Report)

**Status: MEILENSTEIN ERREICHT** – kernel32-Builtin deckt jetzt eine echte,
auf Linux abgebildete Funktionsgruppe ab. Bewiesen nativ und in der
gebooteten Joys-Live-ISO.

## Neue kernel32-Funktionen (alle mit echter Linux-Abbildung)

| Funktion | Abbildung |
|----------|-----------|
| `Sleep` | `nanosleep` |
| `GetTickCount` | `CLOCK_MONOTONIC` in ms |
| `GetCurrentProcess` | Pseudo-Handle (-1) |
| `GetCurrentProcessId` | `getpid` |
| `GetCurrentThreadId` | `gettid` |
| `GetLastError`/`SetLastError` | thread-lokaler Fehlerwert (TLS) |
| `VirtualAlloc` | `mmap` (mit Alloc-Tracking) |
| `VirtualFree` | `munmap` (über Tracking, MEM_RELEASE/DECOMMIT) |
| `GetSystemInfo` | SYSTEM_INFO (x64-Layout) mit echten Linux-Daten |
| `lstrlenA` | C-String-Länge |
| `GetCommandLineA` | leere Kommandozeile (TODO: Argv-Mapping) |

Zusammen mit den PHASE-6-Basisfunktionen (`GetStdHandle`, `WriteFile`,
`ExitProcess`) sind damit **14 kernel32-Funktionen** real implementiert.

## Technik

- **ABI-Stubs per Makro**: `runtime/abi.rs` erzeugt Win64→SysV-Stubs für
  0–4 Argumente (`mov edi, ecx` für DWORD = Zero-Extend, `mov rdi, rcx` für
  Zeiger/u64); `WriteFile` (5 Argumente) und `ExitProcess` (noreturn) bleiben
  manuell.
- **`VirtualAlloc`-Tracking**: statisches `HashMap` (Adresse → Größe) für
  korrektes `munmap` bei `MEM_RELEASE`.

## Test-Fixture `apitest.exe`

`tests/binaries/apitest.c` + `apitest.exe` (nur kernel32, kein CRT) übt alle
neuen Funktionen aus. Natives Windows-Referenzergebnis und joy-win-Ergebnis
stimmen überein (GetLastError=4660, nproc, page=4096, gran=65536, arch=9,
lstrlenA=5, Sleep-Diff, VirtualAlloc/Free).

## Tests (gemessen)

- `cargo test --workspace` Linux: **32 grün** (u. a. neue `runs_apitest_exe`,
  Unit-Tests für `lstrlen`, `GetLastError`, `VirtualAlloc`-Roundtrip)
- Windows: **30 grün** (Ausführungstests platformbedingt deaktiviert)
- Clippy: 0 Warnings (beide Plattformen), `cargo fmt`: sauber
- **Live-ISO-Beweis** (`./scripts/test-system.sh`, QEMU):
  ```
  OK: apitest.exe (kernel32-API) läuft via joys-win im Live-System
  OK: apitest.exe: lstrlenA korrekt
  ```

## Bekannte Punkte / ehrliche Einordnung

- Nicht implementierte kernel32-Funktionen → `UnimplementedApi` (kein Dummy).
- `GetCommandLineA` liefert vorerst eine leere Kommandozeile
  (TODO: Environment-/Argv-Mapping aus dem Aufrufer).
- `VirtualFree` MEM_DECOMMIT wird vereinfachend wie munmap behandelt.
- `GetSystemInfo` liefert Linux-Werte (echt), nicht Windows-Hardware-Werte.
- Kein HeapAlloc/GetProcessHeap, keine File-API (PHASE 8), kein User32 (PHASE 9).

## Nächste Phase

PHASE 8: Windows-Dateisystem-API (`CreateFile`, `ReadFile`, `WriteFile` auf
Dateien, `GetCurrentDirectory` …) + Registry. Danach User32 (PHASE 9).

---

*Bericht erstellt: 2026-08-16. Alle Angaben sind gemessen bzw. explizit als
„noch nicht implementiert" gekennzeichnet.*
