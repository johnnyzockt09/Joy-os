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
echo "--- joys-win run windowtest.exe (User32) ---"
/usr/bin/joys-win run /root/windowtest.exe
echo "--- joys-win run gditest.exe (GDI32) ---"
/usr/bin/joys-win run /root/gditest.exe
echo "--- joys-win run networktest.exe (ws2_32) ---"
/usr/bin/joys-win run /root/networktest.exe
echo "--- joys-win run audiotest.exe (winmm) ---"
timeout 15 /usr/bin/joys-win run /root/audiotest.exe 2>/dev/null | grep waveOut || echo "(kein waveOut-Output, Audio-Device fehlt)"
echo "--- desktop-check (unabhaengig vom 9p-Service) ---"
for i in $(seq 1 90); do
    if pgrep -x openbox >/dev/null 2>&1 && pgrep -f "joys-shell.py" >/dev/null 2>&1; then
        break
    fi
    sleep 5
done
if pgrep -x openbox >/dev/null 2>&1; then echo "DESKTOP_CHECK: openbox laeuft"; else echo "DESKTOP_CHECK: openbox fehlt"; fi
if pgrep -f "joys-shell.py" >/dev/null 2>&1; then echo "DESKTOP_CHECK: joys-shell laeuft"; else echo "DESKTOP_CHECK: joys-shell fehlt"; fi
if pgrep -x pcmanfm >/dev/null 2>&1; then echo "DESKTOP_CHECK: pcmanfm laeuft"; else echo "DESKTOP_CHECK: pcmanfm fehlt"; fi
echo "--- ende ---"
echo "=== JOYS BOOT TEST ENDE ==="
