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

log "debootstrap $RELEASE -> $TARGET (Arch $ARCH)"
debootstrap --arch="$ARCH" --variant=minbase --include=\
"systemd-sysv,\
locales,\
kmod,\
udev" \
    "$RELEASE" "$TARGET" "$MIRROR"

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
    || true
    apt-get clean
    rm -f /etc/resolv.conf
    ln -s /run/systemd/resolve/stub-resolv.conf /etc/resolv.conf 2>/dev/null || true
'

umount "$TARGET/proc" 2>/dev/null || true
umount "$TARGET/sys" 2>/dev/null || true
umount "$TARGET/dev" 2>/dev/null || true

ok "Rootfs fertig: $TARGET"
