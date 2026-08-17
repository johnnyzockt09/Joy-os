#!/usr/bin/env bash
# Joys Installer – installiert das laufende Live-System auf eine Festplatte.
#
#   ./scripts/joys-install.sh /dev/sdX [benutzername]
#
# Ablauf: GPT-Partitionierung (EFI + ext4) -> Formatieren -> Kopieren
# (rsync) -> GRUB/UEFI installieren -> Benutzer + Hostname setzen.
# Sicherheitshinweis: überschreibt die Zielplatte vollständig!

set -euo pipefail

DISK="${1:-}"
USERNAME="${2:-joys}"
[ -b "$DISK" ] || { echo "FEHLER: '$DISK' ist kein Blockgerät"; exit 1; }

echo "=== Joys Installer ==="
echo "Zielplatte: $DISK  (wird VOLLSTÄNDIG überschrieben!)"

TARGET=/mnt/joys-install
case "$DISK" in
    /dev/nvme*) P1="${DISK}p1"; P2="${DISK}p2" ;;
    *)          P1="${DISK}1"; P2="${DISK}2" ;;
esac

umount "$TARGET" 2>/dev/null || true

echo "[1/6] Partitionieren (GPT: EFI + ext4) ..."
parted -s "$DISK" mklabel gpt
parted -s "$DISK" mkpart ESP fat32 1MiB 513MiB
parted -s "$DISK" set 1 esp on
parted -s "$DISK" mkpart ROOT ext4 513MiB 100%
partprobe "$DISK" || true
sleep 1

echo "[2/6] Formatieren ..."
mkfs.fat -F32 -n JOYS-EFI "$P1"
mkfs.ext4 -F -L JOYS-ROOT "$P2"

echo "[3/6] Kopieren des Live-Systems (rsync) ..."
mkdir -p "$TARGET"
mount "$P2" "$TARGET"
mkdir -p "$TARGET/boot/efi"
mount "$P1" "$TARGET/boot/efi"
rsync -aAX --info=progress2 \
    --exclude=/proc --exclude=/sys --exclude=/dev --exclude=/run \
    --exclude=/tmp --exclude=/cdrom --exclude=/rofs --exclude=/media \
    --exclude=/mnt --exclude=/var/cache/apt/archives \
    / "$TARGET/"

echo "[4/6] GRUB/UEFI installieren ..."
mount --bind /dev "$TARGET/dev"
mount --bind /proc "$TARGET/proc"
mount --bind /sys "$TARGET/sys"

ROOT_UUID="$(blkid -s UUID -o value "$P2")"
EFI_UUID="$(blkid -s UUID -o value "$P1")"

chroot "$TARGET" /bin/bash -c "
    set -e
    export DEBIAN_FRONTEND=noninteractive
    echo 'nameserver 1.1.1.1' > /etc/resolv.conf
    grub-install --target=x86_64-efi --efi-directory=/boot/efi --bootloader-id=Joys
    update-grub || true
    echo 'joys' > /etc/hostname
    echo 'root:joys' | chpasswd
    id -u $USERNAME 2>/dev/null || useradd -m -s /bin/bash $USERNAME
    echo '$USERNAME:joys' | chpasswd
    rm -f /etc/resolv.conf
    ln -s /run/systemd/resolve/stub-resolv.conf /etc/resolv.conf 2>/dev/null || true
" || echo "WARN: chroot-Konfiguration hatte Fehler"

echo "[5/6] /etc/fstab schreiben ..."
cat > "$TARGET/etc/fstab" <<EOF
# /etc/fstab – statische Informationen über Dateisysteme.
# <file system> <mount point> <type> <options> <dump> <pass>
UUID=$ROOT_UUID / ext4 errors=remount-ro 0 1
UUID=$EFI_UUID /boot/efi vfat umask=0077 0 1
EOF

echo "[6/6] Aufräumen ..."
umount "$TARGET/proc" 2>/dev/null || true
umount "$TARGET/sys" 2>/dev/null || true
umount "$TARGET/dev" 2>/dev/null || true
umount "$TARGET/boot/efi" 2>/dev/null || true
umount "$TARGET" 2>/dev/null || true

echo "=== FERTIG: Joys wurde installiert. Neustart möglich. ==="
