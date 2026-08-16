#!/usr/bin/env bash
# Testet eine gebaute Joys-ISO auf strukturelle Korrektheit:
#   - ISO existiert, ist nicht leer
#   - enthält Bootloader (grub) und Kernel (vmlinuz) und Initramfs
#   - enthält Root-Filesystem (usr/bin/bash)
#   - xorriso kann das Image lesen
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
check() { # check <name> <bedingung...>
    local name="$1"; shift
    if "$@"; then ok "$name"; else
        warn "TEST FEHLGESCHLAGEN: $name"; FAIL=1
    fi
}

size=$(stat -c%s "$ISO")
check "ISO existiert und ist nicht leer" test "$size" -gt 10000000

# Inhalt über xorriso auflisten (ISO-9660).
LISTING="$(xorriso -indev "$ISO" -find / -exec lsdl -- 2>/dev/null || true)"

check "ISO enthält GRUB-Bootloader" bash -c "echo \"\$1\" | grep -qi 'boot/grub' && echo \"\$1\" | grep -qi 'core.img'" _ "$LISTING"
check "ISO enthält Linux-Kernel" bash -c "echo \"\$1\" | grep -q 'vmlinuz'" _ "$LISTING"
check "ISO enthält Initramfs" bash -c "echo \"\$1\" | grep -q 'initrd'" _ "$LISTING"
check "ISO enthält Root-Filesystem" bash -c "echo \"\$1\" | grep -q 'usr/bin/bash'" _ "$LISTING"

check "xorriso liest ISO ohne Fehler" xorriso -indev "$ISO" -report_el_torito 2>/dev/null

echo
if [ "$FAIL" -eq 0 ]; then
    ok "ISO-Test bestanden ($ISO, $(du -h "$ISO" | cut -f1))"
else
    die "ISO-Test fehlgeschlagen"
fi
