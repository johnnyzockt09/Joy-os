#!/bin/sh
# Joys Desktop-Diagnose – startet nach dem Boot und wartet, bis die
# X-Sitzung (openbox) vollständig hochgefahren ist. Schreibt die Diagnose
# auf die Serienkonsole UND (falls verfügbar) in den 9p-Share.
# Statt fixem sleep wartet das Skript aktiv auf openbox (max. 360s).

# Warten bis openbox läuft (X-Sitzung fertig).
for i in $(seq 1 72); do
    if pgrep -x openbox >/dev/null 2>&1; then
        break
    fi
    sleep 5
done

diagnose() {
    echo "=== JOYS DESKTOP DIAG ==="
    echo "--- session-prozesse ---"
    ps aux | grep -E "Xorg|openbox|pcmanfm|joys-shell|joys-settings|xinit|startx|joys-session" | grep -v grep
    echo "--- netzwerk ---"
    ip -4 addr show 2>/dev/null | grep -E "inet |state" | head -6 || echo "(kein ip)"
    echo "--- netzwerk-test ---"
    (ip route get 1.1.1.1 2>/dev/null | head -1) || echo "(kein Netz)"
    echo "--- PERFORMANCE ---"
    echo "userspace_ms: $(systemctl show --property=UserspaceTimestampUSec --value 2>/dev/null || echo '?')"
    echo "uptime: $(uptime -p 2>/dev/null || echo '?')"
    echo "ram_free_mb: $(grep MemAvailable /proc/meminfo | awk '{print int($2/1024)}')"
    echo "ram_total_mb: $(grep MemTotal /proc/meminfo | awk '{print int($2/1024)}')"
    echo "processes: $(ps -e --no-headers 2>/dev/null | wc -l)"
    echo "disk_used_mb: $(df -m / | awk 'NR==2{print int($3)}')"
    echo "kernel: $(uname -r)"
    echo "--- PERFORMANCE ENDE ---"
    echo "--- boot-test service ---"
    systemctl status joys-boot-test.service --no-pager 2>&1 | head -12 || true
    echo "--- joys-win prozesse ---"
    ps aux | grep joys-win | grep -v grep || echo "(keine joys-win)"
    echo "--- Xorg-Log tail ---"
    tail -25 /var/log/Xorg.0.log 2>/dev/null || echo "(kein Xorg-Log)"
    echo "--- exe-assoziation ---"
    command -v xdg-open >/dev/null 2>&1 && \
        xdg-mime query default application/vnd.microsoft.portable-executable 2>/dev/null \
        || echo "(kein xdg-mime)"
    echo "--- joys-exe-manager vorhanden? ---"
    ls -l /usr/local/bin/joys-exe-manager.py 2>/dev/null || echo "(fehlt)"
    echo "--- doppelklick-durchstich (joys-win run hello.exe) ---"
    timeout 10 /usr/bin/joys-win run /root/hello.exe 2>&1 | tail -2
    echo "=== JOYS DESKTOP DIAG ENDE ==="
}

# Immer auf die Serienkonsole ausgeben.
diagnose

# Zusätzlich in den 9p-Share (mit Retry) schreiben, falls verfügbar.
mkdir -p /mnt/host
modprobe 9p 9pnet 9pnet_virtio 2>/dev/null || true
for i in 1 2 3; do
    if mount -t 9p -o trans=virtio hostshare /mnt/host 2>/dev/null; then
        # Installer-Test: nur wenn das DO_INSTALL-Flag gesetzt ist. Die
        # Zielplatte kann /dev/sda oder /dev/vda sein (QEMU-Abstraktion).
        if [ -e /mnt/host/DO_INSTALL ]; then
            echo "=== JOYS INSTALLER TEST (headless) ==="
            TARGET_DISK=""
            for d in /dev/vda /dev/sda /dev/nvme0n1; do
                if [ -b "$d" ]; then TARGET_DISK="$d"; break; fi
            done
            if [ -z "$TARGET_DISK" ]; then
                echo "FEHLER: keine Zielplatte gefunden" > /mnt/host/install.log
            else
                echo "Zielplatte: $TARGET_DISK"
                /usr/local/bin/joys-install.sh "$TARGET_DISK" joys > /mnt/host/install.log 2>&1
            fi
            echo "installer exit=$?" >> /mnt/host/install.log
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
