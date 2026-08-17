#!/bin/sh
# Joys Shell – Desktop-Sitzung (leichtgewichtig, modern, kein GNOME/KDE).
#
#   openbox       -> Fenstermanager (leicht)
#   joys-shell    -> moderne, Windows-11-artige Taskbar + Startmenü
#   pcmanfm       -> Dateimanager + Desktop-Icons

set -e

# Joys-Branding im Hintergrund (dunkles Blau).
if command -v xsetroot >/dev/null 2>&1; then
    xsetroot -solid "#1b1b2f"
fi

openbox &
python3 /usr/local/bin/joys-shell.py &
pcmanfm --desktop --profile=joys &

# Warten, bis alle beendet werden (Sitzung beenden).
wait
