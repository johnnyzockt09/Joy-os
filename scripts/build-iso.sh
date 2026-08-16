#!/usr/bin/env bash
# ============================================================================
# Joys OS – Bootfähige ISO bauen
#
#   ./scripts/build-iso.sh                # Profil 'minimal'
#   PROFILE=desktop ./scripts/build-iso.sh
#   PROFILE=minimal   ./scripts/build-iso.sh
#
# Ergebnis: dist/Joys-<VERSION>-x86_64.iso  (+ SHA256SUMS)
#
# Voraussetzungen (Ubuntu/Debian):
#   sudo apt install debootstrap xorriso mtools grub-pc-bin \
#                    grub-efi-amd64-bin qemu-utils dosfstools
#
# Ablauf:
#   Source-Code → Root-Filesystem → GRUB/UEFI → ISO-Image → Prüfsumme
# ============================================================================

set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/common.sh"

PROFILE="${PROFILE:-minimal}"
ROOTFS_DIR="$BUILD_DIR/rootfs-$PROFILE"
CACHE_DIR="$BUILD_DIR/cache-$PROFILE"

usage() {
    echo "Verwendung: PROFILE=<minimal|desktop> ./scripts/build-iso.sh [clean]"
    echo "  - ersten Build mit 'clean' starten (Rootfs frisch erzeugen)"
}

if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
    usage
    exit 0
fi

log "=== Joys OS $VERSION ($ARCH, Profil: $PROFILE) ISO-Build ==="

require_cmd debootstrap
require_cmd xorriso
require_cmd grub-mkrescue
require_cmd mformat
require_root

mkdir -p "$DIST_DIR" "$BUILD_DIR"

# 1. Root-Filesystem erzeugen (debootstrap, Ubuntu 'noble').
if [ "${1:-}" = "clean" ] || [ ! -d "$ROOTFS_DIR/etc" ]; then
    log "Root-Filesystem wird erzeugt (debootstrap, noble)..."
    rm -rf "$ROOTFS_DIR"
    ./scripts/build-rootfs.sh "$ROOTFS_DIR"
else
    log "Root-Filesystem vorhanden: $ROOTFS_DIR (mit 'clean' neu bauen)"
fi

# 2. Kernel + Basis-Pakete + Initramfs in das Rootfs installieren.
./scripts/build-kernel.sh "$ROOTFS_DIR"

# 3. Joys Core / joys-win Binaries in das Rootfs kopieren (wenn gebaut).
install_joys_binaries() {
    if [ -f "$ROOT_DIR/target/release/joys-core" ]; then
        install -m 0755 -D "$ROOT_DIR/target/release/joys-core" \
            "$ROOTFS_DIR/usr/bin/joys-core"
        ok "joys-core installiert"
    else
        warn "joys-core nicht gebaut (cargo build --release). Wird übersprungen."
    fi
    if [ -f "$ROOT_DIR/target/release/joys-win" ]; then
        install -m 0755 -D "$ROOT_DIR/target/release/joys-win" \
            "$ROOTFS_DIR/usr/bin/joys-win"
        ok "joys-win installiert"
    else
        warn "joys-win nicht gebaut (cargo build --release). Wird übersprungen."
    fi
}
install_joys_binaries

# 4. GRUB-Konfiguration schreiben.
cat > "$ROOTFS_DIR/boot/grub/grub.cfg" <<EOF
set timeout=3
set default=0

menuentry "Joys OS $VERSION (${PROFILE})" {
    linux /boot/vmlinuz root=/dev/ram0 rw quiet
    initrd /boot/initrd.img
}

menuentry "Joys OS $VERSION (${PROFILE}) – Diagnose" {
    linux /boot/vmlinuz root=/dev/ram0 rw single
    initrd /boot/initrd.img
}
EOF
ok "GRUB-Konfiguration geschrieben"

# 5. ISO mit grub-mkrescue erzeugen (hybrid: UEFI + BIOS).
log "ISO wird erzeugt: $ISO_FILE"
rm -f "$ISO_FILE"
grub-mkrescue -o "$ISO_FILE" "$ROOTFS_DIR" 2>&1 | tail -n 5 \
    || die "grub-mkrescue fehlgeschlagen"
ls -lh "$ISO_FILE"

# 6. Prüfsummen + ISO-Test.
./scripts/test-iso.sh "$ISO_FILE"

cd "$DIST_DIR"
sha256sum "$(basename "$ISO_FILE")" > "$SHA_FILE"
ok "Prüfsumme: $SHA_FILE"

log "=== FERTIG ==="
log "ISO:   $ISO_FILE"
log "Test:  ./scripts/run-qemu.sh"
log "Bereit für: $SHA_FILE"
