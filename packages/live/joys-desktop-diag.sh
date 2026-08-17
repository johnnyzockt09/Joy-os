#!/bin/sh
# Joys Desktop-Diagnose – läuft verzögert nach dem Boot, wenn die
# X-Sitzung vollständig gestartet ist. Schreibt die Diagnose auf die
# Serienkonsole UND (falls verfügbar) in den 9p-Share für QEMU-Tests.
sleep 150

diagnose() {
    echo "=== JOYS DESKTOP DIAG ==="
    echo "--- session-prozesse ---"
    ps aux | grep -E "Xorg|openbox|pcmanfm|joys-shell|joys-settings|xinit|startx|joys-session" | grep -v grep
    echo "--- netzwerk ---"
    ip -4 addr show 2>/dev/null | grep -E "inet |state" | head -6 || echo "(kein ip)"
    echo "--- netzwerk-test ---"
    (ip route get 1.1.1.1 2>/dev/null | head -1) || echo "(kein Netz)"
    echo "--- boot-test service ---"
    systemctl status joys-boot-test.service --no-pager 2>&1 | head -12 || true
    echo "--- joys-win prozesse ---"
    ps aux | grep joys-win | grep -v grep || echo "(keine joys-win)"
    echo "--- Xorg-Log tail ---"
    tail -25 /var/log/Xorg.0.log 2>/dev/null || echo "(kein Xorg-Log)"
    echo "=== JOYS DESKTOP DIAG ENDE ==="
}

# Immer auf die Serienkonsole ausgeben.
diagnose

# Zusätzlich in den 9p-Share (mit Retry) schreiben, falls verfügbar.
mkdir -p /mnt/host
modprobe 9p 9pnet 9pnet_virtio 2>/dev/null || true
for i in 1 2 3; do
    if mount -t 9p -o trans=virtio hostshare /mnt/host 2>/dev/null; then
        # Installer-Test: nur wenn das DO_INSTALL-Flag gesetzt ist und eine
        # Zielplatte existiert (für den automatisierten QEMU-Install-Test).
        if [ -e /mnt/host/DO_INSTALL ] && [ -b /dev/sda ]; then
            echo "=== JOYS INSTALLER TEST (headless) ==="
            /usr/local/bin/joys-install.sh /dev/sda joys > /mnt/host/install.log 2>&1
            echo "installer exit=$?"
            umount /mnt/host 2>/dev/null || true
            exit 0
        fi
        diagnose > /mnt/host/desktop-diag.txt 2>&1
        umount /mnt/host 2>/dev/null || true
        echo "9p-Diagnose geschrieben"
        exit 0
    fi
    sleep 5
done
echo "9p-Share nicht verfügbar (Diag auf Serienkonsole)"
