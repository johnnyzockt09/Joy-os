#!/usr/bin/env bash
# Erzeugt das minimale Windows-Testprogramm hello.exe (Test-Fixture für joys-win).
#
#   MSVC (Windows):  ./scripts/build-hello.sh
#   MinGW (Linux):   CC=x86_64-w64-mingw32-gcc ./scripts/build-hello.sh
#
# Ergebnis: compatibility/joys-win/tests/binaries/hello.exe
# Das hello.exe importiert ausschließlich kernel32!GetStdHandle/WriteFile/ExitProcess
# und benötigt KEIN CRT – ideal zum Testen des joys-win-Loaders.

set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/common.sh"

OUT="$ROOT_DIR/compatibility/joys-win/tests/binaries/hello.exe"
SRC="$ROOT_DIR/compatibility/joys-win/tests/binaries/hello.c"

mkdir -p "$(dirname "$OUT")"

if command -v "${CC:-x86_64-w64-mingw32-gcc}" >/dev/null 2>&1; then
    log "MinGW-Build: ${CC:-x86_64-w64-mingw32-gcc}"
    "${CC:-x86_64-w64-mingw32-gcc}" \
        -nostdlib \
        -Wl,-e,mainCRTStartup \
        -Wl,--subsystem,console \
        "$SRC" -lkernel32 -o "$OUT"
elif command -v cl >/dev/null 2>&1; then
    log "MSVC-Build (benötigt vcvars64-Umgebung)"
    cl /nologo /GS- /c "$SRC" /Fo:"$(dirname "$OUT")/hello.obj"
    link /nologo /SUBSYSTEM:CONSOLE /ENTRY:mainCRTStartup /NODEFAULTLIB \
        "$(dirname "$OUT")/hello.obj" kernel32.lib /OUT:"$OUT"
else
    die "Kein MinGW (x86_64-w64-mingw32-gcc) oder MSVC cl gefunden"
fi

ls -lh "$OUT"
ok "hello.exe erzeugt"
