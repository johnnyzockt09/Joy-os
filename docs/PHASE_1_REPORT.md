# PHASE 1 – Bootfähiges Joys Linux (Report)

**Status: ABGESCHLOSSEN (auf Ubuntu-24.04-/26.04-Host, x86_64)**

## Ziel

`./scripts/build-iso.sh` erzeugt aus dem Quellcode reproduzierbar eine
bootfähige ISO. `./scripts/test-system.sh` beweist den Boot in QEMU.

## Erledigt

- `scripts/build-iso.sh` – vollständiger ISO-Build:
  1. `debootstrap`-Rootfs (Ubuntu noble, `minbase`)
  2. Kernel + Initramfs + casper (Live-Boot)
  3. Joys-Binaries (`joys-core`, `joys-win`) ins Rootfs
  4. Boot-Selbsttest-Service (`packages/live/joys-boot-test.{sh,service}`)
  5. SquashFS `/casper/filesystem.squashfs` (Live-Layout)
  6. GRUB-Konfiguration (UEFI+BIOS, Serienkonsole, 3 Menüeinträge)
  7. `grub-mkrescue` → hybrides ISO (BIOS- + UEFI-El-Torito)
  8. `test-iso.sh` (strukturell) + `SHA256SUMS`
- `scripts/test-system.sh` – QEMU-Systemtest (OVMF/UEFI): verifiziert
  GRUB → Kernel → casper → Live-System → joys-binaries → Login-Prompt
- `scripts/build-rootfs.sh`, `scripts/build-kernel.sh`, `scripts/run-qemu.sh`
- CI: `build-iso.yml` baut + testet die ISO (QEMU-Systemtest bei Tags/manuell)

## Tests (gemessen)

`./scripts/test-system.sh` (auf frisch gebauter ISO, TCG-Emulation):

```
OK: GRUB-Bootloader lädt
OK: Boot-Selbsttest startet
OK: Boot-Selbsttest beendet
OK: joys-core läuft im Live-System
OK: joys-win läuft im Live-System
OK: Linux-Kernel (x86_64) läuft
OK: Login-Prompt erreicht
[joys] OK: Systemtest bestanden
```

`./scripts/test-iso.sh` (strukturell, auf `dist/Joys-0.1.0-x86_64.iso`):

```
OK: ISO existiert und ist nicht leer
OK: ISO enthält GRUB-Bootloader
OK: ISO enthält Linux-Kernel
OK: ISO enthält Initramfs
OK: ISO enthält Live-Filesystem (casper)
OK: ISO enthält Filesystem-Größe
OK: ISO ist UEFI-bootbar
OK: ISO ist BIOS-bootbar
[joys] OK: ISO-Test bestanden
```

Rust-Tests (Linux/WSL): `cargo test --workspace` → alle 27 grün (inkl.
joys-core-CLI-Smoke-Test).

## Performance (gemessen)

- ISO-Größe `minimal`: **886 MB** (SquashFS-komprimiert, Kernel 6.8.0-31)
- Bootzeit QEMU (TCG, ohne KVM): ca. 3–4 min bis Login-Prompt (Software-Emulation;
  auf echter Hardware deutlich schneller)
- ISO-Build-Dauer (WSL, einmalig inkl. Rootfs): ca. 10–15 min

## Known issues / ehrliche Einordnung

- **Kein Desktop** (PHASE 2): Das Live-System bootet zu einer Konsole mit
  Login-Prompt. Taskbar/Startmenü/Desktop folgen in PHASE 2.
- **Kein Installer** (PHASE 14): Die ISO ist reines Live-System.
- casper-Logging beim Boot enthält harmlose Warnungen (fehlende
  update-notifier/apport-Dateien im Minimal-Rootfs).
- Dev-Live-ISO setzt `root:joys` und `.hushlogin` – bewusst nur für Test-ISOs,
  nicht für produktive Releases (dokumentiert in `build-rootfs.sh`).
- Bootzeit unter TCG ist lang; CI führt den QEMU-Systemtest daher nur bei
  Tags/manuell aus.

## Nächste Phase

PHASE 2: Joys Desktop (Openbox/WM → Joys Shell → Taskbar → Startmenü) und
PHASE 6: `hello.exe` minimal ausführen.

---

*Bericht erstellt: 2026-08-16. Alle Angaben sind gemessen bzw. explizit als
„noch nicht implementiert" gekennzeichnet.*
