#!/usr/bin/env bash
# Gemeinsame Hilfsfunktionen für die Joys-Build-Skripte.
# Wird mit `source scripts/lib/common.sh` geladen.

set -euo pipefail

# Projekt-Root bestimmen (dieses Skript liegt unter $ROOT/scripts/lib/).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Zentrale Versionsquelle: VERSION-Datei.
VERSION="$(tr -d '[:space:]' < "$ROOT_DIR/VERSION")"

ARCH="${ARCH:-x86_64}"
PROFILE="${PROFILE:-minimal}"

BUILD_DIR="$ROOT_DIR/build"
DIST_DIR="$ROOT_DIR/dist"
WORK_DIR="$BUILD_DIR/work"
ROOTFS_DIR="$BUILD_DIR/rootfs-$PROFILE"

ISO_FILE="$DIST_DIR/Joys-$VERSION-$ARCH.iso"
SHA_FILE="$DIST_DIR/SHA256SUMS"

# ---- Ausgabe ----
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
    C_GREEN=$'\033[32m'; C_YELLOW=$'\033[33m'; C_RED=$'\033[31m'; C_RESET=$'\033[0m'
else
    C_GREEN=""; C_YELLOW=""; C_RED=""; C_RESET=""
fi

log()  { echo "[joys] $*"; }
ok()   { echo "${C_GREEN}[joys] OK:${C_RESET} $*"; }
warn() { echo "${C_YELLOW}[joys] WARN:${C_RESET} $*" >&2; }
die()  { echo "${C_RED}[joys] FEHLER:${C_RESET} $*" >&2; exit 1; }

# ---- Werkzeugprüfung ----
require_cmd() {
    command -v "$1" >/dev/null 2>&1 || die "Werkzeug '$1' ist nicht installiert (apt install ${1})"
}

require_root() {
    [ "$(id -u)" -eq 0 ] || die "Dieses Skript benötigt root (debootstrap). Bitte mit sudo ausführen."
}

# ---- Versions-/Umgebungsinfos ----
joys_version() { echo "$VERSION"; }
joys_iso_name() { echo "Joys-$VERSION-$ARCH.iso"; }

# Joys-Arch -> Debian-Arch für debootstrap.
deb_arch() {
    case "$ARCH" in
        x86_64)  echo amd64 ;;
        aarch64) echo arm64 ;;
        i386)    echo i386 ;;
        *) die "Unbekannte Architektur für debootstrap: $ARCH" ;;
    esac
}
