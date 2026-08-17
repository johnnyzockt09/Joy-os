#!/usr/bin/env bash
# Erzeugt ein modernes Joys-Wallpaper (dunkler Farbverlauf + dezentes Logo).
# Ergebnis: /usr/share/backgrounds/joys-wallpaper.png
set -euo pipefail

OUT="/usr/share/backgrounds/joys-wallpaper.png"
W=1920
H=1080

mkdir -p "$(dirname "$OUT")"

# Dunkler Blau-zu-Dunkelviolett-Verlauf + zentriertes, blasses Viereck-Symbol.
convert -size ${W}x${H} gradient:'#1b1b2f'-'#0f0f1a' \
    -fill 'rgba(61,126,255,0.12)' -draw "rectangle 0,0 $W,$H" \
    -fill 'rgba(255,255,255,0.10)' \
    -draw "roundrectangle $((W/2-180)),$((H/2-180)),$((W/2+180)),$((H/2+180)),40,40" \
    -fill 'rgba(255,255,255,0.15)' \
    -draw "roundrectangle $((W/2-140)),$((H/2-140)),$((W/2+140)),$((H/2+140)),32,32" \
    "$OUT"

echo "Wallpaper: $OUT"
