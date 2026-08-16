#!/bin/sh
# Joys Boot-Selbsttest – wird beim Live-Boot als systemd-Service ausgeführt.
# Schreibt die Ergebnisse auf die Serienkonsole (ttyS0), damit sie in QEMU
# automatisiert erfasst werden können (scripts/test-system.sh).
echo "=== JOYS BOOT TEST ==="
uname -a
/usr/bin/joys-core
/usr/bin/joys-win
echo "=== JOYS BOOT TEST ENDE ==="
