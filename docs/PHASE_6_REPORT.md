# PHASE 6 – Erste Windows-Programme ausführen (Report)

**Status: ERSTER MEILENSTEIN ERREICHT** – `hello.exe` läuft über `joys-win`,
ohne Wine, sowohl nativ auf Linux als auch **in der gebooteten Joys-Live-ISO**.

## Ziel

Ein selbst kompiliertes `hello.exe` soll unter Joys direkt ausgeführt werden:

```
hello.exe → PE-Parser → Image-Mapping → Import-Auflösung (kernel32-Builtin)
         → Win64-ABI-Stubs → Entry-Point → "Hello from Windows!"
```

## Umgesetzt

### runtime/ (Ausführung)
- `runtime/image.rs` – **echtes PE-Mapping**: mmap an bevorzugter ImageBase
  (Fallback: Basis-Relocations, inkl. Schutzprüfung), Section-Kopien,
  finaler Seiten-Schutz (R/W/X) via `mprotect`
- `runtime/abi.rs` – **Win64→SysV-ABI-Brücke** in Assembly (`global_asm!`,
  Intel-Syntax): übersetzt RCX/RDX/R8/R9 nach RDI/RSI/RDX/RCX und erhält die
  Win64-callee-saved-Register
- `runtime/mod.rs` – `joys-win run <exe>`: Map → IAT füllen → Entry aufrufen
  (PE32+/x86_64-Linux; andere Plattformen: sauberer `UnsupportedPlatform`)

### api/kernel32.rs (Builtin-DLL, echte Abbildung)
- `GetStdHandle` → Windows-Pseudo-Handles (-10/-11/-12)
- `WriteFile` → `write(2)` auf fd 1/2
- `ExitProcess` → `_exit(code)`
- Nicht implementierte API → klare Fehlermeldung, KEINE Dummy-Werte

### Test-Fixture
- `tests/binaries/hello.c` + `hello.exe`: minimiertes PE32+ (nur kernel32,
  kein CRT, 3 KB), Build via `scripts/build-hello.sh` (MSVC/MinGW)
- `tests/execution.rs`: `joys-win run hello.exe` als Subprozess,
  prüft stdout `Hello from Windows!`

## Tests (gemessen)

`cargo test --workspace` (Linux): **28 Tests grün**, u. a.:
- `runs_hello_exe` – führt `hello.exe` aus, stdout = `Hello from Windows!`, Exit 0
- `analyzes_hello_exe` – Imports (WriteFile, GetStdHandle, ExitProcess) erkannt
- `missing_entry_point_is_error` – kein PE → Exit-Code 3

Clippy (beide Plattformen): 0 Warnings. `cargo fmt`: sauber.

**Beweis im Live-System** (`./scripts/test-system.sh`, ISO in QEMU gebootet):
```
OK: joys-win läuft im Live-System
OK: hello.exe läuft via joys-win im Live-System   ← "Hello from Windows!"
```

## Bekannte Punkte / ehrliche Einordnung

- Nur **kernel32: GetStdHandle/WriteFile/ExitProcess** sind implementiert.
  Jede weitere API → expliziter Fehler statt Dummy.
- Nur **PE32+ / x86_64 / Linux** wird ausgeführt. PE32 (32-bit) und andere
  Plattformen → `UnsupportedPlatform`.
- Kein PEB/TEB, kein TLS, kein Loader-Lock, keine System-DLLs außerhalb der
  Builtins – bewusst minimal für den ersten Meilenstein.
- Die Ausführung ist **in-process** (kein separates sandboxiertes Modell);
  Isolierung/Sandboxing ist dokumentierte TODO-Arbeit.

## Definition of Done – Joys-Win 0.1 (Teilbeweis)

| Punkt | Status |
|-------|--------|
| PE32+ / x86_64 | ✅ |
| .exe-Loading | ✅ |
| Imports | ✅ (kernel32-Builtin) |
| Exports | ✅ (Parser; Builtin-DLL-Export folgt) |
| Relocations | ✅ (Parser + Anwendung) |
| Entrypoint | ✅ |
| Basis-Prozess/Speicher | ✅ (Mapping, mprotect) |
| Basis-kernel32 | ✅ (3 Funktionen, echt) |
| Konsolenausgabe | ✅ (`Hello from Windows!`) |
| Dateisystem | ⏳ (PHASE 8) |
| **hello.exe direkt, ohne Wine** | ✅ **gemessen** |

## Nächste Phase

Weitere kernel32-Funktionen (Sleep, GetTickCount, GetSystemInfo, VirtualAlloc…),
Dateisystem-API (PHASE 8) und dann User32 (PHASE 9).

---

*Bericht erstellt: 2026-08-16. Alle Angaben sind gemessen bzw. explizit als
„noch nicht implementiert" gekennzeichnet.*
