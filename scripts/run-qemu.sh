#!/usr/bin/env bash
# Startet die Joys-ISO in QEMU (UEFI via OVMF).
#
#   ./scripts/run-qemu.sh [datei.iso]
#
# Voraussetzungen:
#   sudo apt install qemu-system-x86 ovmf

set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/common.sh"

ISO="${1:-$ISO_FILE}"
[ -f "$ISO" ] || die "ISO nicht gefunden: $ISO (erst ./scripts/build-iso.sh)"

require_cmd qemu-system-x86_64

# OVMF (UEFI-Firmware) finden.
OVMF_CODE=""
for p in \
    /usr/share/OVMF/OVMF_CODE.fd \
    /usr/share/ovmf/OVMF_CODE.fd \
    /usr/share/edk2/x64/OVMF_CODE.4m.fd \
    /usr/share/edk2-ovmf/OVMF_CODE.fd; do
    if [ -f "$p" ]; then OVMF_CODE="$p"; break; fi
done

QEMU_ARGS=(
    -machine q35,accel=kvm:tcg
    -m 2048
    -smp 2
    -cdrom "$ISO"
    -boot order=d
    -netdev user,id=net0
    -device e1000,netdev=net0
    -display gtk
)

if [ -n "$OVMF_CODE" ]; then
    mkdir -p "$BUILD_DIR"
    OVMF_VARS="$BUILD_DIR/ovmf-vars.fd"
    VARS_SRC="$(dirname "$OVMF_CODE")/OVMF_VARS.fd"
    [ -f "$VARS_SRC" ] || VARS_SRC="$(dirname "$OVMF_CODE")/OVMF_VARS_4M.fd"
    if [ ! -f "$OVMF_VARS" ] && [ -f "$VARS_SRC" ]; then
        cp "$VARS_SRC" "$OVMF_VARS"
    fi
    QEMU_ARGS+=(
        -drive "if=pflash,format=raw,readonly=on,file=$OVMF_CODE"
        -drive "if=pflash,format=raw,file=$OVMF_VARS"
    )
    log "UEFI-Boot mit OVMF ($OVMF_CODE)"
else
    warn "OVMF nicht gefunden – bootet evtl. nur BIOS-SeaBIOS"
fi

log "QEMU startet: $ISO"
qemu-system-x86_64 "${QEMU_ARGS[@]}"
