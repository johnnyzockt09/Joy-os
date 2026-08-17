# PHASE 11 – Networking (ws2_32) – Report

**Status: MEILENSTEIN ERREICHT** – Windows-Sockets laufen über joys-win,
bewiesen durch einen Loopback-Echo in der gebooteten Live-ISO.

## Umgesetzt (ws2_32.dll, auf Linux-Sockets abgebildet)

| Windows | Abbildung |
|---------|-----------|
| `WSAStartup`/`WSACleanup` | WSADATA-Version 2.2 füllen |
| `socket` | `socket(2)` (SOCKET = Linux-fd) |
| `bind`/`connect`/`listen`/`accept` | libc-Direktaufrufe |
| `send`/`recv` | `send`/`recv` (ssize_t → int) |
| `closesocket`/`getsockname` | `close`/`getsockname` |
| `WSAGetLastError` | thread-lokaler Fehler (aus errno) |
| `htons`/`htonl`/`inet_addr` | echte Byteorder-/Adress-Umrechnung |

- Das Windows-`sockaddr_in` hat auf x86_64 das identische Speicher-Layout
  wie das Linux-`sockaddr_in` (family LE, port BE, addr BE, zero[8]) – die
  Pointer werden direkt an libc übergeben.
- **Ordinal-Imports**: MSVC importiert ws2_32 per Ordinal
  (socket=23, bind=2, connect=4, listen=13, accept=1, send=19, recv=16,
  closesocket=3, getsockname=6, htonl=8, WSAStartup=115, WSACleanup=116,
  WSAGetLastError=111). Die Ordinal-Tabelle wurde aus der echten
  `C:\Windows\System32\ws2_32.dll` ermittelt.

## Test `networktest.exe` (nur kernel32 + ws2_32, kein CRT)

Loopback-Echo: Server (socket→bind Port 0→getsockname→listen→accept),
Client (socket→connect), send("ping")→Server-Echo→recv. Ausgabe:
```
wsastartup=0
sock=1 bind=0
echo=ping net ok=1        (Exit 0)
```

## Tests (gemessen)

- Rust-Workspace: **52 grün** (Linux), 42 grün (Windows), Clippy 0, fmt sauber
- **Live-ISO** (`test-system.sh`, QEMU):
  ```
  OK: networktest.exe: Loopback-Echo (PHASE 11)
  [joys] OK: Systemtest bestanden   (alle 18 Checks grün)
  ```

## Bekannte Punkte / ehrliche Einordnung

- Nur die Kern-Socket-Funktionen; kein select/WSAEventSelect, keine
  async/Overlapped-IO, kein getaddrinfo (DNS).
- Loopback-tauglich; echte Netzwerk-Ziele erfordern das Gastnetz
  (in QEMU: `-netdev user`), was der Desktop-Test nutzt.

## Nächste Phase

PHASE 12: Audio (PipeWire/ALSA) und PHASE 13: Graphics (Vulkan-Basis).

---

*Bericht erstellt: 2026-08-17. Alle Angaben sind gemessen bzw. explizit als
„noch nicht implementiert" gekennzeichnet.*
