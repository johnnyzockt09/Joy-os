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
#                    grub-efi-amd64-bin qemu-utils dosfstools squashfs-tools
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
require_cmd mksquashfs
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

# 2b. Joys Desktop (Openbox, Taskbar, Startmenü, File Manager, Terminal).
./scripts/build-desktop.sh "$ROOTFS_DIR"

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
    # hello.exe als Test-Fixture ins Live-System (für PHASE-6-Beweis in QEMU).
    if [ -f "$ROOT_DIR/compatibility/joys-win/tests/binaries/hello.exe" ]; then
        install -m 0644 -D \
            "$ROOT_DIR/compatibility/joys-win/tests/binaries/hello.exe" \
            "$ROOTFS_DIR/root/hello.exe"
        ok "hello.exe als Test-Fixture installiert"
    fi
    # apitest.exe als Test-Fixture (PHASE 7: erweiterte kernel32-Abdeckung).
    if [ -f "$ROOT_DIR/compatibility/joys-win/tests/binaries/apitest.exe" ]; then
        install -m 0644 -D \
            "$ROOT_DIR/compatibility/joys-win/tests/binaries/apitest.exe" \
            "$ROOTFS_DIR/root/apitest.exe"
        ok "apitest.exe als Test-Fixture installiert"
    fi
    # filetest.exe als Test-Fixture (PHASE 8: Dateisystem + Registry).
    if [ -f "$ROOT_DIR/compatibility/joys-win/tests/binaries/filetest.exe" ]; then
        install -m 0644 -D \
            "$ROOT_DIR/compatibility/joys-win/tests/binaries/filetest.exe" \
            "$ROOTFS_DIR/root/filetest.exe"
        ok "filetest.exe als Test-Fixture installiert"
    fi
    # windowtest.exe / gditest.exe (PHASE 9/10: User32/GDI32).
    for f in windowtest gditest; do
        if [ -f "$ROOT_DIR/compatibility/joys-win/tests/binaries/$f.exe" ]; then
            install -m 0644 -D \
                "$ROOT_DIR/compatibility/joys-win/tests/binaries/$f.exe" \
                "$ROOTFS_DIR/root/$f.exe"
            ok "$f.exe als Test-Fixture installiert"
        fi
    done
}
install_joys_binaries

# 4. Live-Stage im casper-Layout vorbereiten.
#    ISO-Struktur:
#      /boot/vmlinuz, /boot/initrd.img
#      /casper/filesystem.squashfs  (komprimiertes Root-Filesystem)
#      /casper/filesystem.size
#      /boot/grub/grub.cfg
LIVE_DIR="$BUILD_DIR/live-$PROFILE"
rm -rf "$LIVE_DIR"
mkdir -p "$LIVE_DIR/boot" "$LIVE_DIR/casper"
cp -a "$ROOTFS_DIR"/boot/. "$LIVE_DIR/boot/"

log "Root-Filesystem wird als SquashFS komprimiert..."
mksquashfs "$ROOTFS_DIR" "$LIVE_DIR/casper/filesystem.squashfs" \
    -noappend -processors 2 -quiet
# Unkomprimierte Rootfs-Größe in Bytes (für casper/filesystem.size).
du -sB1 "$ROOTFS_DIR" | awk '{print $1}' > "$LIVE_DIR/casper/filesystem.size"
du -sh "$LIVE_DIR/casper/filesystem.squashfs"

# GRUB-Konfiguration schreiben (Live-Boot über casper, UEFI+BIOS,
# Serienkonsole für Headless-/QEMU-Tests).
cat > "$LIVE_DIR/boot/grub/grub.cfg" <<EOF
set timeout=5
set default=0
set gfxmode=auto

if loadfont /boot/grub/fonts/unicode.pf2; then
    set gfxterm=1
    insmod all_video
    insmod gfxterm
    terminal_output gfxterm
fi

serial --unit=0 --speed=115200
terminal_input console serial
terminal_output console serial

menuentry "Joys OS $VERSION (${PROFILE})" {
    linux /boot/vmlinuz boot=casper noprompt quiet splash console=tty0 console=ttyS0,115200
    initrd /boot/initrd.img
}

menuentry "Joys OS $VERSION (${PROFILE}) – Diagnose (serial)" {
    linux /boot/vmlinuz boot=casper noprompt console=ttyS0,115200
    initrd /boot/initrd.img
}

menuentry "Joys OS $VERSION (${PROFILE}) – Single User" {
    linux /boot/vmlinuz boot=casper noprompt single console=ttyS0,115200
    initrd /boot/initrd.img
}
EOF
ok "GRUB-Konfiguration geschrieben (Live-Boot via casper)"

# 5. ISO mit grub-mkrescue erzeugen (hybrid: UEFI + BIOS).
log "ISO wird erzeugt: $ISO_FILE"
rm -f "$ISO_FILE"
grub-mkrescue -o "$ISO_FILE" "$LIVE_DIR" 2>&1 | tail -n 5 \
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
