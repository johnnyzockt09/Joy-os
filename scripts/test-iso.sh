#!/usr/bin/env bash
# Testet eine gebaute Joys-ISO auf strukturelle Korrektheit:
#   - ISO existiert, ist nicht leer
#   - enthält Bootloader (grub) und Kernel (vmlinuz) und Initramfs
#   - enthält Root-Filesystem (usr/bin/bash)
#   - xorriso kann das Image lesen (El Torito UEFI/BIOS)
#
# Wenn ein Test fehlschlägt, endet das Skript mit Exit-Code 1.
#
#   ./scripts/test-iso.sh [datei.iso]

set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/common.sh"

ISO="${1:-$ISO_FILE}"
[ -f "$ISO" ] || die "ISO nicht gefunden: $ISO"

require_cmd xorriso

FAIL=0
check() { # check <name> <befehl...>
    local name="$1"; shift
    if "$@"; then ok "$name"; else
        warn "TEST FEHLGESCHLAGEN: $name"; FAIL=1
    fi
}

size=$(stat -c%s "$ISO")
check "ISO existiert und ist nicht leer" test "$size" -gt 10000000

# Dateiliste in Temp-Datei statt als Kommandoargument (ARG_MAX).
LISTING="$(mktemp)"
trap 'rm -f "$LISTING"' EXIT
xorriso -indev "$ISO" -find / -exec lsdl -- 2>/dev/null > "$LISTING" || true

check "ISO enthält GRUB-Bootloader" grep -q 'boot/grub/grub.cfg' "$LISTING"
check "ISO enthält Linux-Kernel" grep -q 'vmlinuz' "$LISTING"
check "ISO enthält Initramfs" grep -q 'initrd' "$LISTING"
check "ISO enthält Live-Filesystem (casper)" grep -q 'casper/filesystem.squashfs' "$LISTING"
check "ISO enthält Filesystem-Größe" grep -q 'casper/filesystem.size' "$LISTING"

# El Torito: UEFI- und BIOS-Booteinträge vorhanden.
ET="$(mktemp)"
trap 'rm -f "$LISTING" "$ET"' EXIT
xorriso -indev "$ISO" -report_el_torito 2>/dev/null > "$ET" || true
check "ISO ist UEFI-bootbar" grep -q 'UEFI' "$ET"
check "ISO ist BIOS-bootbar" grep -q 'BIOS' "$ET"

echo
if [ "$FAIL" -eq 0 ]; then
    ok "ISO-Test bestanden ($ISO, $(du -h "$ISO" | cut -f1))"
else
    die "ISO-Test fehlgeschlagen"
fi
