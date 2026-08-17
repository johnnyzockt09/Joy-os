# PHASE 12 – Audio (winmm/waveOut via ALSA) – Report

**Status: ERSTER MEILENSTEIN ERREICHT** – die winmm-Audio-API ist als
Builtin implementiert und verhält sich ohne Soundkarte korrekt wie Windows.

## Umgesetzt (winmm.dll → ALSA)

| Funktion | Abbildung |
|----------|-----------|
| `waveOutOpen` | öffnet das ALSA-Default-Device via `dlopen`-`libasound`; **ohne Gerät → MMSYSERR_NODRIVER (6)** |
| `waveOutClose` | schließt das ALSA-PCM |
| `waveOutPrepareHeader` | WAVEHDR validieren |
| `waveOutWrite` | schreibt Puffer an ALSA (`snd_pcm_writei`), setzt `WHDR_DONE` |

- libasound wird zur Laufzeit per `dlopen`/`dlsym` geladen (optionales Modul,
  keine feste Build-Abhängigkeit).
- **Ehrlich**: Ohne echtes Audio-Device (WSL/QEMU ohne Soundkarte) liefert
  `waveOutOpen` `MMSYSERR_NODRIVER` – genau wie Windows ohne Soundkarte. Das
  ist KEIN Dummy, sondern das echte Betriebssystem-Verhalten.

## Tests (gemessen)

- `audiotest.exe` nativ (Windows mit Soundkarte): `waveOutOpen=0 hwo=1 write=0`
- `audiotest.exe` via joys-win (WSL, kein Device):
  ```
  waveOutOpen=6 hwo=0     ← MMSYSERR_NODRIVER, korrekt
  ```
- Rust-Tests: 9 Execution-Tests grün inkl. `runs_audiotest_exe` (akzeptiert
  GDDER 6 oder 0); Clippy 0, Lib 23 Unit-Tests grün.

## Bekannte Punkte / ehrliche Einordnung

- Nur die Kern-`waveOut*`-Funktionen; kein waveIn/Aufnahme, kein MCI,
  kein DirectSound.
- Echte Audio-Wiedergabe ist nur mit ALSA-Device möglich (Soundkarte /
  PipeWire-Passthrough); das ist im Live-System/Sound-Gerät noch nicht
  automatisiert testbar.
- ALSA-Konfigurations-Warnungen erscheinen im stderr, wenn kein `default`-PCM
  konfiguriert ist.

## Nächste Phase

PHASE 13: Graphics (Vulkan-Basis) / PHASE 14: Installer (in Arbeit).

---

*Bericht erstellt: 2026-08-17. Alle Angaben sind gemessen bzw. explizit als
„noch nicht implementiert" gekennzeichnet.*
