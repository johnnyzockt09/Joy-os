#!/usr/bin/env bash
# Installer-Test (CI-fähig): bootet die Joys-Live-ISO in QEMU mit einer
# realen virtuellen Platte und führt den Joys-Installer headless aus.
#
#   ./scripts/test-installer.sh [datei.iso]
#
# Voraussetzungen: qemu-system-x86_64, ovmf, virtio (Kernel-Modul im Live).

set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/common.sh"

ISO="${1:-}"
if [ -z "$ISO" ]; then
    ISO="$(ls "$DIST_DIR"/Joys-*-"$ARCH".iso 2>/dev/null | head -n1 || true)"
fi
[ -n "$ISO" ] && [ -f "$ISO" ] || die "ISO nicht gefunden (erst ./scripts/build-iso.sh)"

require_cmd qemu-system-x86_64

OVMF_CODE=""
for p in /usr/share/OVMF/OVMF_CODE_4M.fd /usr/share/OVMF/OVMF_CODE.fd \
    /usr/share/ovmf/OVMF_CODE.fd /usr/share/edk2/x64/OVMF_CODE.4m.fd; do
    if [ -f "$p" ]; then OVMF_CODE="$p"; break; fi
done
[ -n "$OVMF_CODE" ] || die "OVMF nicht gefunden"
OVMF_VARS="$BUILD_DIR/ovmf-vars-install.fd"
VARS_SRC="${OVMF_CODE%_CODE*}""_VARS.fd"; [ -f "$VARS_SRC" ] || VARS_SRC="${OVMF_CODE%_CODE*}""_VARS_4M.fd"
cp "$VARS_SRC" "$OVMF_VARS"

WORK_DIR="$BUILD_DIR/installer-test"
DISK="$WORK_DIR/disk.img"
SHARE="$WORK_DIR/share"
rm -rf "$WORK_DIR"; mkdir -p "$SHARE"
echo "installing" > "$SHARE/DO_INSTALL"
truncate -s 8G "$DISK"

log "Installer-Test startet: $ISO (Kopieren dauert unter TCG ~10-20 min)"
qemu-system-x86_64 \
    -machine q35,accel=tcg -m 2048 -smp 2 \
    -drive "if=pflash,format=raw,readonly=on,file=$OVMF_CODE" \
    -drive "if=pflash,format=raw,file=$OVMF_VARS" \
    -cdrom "$ISO" -boot order=d \
    -drive "file=$DISK,format=raw,if=none,id=disk0" \
    -device "virtio-blk-pci,drive=disk0" \
    -vga std -display none \
    -virtfs "local,path=$SHARE,mount_tag=hostshare,security_model=none" \
    -no-reboot &
QPID=$!

FAIL=0
OK_LOG=0
sleep 5
for i in $(seq 1 150); do
    if grep -aq "FERTIG: Joys wurde installiert" "$SHARE/install.log" 2>/dev/null; then
        OK_LOG=1; echo "Installation abgeschlossen (nach $((i*10))s)"; break
    fi
    if grep -aq "FEHLER: keine Zielplatte\|FEHLER: Zielplatte zu klein" "$SHARE/install.log" 2>/dev/null; then
        echo "FEHLER im Installer-Skript"; break
    fi
    if ! kill -0 $QPID 2>/dev/null; then echo "QEMU vorzeitig beendet"; break; fi
    sleep 10
done
kill $QPID 2>/dev/null || true

check() { # check <name> <bedingung...>
    local name="$1"; shift
    if "$@"; then ok "$name"; else warn "TEST FEHLGESCHLAGEN: $name"; FAIL=1; fi
}

if [ "$OK_LOG" -eq 1 ]; then
    check "Installer meldet erfolgreichen Abschluss" true
    check "Zielplatte wurde beschrieben (nicht leer)" test -s "$DISK"
    check "GRUB wurde installiert" grep -aq "Installing for x86_64-efi" "$SHARE/install.log"
    check "fstab geschrieben" grep -aq "/etc/fstab" "$SHARE/install.log"
else
    check "Installer-Abschluss" false
fi

echo
if [ "$FAIL" -eq 0 ]; then
    ok "Installer-Test bestanden ($ISO)"
    exit 0
else
    die "Installer-Test fehlgeschlagen – Log: $SHARE/install.log"
fi
