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

## Phase 2 – Joys Desktop ✅
- Openbox (WM) + lxpanel (Taskbar/Startmenü) + pcmanfm (Dateimanager/Desktop)
  + lxterminal + lxappearance; Autologin → Joys-Sitzung in der Live-ISO
- Netzwerk (DHCP) und Shutdown funktionieren; Screenshot-Beweis in QEMU
  (siehe `docs/PHASE_2_REPORT.md`)

## Phase 3 – Joys Core ✅
- System-API: system/hardware/processes/files/network/user mit echten
  Implementierungen (siehe `docs/PHASE_3_9_10_REPORT.md`)

## Phase 4 – Joys Application Manager
- Grundgerüst offen

## Phase 4 – Joys Application Manager
- Linux-Apps, Windows-Apps, AppImages, Flatpaks

## Phase 5 – Joys-Win PE Loader ✅
- .exe erkennen, PE-Header lesen, Architektur erkennen, Sections laden,
  Entry-Point erkennen (Tests gegen echte Windows-Systemdateien)

## Phase 6 – erste .exe ausführen ✅
- `hello.exe` → gibt `Hello from Windows!` aus (kein Wine) – **gemessen**
  nativ und in der gebooteten Joys-Live-ISO
  (siehe `docs/PHASE_6_REPORT.md`)

## Phase 7 – Kernel32 ✅
- 14 kernel32-Funktionen real auf Linux abgebildet (Sleep, GetTickCount,
  Prozess/Thread-ID, GetLastError, VirtualAlloc/Free, GetSystemInfo,
  lstrlenA …), bewiesen via apitest.exe in der Live-ISO
  (siehe `docs/PHASE_7_REPORT.md`)

## Phase 8 – Filesystem + Registry ✅
- Windows-Dateisystem-API (CreateFileA/ReadFile/WriteFile/CloseHandle/
  GetFileSize/GetCurrentDirectoryA/SetCurrentDirectoryA) mit Pfad-Abbildung
  (relativ → Linux-CWD, `C:\` → `~/.joys/windows/`)
- Joys-eigene Registry unter `~/.joys/windows/registry/` (advapi32:
  RegCreateKeyA/RegOpenKeyExA/RegSetValueExA/RegQueryValueExA/RegCloseKey)
- Bewiesen via filetest.exe in der Live-ISO
  (siehe `docs/PHASE_8_REPORT.md`)

## Phase 9 – User32 ✅
- Fensterklassen, Fenster, Message-Loop (RegisterClassExA/CreateWindowExA/
  GetMessageA/DispatchMessageA/PostMessageA/…) mit Win64-WndProc-Trampoline
- Bewiesen via windowtest.exe in der Live-ISO

## Phase 10 – GDI32 ✅
- Memory-DCs, Bitmaps, SetPixelV/GetPixel (echte Pixel, kein Dummy)
- Bewiesen via gditest.exe in der Live-ISO
  (beide: `docs/PHASE_3_9_10_REPORT.md`)
## Phase 11 – Networking (ws2_32) ✅
- socket/bind/connect/listen/accept/send/recv/closesocket/getsockname +
  WSAStartup/WSAGetLastError/htons/htonl/inet_addr auf Linux-Sockets;
  Ordinal-Imports unterstützt
- Bewiesen via networktest.exe (Loopback-Echo) in der Live-ISO
  (siehe `docs/PHASE_11_REPORT.md`)
## Phase 12 – Audio ✅ (winmm/waveOut → ALSA)
- waveOutOpen/Close/PrepareHeader/Write auf ALSA (dlopen-libasound);
  ohne Device MMSYSERR_NODRIVER wie Windows
  (siehe `docs/PHASE_12_REPORT.md`)
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
