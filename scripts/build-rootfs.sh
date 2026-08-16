#!/usr/bin/env bash
# Erzeugt ein minimales Debian-basiertes Root-Filesystem für Joys.
# Verwendet debootstrap. Benötigt root.
#
#   ./scripts/build-rootfs.sh <ziel-verzeichnis>

set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/common.sh"

TARGET="${1:-$ROOTFS_DIR}"
RELEASE="${UBUNTU_RELEASE:-noble}"
MIRROR="${UBUNTU_MIRROR:-http://archive.ubuntu.com/ubuntu/}"

[ -n "$TARGET" ] || die "Zielverzeichnis fehlt"
require_root
require_cmd debootstrap

if [ -d "$TARGET/etc" ]; then
    warn "Rootfs existiert bereits: $TARGET (überspringe)"
    exit 0
fi

DEB_ARCH="$(deb_arch)"
log "debootstrap $RELEASE -> $TARGET (Debian-Arch $DEB_ARCH, Joys-Arch $ARCH)"
debootstrap --arch="$DEB_ARCH" --variant=minbase --include=\
"systemd-sysv,\
locales,\
kmod,\
udev" \
    "$RELEASE" "$TARGET" "$MIRROR"

# Joys-spezifische Dateien ins Rootfs installieren (Boot-Selbsttest).
install -m 0755 -D "$ROOT_DIR/packages/live/joys-boot-test.sh" \
    "$TARGET/usr/local/bin/joys-boot-test.sh"
install -m 0644 -D "$ROOT_DIR/packages/live/joys-boot-test.service" \
    "$TARGET/etc/systemd/system/joys-boot-test.service"

# Basis-Konfiguration.
mount --bind /proc "$TARGET/proc" 2>/dev/null || true
mount --bind /sys "$TARGET/sys" 2>/dev/null || true
mount --bind /dev "$TARGET/dev" 2>/dev/null || true

chroot "$TARGET" /bin/bash -c '
    set -e
    export DEBIAN_FRONTEND=noninteractive
    echo "joys" > /etc/hostname
    echo "LANG=C.UTF-8" > /etc/default/locale
    rm -f /etc/resolv.conf
    echo "nameserver 1.1.1.1" > /etc/resolv.conf
    apt-get update -qq
    apt-get install -y -qq \
        linux-image-generic \
        initramfs-tools \
        casper \
        systemd-sysv \
        systemd \
        bash \
        coreutils \
        util-linux \
        kmod \
        udev \
        iproute2 \
        iputils-ping \
        net-tools \
        vim-tiny \
        nano \
        less \
        procps \
        e2fsprogs \
        dosfstools \
        pciutils \
        usbutils \
        openssh-client \
    || true
    apt-get clean
    rm -f /etc/resolv.conf
    ln -s /run/systemd/resolve/stub-resolv.conf /etc/resolv.conf 2>/dev/null || true

    # Boot-Selbsttest aktivieren.
    systemctl enable joys-boot-test.service 2>/dev/null || true

    # DEV-LIVE-IMAGE: festes Root-Passwort für den QEMU-/Interaktivtest.
    # Hinweis: Nur für lokale Test-ISOs gedacht, nicht für produktive Releases.
    echo "root:joys" | chpasswd
    touch /root/.hushlogin
    # Bracketed-Paste abschalten, damit Serienkonsole/Expect-Tests funktionieren.
    printf "\nset enable-bracketed-paste off\n" >> /root/.bashrc
'

umount "$TARGET/proc" 2>/dev/null || true
umount "$TARGET/sys" 2>/dev/null || true
umount "$TARGET/dev" 2>/dev/null || true

ok "Rootfs fertig: $TARGET"
