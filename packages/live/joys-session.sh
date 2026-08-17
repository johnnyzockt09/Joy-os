#!/bin/sh
# Joys Shell – Desktop-Sitzung (leichtgewichtig, kein GNOME/KDE).
#
#   openbox    -> Fenstermanager (leicht)
#   lxpanel    -> Taskbar + Anwendungsmenü
#   pcmanfm    -> Dateimanager + Desktop-Icons
#   lxappearance -> Einstellungen (Erscheinungsbild)
#
# Aufruf über /root/.xinitrc (startx auf tty1).

set -e

# Joys-Branding im Hintergrund (einfaches, dunkles Blau).
if command -v xsetroot >/dev/null 2>&1; then
    xsetroot -solid "#1a1a2e"
fi

openbox &
lxpanel &
pcmanfm --desktop --profile=joys &

# Warten, bis alle beendet werden (Sitzung beenden).
wait
