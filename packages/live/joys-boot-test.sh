#!/bin/sh
# Joys Boot-Selbsttest – wird beim Live-Boot als systemd-Service ausgeführt.
# Schreibt die Ergebnisse auf die Serienkonsole (ttyS0), damit sie in QEMU
# automatisiert erfasst werden können (scripts/test-system.sh).
echo "=== JOYS BOOT TEST ==="
uname -a
/usr/bin/joys-core
/usr/bin/joys-win
/usr/bin/joys-win /root/hello.exe
echo "--- joys-win run hello.exe ---"
/usr/bin/joys-win run /root/hello.exe
echo "--- joys-win run apitest.exe ---"
/usr/bin/joys-win run /root/apitest.exe
echo "--- joys-win run filetest.exe ---"
cd /tmp
rm -f joys_test.txt
rm -rf /root/.joys/windows/registry
/usr/bin/joys-win run /root/filetest.exe
echo "--- datei-inhalt ---"
cat /tmp/joys_test.txt 2>/dev/null || echo "(fehlt)"
echo "--- ende ---"
echo "=== JOYS BOOT TEST ENDE ==="
