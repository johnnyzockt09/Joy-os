# PHASE 8 – Windows-Dateisystem + Registry (Report)

**Status: MEILENSTEIN ERREICHT** – `filetest.exe` schreibt/liest Dateien und
nutzt die Joys-Registry, bewiesen nativ und in der gebooteten Live-ISO.

## Dateisystem-API (kernel32)

| Funktion | Abbildung |
|----------|-----------|
| `CreateFileA` | `open` (CREATE_NEW/ALWAYS, OPEN_EXISTING, TRUNCATE_EXISTING, R/W) |
| `ReadFile` | `read` |
| `WriteFile` | erweitert: `write` (stdout/stderr/Datei-fds) |
| `CloseHandle` | `close` (Pseudo-Handles werden ignoriert) |
| `GetFileSize` | `fstat` |
| `GetCurrentDirectoryA` | `getcwd` |
| `SetCurrentDirectoryA` | `chdir` |

**Pfad-Abbildung** (`api/filesystem.rs`):
- Relative Windows-Pfade → direkt auf das Linux-CWD
- `C:\...` → `~/.joys/windows/...` (virtuelles Laufwerk) – die Windows-App
  merkt nicht, dass darunter Linux läuft

## Registry (advapi32) – Joys-eigene Struktur

NICHT die Windows-Registry kopiert, sondern eine Joys-interne Struktur unter
`~/.joys/windows/registry/`:
- Schlüssel = Verzeichnisse, Werte = Dateien (`Name@Typ`)
- Handle-Tabelle: HKEY → Pfad
- `RegCreateKeyA`, `RegOpenKeyExA`, `RegSetValueExA`, `RegQueryValueExA`,
  `RegDeleteKeyA`, `RegCloseKey`
- Windows-Fehlercodes (ERROR_SUCCESS/ERROR_FILE_NOT_FOUND), keine Dummies

## Technische Hürden (gelöst)

- **7-Arg-Stub** (`CreateFileA`): Stack-Offsets + Alignment für den 5.–7.
  Parameter auf der SysV-Seite
- **HKEY-Sign-Erweiterung**: MSVC übergibt HKEY_CURRENT_USER als
  `0xffffffff80000001` (sign-extended) – Konstanten angepasst
- Registry-Handles über `LazyLock<Mutex<HashMap>>`

## Test-Fixture `filetest.exe`

`tests/binaries/filetest.c` + `filetest.exe` (nur kernel32 + advapi32, kein
CRT): schreibt `joys_test.txt`, liest es zurück, erzeugt/liest Registry-Wert.

## Tests (gemessen)

- `cargo test` Linux: **34 grün**, u. a. `runs_filetest_exe` (prüft
  Datei-Inhalt und Registry-Datei auf dem Dateisystem)
- Windows: 31 grün; Clippy 0; fmt sauber
- **Live-ISO** (`test-system.sh`, QEMU):
  ```
  OK: filetest.exe: Datei schreiben (PHASE 8)
  OK: filetest.exe: Datei lesen (PHASE 8)
  OK: filetest.exe: Registry (PHASE 8)
  ```

## Bekannte Punkte / ehrliche Einordnung

- Nur ANSI-Varianten (CreateFileA/Reg*A); Wide-Varianten folgen später.
- Kein FindFirstFile/FindNextFile, kein GetFullPathName, keine
  Verzeichnis-Enumeration.
- `C:\`-Abbildung ist auf `~/.joys/windows/` fixiert (konfigurierbar nötig).
- Registry-Store ist flach (Datei pro Wert); Konkurrenz/Werte-Size-Budget
  noch nicht behandelt.

## Nächste Phase

PHASE 9: User32 (Fenster/Events) – und PHASE 2: Joys Desktop in der ISO.

---

*Bericht erstellt: 2026-08-16. Alle Angaben sind gemessen bzw. explizit als
„noch nicht implementiert" gekennzeichnet.*
