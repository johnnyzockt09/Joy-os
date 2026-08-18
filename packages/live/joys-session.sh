#!/bin/sh
# Joys Shell – Desktop-Sitzung (leichtgewichtig, modern, kein GNOME/KDE).
#
#   openbox       -> Fenstermanager (leicht)
#   joys-shell    -> moderne, Windows-11-artige Taskbar + Startmenü
#   pcmanfm       -> Dateimanager + Desktop-Icons (Wallpaper)
#   picom         -> Kompositor (Transparenz/Schatten), falls installiert

# Modernes Joys-Wallpaper setzen (falls vorhanden).
if command -v feh >/dev/null 2>&1 && [ -f /usr/share/backgrounds/joys-wallpaper.png ]; then
    feh --bg-fill /usr/share/backgrounds/joys-wallpaper.png &
fi

openbox &

# Kompositor (Fade-Animationen/Schatten wie Win11) – falls picom installiert.
if command -v picom >/dev/null 2>&1 && [ -f /etc/xdg/picom-joys.conf ]; then
    picom --config /etc/xdg/picom-joys.conf -b 2>/dev/null || true
fi

python3 /usr/local/bin/joys-shell.py &

# Im Live-Modus (ISO) das professionelle Welcome-/Install-Menü zeigen.
# Nach einer Installation existiert /run/live/medium nicht mehr.
if [ -d /run/live/medium ] || [ -d /cdrom ] || [ -e /run/live/medium ]; then
    python3 /usr/local/bin/joys-welcome.py &
fi

pcmanfm --desktop --profile=joys &

# Warten, bis alle beendet werden (Sitzung beenden).
wait
