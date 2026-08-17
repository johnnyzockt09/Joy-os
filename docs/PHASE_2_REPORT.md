# PHASE 2 – Joys Desktop (Report)

**Status: MEILENSTEIN ERREICHT** – Die Live-ISO bootet in QEMU zu einem
funktionierenden Joys-Desktop mit Fenstermanager, Taskbar/Startmenü,
Dateimanager, Terminal, Netzwerk und funktionierendem Shutdown.

## Umgesetzt

### Desktop-Stack (leichtgewichtig, kein GNOME/KDE)
- **openbox** – Fenstermanager (leicht)
- **lxpanel** – Taskbar + Anwendungsmenü (Startmenü)
- **pcmanfm** – Dateimanager + Desktop-Icons
- **lxterminal** – Terminal
- **lxappearance** – Einstellungen (Erscheinungsbild)
- **xsetroot** – Joys-Hintergrund (#1a1a2e)

### Integration (`scripts/build-desktop.sh`)
- Autologin root auf tty1 → `startx` → `/root/.xinitrc` → `joys-session`
- Openbox-Konfiguration (`rc.xml` + `menu.xml`): Startmenü mit Terminal,
  Dateimanager, Einstellungen, joys-core/joys-win, Neustart/Ausschalten;
  Tastenkürzel (Alt+F1/Ctrl+Esc = Startmenü, Alt+F4 = Schließen,
  Strg+Alt+Entf = Neustart)
- lxpanel-Konfiguration (Taskbar unten, Menü + Launcher + Taskbar + Uhr)
- Xorg-Fix: `modesetting` mit `AccelMethod none` (vermeidet den bekannten
  glamor/llvmpipe-Absturz in QEMU)
- Netzwerk: systemd-networkd mit DHCP für kabelgebundene NICs
- `noprompt` im Kernel-Cmdline (deaktiviert den CD-ROM-Entnahmeprompt beim
  Shutdown, den casper sonst zeigt)

## Tests (gemessen, `scripts/test-desktop.sh` in QEMU/UEFI)

```
OK: Gast-Diagnose vorhanden
OK: Xorg läuft
OK: joys-session läuft
OK: openbox (Fenstermanager) läuft
OK: lxpanel (Taskbar/Startmenü) läuft
OK: pcmanfm (Dateimanager/Desktop) läuft
OK: Netzwerk (DHCP-IP) läuft
OK: Screenshot rendert Desktop (1280x800, 21 Farbcluster, bunte Pixel)
OK: Shutdown funktioniert (Gast fährt herunter, QEMU endet)
[joys] OK: Desktop-Test bestanden
```

Der Screenshot liegt als Artefakt vor (`dist/Joys-Desktop-Screenshot.png`).
`scripts/test-system.sh` bleibt weiterhin grün (GRUB → Kernel → Live-System →
joys-binaries → Login).

## Bekannte Punkte / ehrliche Einordnung

- Kein eigener Window-Manager mehr nötig: openbox als leichtgewichtige Basis.
- „Settings" ist derzeit lxappearance (Erscheinungsbild); ein umfangreicheres
  Joys-Settings-Tool ist TODO.
- Reboot ist im Startmenü verdrahtet (`systemctl reboot`); automatisiert
  getestet ist bisher Shutdown.
- Die ISO wächst durch den Desktop auf ~1,2 GB.
- Boot dauert unter TCG-Emulation (QEMU ohne KVM) ~5 min bis zum Desktop.

## Nächste Phase

- PHASE 9–16 (User32, GDI32, Networking, Audio, Graphics, Installer, Updates,
  Performance) dokumentieren sich fortlaufend; die Architektur dafür steht.

---

*Bericht erstellt: 2026-08-17. Alle Angaben sind gemessen bzw. explizit als
„noch nicht implementiert" gekennzeichnet.*
