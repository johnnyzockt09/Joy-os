# PHASE 3 / 9 / 10 – Report (Joys Core, User32, GDI32)

**Status: MEILENSTEINE ERREICHT** – Joys Core ist zu einer echten System-API
ausgebaut; joys-win kann Windows-GUI-Programme mit User32-Message-Loop und
GDI-Pixel-Zeichnung ausführen. Bewiesen nativ und in der gebooteten Live-ISO.

## PHASE 3 – Joys Core

`core/joys-core` mit echten, getesteten Modulen (Linux + Windows):
- `system`: Architektur, Hostname, Kernel-Release, Uptime
- `hardware`: CPU-Anzahl, RAM total/verfügbar (`/proc/meminfo`)
- `processes`: Prozessliste/-anzahl (`/proc`, Windows: tasklist)
- `files`: Disk-Nutzung (`statvfs`)
- `network`: Hostname, primäre IPv4
- `user`: Benutzername, Home, is_root

`joys-core`-CLI (im Live-System) zeigt echte Daten: Host, Kernel, Uptime,
8 CPUs, 3,8 GB RAM, 32 Prozesse, Disk, Netz-IP.

## PHASE 9 – User32 (Fenster + Nachrichten)

In-process, fensterlose Abbildung (echt funktionierend, keine Dummies):
- Fensterklassen: `RegisterClassExA` (liest WNDCLASSEXA)
- Fenster: `CreateWindowExA` (12 Argumente, inkl. WM_CREATE-Dispatch),
  `ShowWindow`, `UpdateWindow`, `DestroyWindow` (WM_DESTROY)
- Nachrichten: `GetMessageA` (blockierend, Condvar-Queue), `PostMessageA`,
  `PostQuitMessage`, `TranslateMessage`, `DispatchMessageA`
- `DefWindowProcA` (WM_CLOSE → DestroyWindow)
- WndProc-Aufrufe über ein Win64-ABI-Trampoline

Test `windowtest.exe` (nur kernel32+user32): registriert eine Klasse, erzeugt
ein Fenster, postet WM_APP+1, läuft die Message-Loop → Ausgabe:
`register ok / WM_CREATE / create ok / WM_APP+1 / loop end`.

## PHASE 10 – GDI32 (Memory-DC + Pixel)

In-process Pixel-Infrastruktur:
- `GetDC`, `ReleaseDC`, `CreateCompatibleDC`, `CreateCompatibleBitmap`
- `SelectObject` (bindet Bitmap an DC), `DeleteObject`, `DeleteDC`
- `SetPixelV`/`GetPixel` arbeiten real auf dem Pixel-Puffer (RGBA)

Test `gditest.exe`: 8×8-Bitmap, SetPixelV rot → GetPixel → `ok=1`
(COLORREF 0xFF0000 = 16711680). Hinweis: GetDC/ReleaseDC kommen im modernen
SDK aus user32.dll und sind dort ebenfalls implementiert.

## Tests (gemessen)

- Rust-Workspace: **51 grün** (Linux), 42 grün (Windows), Clippy 0, fmt sauber
- **Live-ISO** (`test-system.sh`, QEMU):
  ```
  OK: filetest.exe: Datei schreiben/lesen/Registry (PHASE 8)
  OK: windowtest.exe: User32-Message-Loop (PHASE 9)
  OK: windowtest.exe: Message-Loop beendet
  OK: gditest.exe: GDI-Pixel-Roundtrip (PHASE 10)
  OK: Linux-Kernel (x86_64) läuft
  OK: Login-Prompt erreicht
  [joys] OK: Systemtest bestanden
  ```

## Bekannte Punkte / ehrliche Einordnung

- User32/GDI32 sind **fensterlos**: Fenster/Nachrichten/Pixel sind real,
  aber es gibt noch keine Bildschirm-Ausgabe der Windows-Fenster
  (Mapping auf X/GTK ist TODO).
- Nur ANSI-Varianten (CreateWindowExA/GetMessageA/…); Wide-Varianten folgen.
- Kein ws2_32, kein Audio, kein DirectX/OpenGL (PHASE 11–13).

## Nächste Phase

PHASE 11: ws2_32 (Sockets) – als nächstes voll testbar.

---

*Bericht erstellt: 2026-08-17. Alle Angaben sind gemessen bzw. explizit als
„noch nicht implementiert" gekennzeichnet.*
