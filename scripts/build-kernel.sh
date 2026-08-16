#!/usr/bin/env bash
# Installiert den Linux-Kernel + Initramfs in ein Root-Filesystem.
# Später: optional eigener Kernel-Build (kernel/).
#
#   ./scripts/build-kernel.sh <rootfs>

set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/common.sh"

TARGET="${1:-$ROOTFS_DIR}"
[ -d "$TARGET/etc" ] || die "Rootfs fehlt: $TARGET (erst build-rootfs.sh)"

require_root

log "Kernel/Initramfs wird in $TARGET aktualisiert..."

mount --bind /proc "$TARGET/proc" 2>/dev/null || true
mount --bind /sys "$TARGET/sys" 2>/dev/null || true
mount --bind /dev "$TARGET/dev" 2>/dev/null || true

chroot "$TARGET" /bin/bash -c '
    set -e
    export DEBIAN_FRONTEND=noninteractive
    export LANG=C.UTF-8
    if [ ! -f /etc/resolv.conf ]; then echo "nameserver 1.1.1.1" > /etc/resolv.conf; fi
    apt-get update -qq
    apt-get install -y -qq linux-image-generic initramfs-tools || true
    update-initramfs -c -k all || true
    # Stabile Pfade für grub.cfg.
    cd /boot
    ln -sf vmlinuz-* vmlinuz
    ln -sf initrd.img-* initrd.img
    apt-get clean
' || warn "Kernel-Installation hatte Warnungen"

umount "$TARGET/proc" 2>/dev/null || true
umount "$TARGET/sys" 2>/dev/null || true
umount "$TARGET/dev" 2>/dev/null || true

ls -lh "$TARGET"/boot/vmlinuz* "$TARGET"/boot/initrd.img* 2>/dev/null \
    || die "Kein vmlinuz/initrd im Rootfs gefunden"
ok "Kernel + Initramfs bereit"
