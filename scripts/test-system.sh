#!/usr/bin/env bash
# Automatisierter Systemtest: bootet die Joys-ISO in QEMU (UEFI/OVMF) und
# verifiziert die gesamte Bootkette bis zu den Joys-Binaries im Live-System.
#
#   ./scripts/test-system.sh [datei.iso]
#
# Prüft:
#   - GRUB lädt
#   - Linux-Kernel startet
#   - casper/Live-System bootet
#   - Login-Prompt erscheint
#   - joys-core und joys-win laufen im Live-System (Boot-Selbsttest)
#
# Voraussetzungen: qemu-system-x86_64, ovmf.
# Hinweis: Boot dauert unter TCG (ohne KVM) mehrere Minuten.

set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/common.sh"

ISO="${1:-}"
if [ -z "$ISO" ]; then
    ISO="$(ls "$DIST_DIR"/Joys-*-"$ARCH".iso 2>/dev/null | head -n1 || true)"
fi
[ -n "$ISO" ] && [ -f "$ISO" ] || die "ISO nicht gefunden (erst ./scripts/build-iso.sh)"

require_cmd qemu-system-x86_64

# OVMF finden.
OVMF_CODE=""
for p in \
    /usr/share/OVMF/OVMF_CODE_4M.fd \
    /usr/share/OVMF/OVMF_CODE.fd \
    /usr/share/ovmf/OVMF_CODE.fd \
    /usr/share/edk2/x64/OVMF_CODE.4m.fd; do
    if [ -f "$p" ]; then OVMF_CODE="$p"; break; fi
done
[ -n "$OVMF_CODE" ] || die "OVMF-Firmware nicht gefunden (apt install ovmf)"

OVMF_VARS="$BUILD_DIR/ovmf-vars-test.fd"
VARS_SRC="${OVMF_CODE%_CODE*}""_VARS.fd"
[ -f "$VARS_SRC" ] || VARS_SRC="${OVMF_CODE%_CODE*}""_VARS_4M.fd"
[ -f "$VARS_SRC" ] || die "OVMF-VARS nicht gefunden"
cp "$VARS_SRC" "$OVMF_VARS"

BOOTLOG="$BUILD_DIR/system-test.log"
mkdir -p "$BUILD_DIR"
rm -f "$BOOTLOG"

log "QEMU-Systemtest startet: $ISO"
timeout 420 qemu-system-x86_64 \
    -machine q35,accel=tcg \
    -m 2048 -smp 2 \
    -drive "if=pflash,format=raw,readonly=on,file=$OVMF_CODE" \
    -drive "if=pflash,format=raw,file=$OVMF_VARS" \
    -cdrom "$ISO" -boot order=d \
    -display none -serial stdio -no-reboot \
    > "$BOOTLOG" 2>&1 || true

FAIL=0
expect() { # expect <marker> <beschreibung>
    if grep -aq "$1" "$BOOTLOG"; then ok "$2"
    else warn "TEST FEHLGESCHLAGEN: $2 (fehlt '$1')"; FAIL=1; fi
}

expect "GNU GRUB" "GRUB-Bootloader lädt"
expect "JOYS BOOT TEST" "Boot-Selbsttest startet"
expect "JOYS BOOT TEST ENDE" "Boot-Selbsttest beendet"
expect "Joys Core" "joys-core läuft im Live-System"
expect "joys-win <datei" "joys-win läuft im Live-System"
expect "Hello from Windows!" "hello.exe läuft via joys-win im Live-System"
expect "nproc=" "apitest.exe (kernel32-API) läuft via joys-win im Live-System"
expect "lstrlenA=5" "apitest.exe: lstrlenA korrekt"
expect "write_ok=1 written=12 size=12" "filetest.exe: Datei schreiben (PHASE 8)"
expect "content=Hello file!" "filetest.exe: Datei lesen (PHASE 8)"
expect "reg_create=0 set=0 get=0 value=registry works" "filetest.exe: Registry (PHASE 8)"
expect "WM_APP+1" "windowtest.exe: User32-Message-Loop (PHASE 9)"
expect "loop end" "windowtest.exe: Message-Loop beendet"
expect "get=16711680 ok=1" "gditest.exe: GDI-Pixel-Roundtrip (PHASE 10)"
expect "echo=ping net ok=1" "networktest.exe: Loopback-Echo (PHASE 11)"
expect "waveOutOpen=6" "audiotest.exe: winmm/waveOut (PHASE 12, NODRIVER ohne Audio)"
expect "x86_64 GNU/Linux" "Linux-Kernel (x86_64) läuft"
expect "login:" "Login-Prompt erreicht"

echo
if [ "$FAIL" -eq 0 ]; then
    ok "Systemtest bestanden ($ISO)"
    exit 0
else
    die "Systemtest fehlgeschlagen – Log: $BOOTLOG"
fi
