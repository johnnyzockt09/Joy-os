#!/usr/bin/env bash
# Installiert den Joys Desktop (leichtgewichtig) in ein Root-Filesystem:
# Openbox (WM), lxpanel (Taskbar/Menü), pcmanfm (Dateimanager/Desktop-Icons),
# lxterminal (Terminal), lxappearance (Einstellungen).
# Autologin auf tty1 + startx -> Joys-Sitzung.
#
#   ./scripts/build-desktop.sh <rootfs>

set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/common.sh"

TARGET="${1:-$ROOTFS_DIR}"
[ -d "$TARGET/etc" ] || die "Rootfs fehlt: $TARGET"

require_root

log "Desktop-Pakete werden installiert (Openbox, lxpanel, pcmanfm, lxterminal)..."

mount --bind /proc "$TARGET/proc" 2>/dev/null || true
mount --bind /sys "$TARGET/sys" 2>/dev/null || true
mount --bind /dev "$TARGET/dev" 2>/dev/null || true

chroot "$TARGET" /bin/bash -c '
    set -e
    export DEBIAN_FRONTEND=noninteractive
    export LANG=C.UTF-8
    rm -f /etc/resolv.conf
    echo "nameserver 1.1.1.1" > /etc/resolv.conf
    # universe-Repo aktivieren (lxpanel/pcmanfm/lxterminal liegen dort).
    if [ -f /etc/apt/sources.list.d/ubuntu.sources ]; then
        sed -i "s/^Components: .*/Components: main universe/" /etc/apt/sources.list.d/ubuntu.sources
    elif [ -f /etc/apt/sources.list ]; then
        sed -i "s/ main$/ main universe/; s/ main / main universe /" /etc/apt/sources.list
    fi
    apt-get update -qq
    apt-get install -y -qq \
        xserver-xorg \
        xinit \
        x11-xserver-utils \
        openbox \
        lxpanel \
        pcmanfm \
        lxterminal \
        lxappearance \
        lxsession \
        hicolor-icon-theme \
        dbus-x11 \
        xdg-utils \
        fonts-dejavu-core \
        python3 \
        python3-gi \
        gir1.2-gtk-3.0 \
        rsync \
        parted \
        dosfstools \
        efibootmgr \
        grub-efi-amd64 \
        grub-efi-amd64-bin \
        os-prober \
    || true
    apt-get clean
    rm -f /etc/resolv.conf
' || warn "Desktop-Installation hatte Warnungen"

# --- Xorg: glamor/llvmpipe-Absturz in QEMU vermeiden ---
mkdir -p "$TARGET/etc/X11/xorg.conf.d"
cat > "$TARGET/etc/X11/xorg.conf.d/99-joys.conf" <<'EOF'
Section "Device"
    Identifier "JoysVGA"
    Driver "modesetting"
    Option "AccelMethod" "none"
    Option "ShadowFB" "true"
EndSection
EOF

# --- Autologin auf tty1 ---
mkdir -p "$TARGET/etc/systemd/system/getty@tty1.service.d"
cat > "$TARGET/etc/systemd/system/getty@tty1.service.d/autologin.conf" <<'EOF'
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin root --noclear tty1 linux
EOF

# --- .profile: auf tty1 automatisch startx ---
if ! grep -q "startx" "$TARGET/root/.profile" 2>/dev/null; then
    cat >> "$TARGET/root/.profile" <<'EOF'

# Joys Desktop automatisch auf tty1 starten.
if [ "$(tty)" = "/dev/tty1" ] && [ -z "$DISPLAY" ] && [ -x /usr/bin/startx ]; then
    exec startx
fi
EOF
fi

# --- Bracketed-Paste global deaktivieren (Serienkonsole/Test-Tools) ---
cat > "$TARGET/etc/inputrc" <<'EOF'
set enable-bracketed-paste off
EOF

# --- xinitrc -> Joys-Sitzung ---
install -m 0644 -D "$ROOT_DIR/packages/live/joys.desktop" \
    "$TARGET/usr/share/xsessions/joys.desktop"
install -m 0755 "$ROOT_DIR/packages/live/joys-session.sh" \
    "$TARGET/usr/local/bin/joys-session"
cat > "$TARGET/root/.xinitrc" <<'EOF'
exec /usr/local/bin/joys-session
EOF

# --- Joys Shell / Settings / Installer (modernes Design) ---
install -m 0755 "$ROOT_DIR/desktop/joys-shell/joys-shell.py" \
    "$TARGET/usr/local/bin/joys-shell.py"
install -m 0755 "$ROOT_DIR/desktop/joys-shell/joys-settings.py" \
    "$TARGET/usr/local/bin/joys-settings.py"
install -m 0755 "$ROOT_DIR/desktop/joys-shell/joys-installer.py" \
    "$TARGET/usr/local/bin/joys-installer.py"
install -m 0755 "$ROOT_DIR/scripts/joys-install.sh" \
    "$TARGET/usr/local/bin/joys-install.sh"

# Desktop-Einträge (Startmenü der Joys Shell).
cat > "$TARGET/usr/share/applications/joys-installer.desktop" <<'EOF'
[Desktop Entry]
Name=Joys Installer
Comment=Joys OS auf der Festplatte installieren
Exec=python3 /usr/local/bin/joys-installer.py
Icon=drive-harddisk
Type=Application
EOF
cat > "$TARGET/usr/share/applications/joys-settings.desktop" <<'EOF'
[Desktop Entry]
Name=Einstellungen
Comment=Joys Einstellungen
Exec=python3 /usr/local/bin/joys-settings.py
Icon=preferences-system
Type=Application
EOF

# --- Joys Executable Manager: Doppelklick auf .exe startet ohne Auswahl ---
install -m 0755 "$ROOT_DIR/desktop/joys-shell/joys-exe-manager.py" \
    "$TARGET/usr/local/bin/joys-exe-manager.py"
cat > "$TARGET/usr/share/applications/joys-exe.desktop" <<'EOF'
[Desktop Entry]
Name=Joys Windows-Programm
Comment=Windows-Programm über joys-win ausführen
Exec=python3 /usr/local/bin/joys-exe-manager.py %f
Type=Application
Terminal=false
MimeType=application/vnd.microsoft.portable-executable
Icon=application-x-ms-dos-executable
EOF
# .exe mit dem Manager assoziieren (zwei Pfade: MIME- und Endungs-Registrierung).
if [ -d "$TARGET/usr/share/application-registry" ]; then :; fi
mkdir -p "$TARGET/usr/share/mime/packages"
cat > "$TARGET/usr/share/mime/packages/application-x-ms-dos-executable.xml" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/vnd.microsoft.portable-executable">
    <comment>Windows executable</comment>
    <glob pattern="*.exe"/>
  </mime-type>
</mime-info>
EOF
cat > "$TARGET/usr/share/mime/globs2" <<'EOF'
50:application/vnd.microsoft.portable-executable:*.exe
EOF
cat > "$TARGET/usr/share/mime/mime.cache" <<'EOF'
EOF

# .exe-Verknüpfung in /root/.local/share, die pcmanfm/xdg-open verwendet.
mkdir -p "$TARGET/root/.local/share/applications"
cp "$TARGET/usr/share/applications/joys-exe.desktop" \
    "$TARGET/root/.local/share/applications/"
mkdir -p "$TARGET/root/.local/share/mime/packages"
cp "$TARGET/usr/share/mime/packages/application-x-ms-dos-executable.xml" \
    "$TARGET/root/.local/share/mime/packages/"

# lxde-file-manager / pcmanfm assoziiert Anwendungen über defaults.list.
cat > "$TARGET/root/.config/mimeapps.list" <<'EOF'
[Default Applications]
application/vnd.microsoft.portable-executable=joys-exe.desktop
EOF
mkdir -p "$TARGET/root/.config"
cp "$TARGET/root/.config/mimeapps.list" "$TARGET/root/.config/mimeapps.list" 2>/dev/null || true

# --- Openbox-Konfiguration (Startmenü, Tasten) ---
mkdir -p "$TARGET/root/.config/openbox"
install -m 0644 "$ROOT_DIR/packages/live/openbox-rc.xml" \
    "$TARGET/root/.config/openbox/rc.xml"
install -m 0644 "$ROOT_DIR/packages/live/openbox-menu.xml" \
    "$TARGET/root/.config/openbox/menu.xml"

# --- Netzwerk: systemd-networkd mit DHCP für kabelgebundene NICs ---
cat > "$TARGET/etc/systemd/network/20-wired.network" <<'EOF'
[Match]
Name=e* en*

[Network]
DHCP=yes
EOF
chroot "$TARGET" systemctl enable systemd-networkd 2>/dev/null || true
chroot "$TARGET" systemctl enable systemd-resolved 2>/dev/null || true

# --- Desktop-Diagnose-Service (verzögert, für QEMU-Tests) ---
install -m 0755 "$ROOT_DIR/packages/live/joys-desktop-diag.sh" \
    "$TARGET/usr/local/bin/joys-desktop-diag.sh"
install -m 0644 "$ROOT_DIR/packages/live/joys-desktop-diag.service" \
    "$TARGET/etc/systemd/system/joys-desktop-diag.service"
chroot "$TARGET" systemctl enable joys-desktop-diag.service 2>/dev/null || true

# --- lxpanel-Konfiguration (Taskbar mit Anwendungsmenü) ---
mkdir -p "$TARGET/root/.config/lxpanel/joys/panels"
cat > "$TARGET/root/.config/lxpanel/joys/panels/panel" <<'EOF'
Global {
  edge=bottom
  align=center
  margin=0
  widthtype=percent
  width=100
  height=30
  transparent=0
}
Plugin {
  type = menu
  Config {
    image = /usr/share/lxpanel/images/lxde_logo.png
    system {
    }
    separator {
    }
    item {
      command = run
    }
  }
}
Plugin {
  type = space
  Config {
    size = 4
  }
}
Plugin {
  type = launchbar
  Config {
    Button {
      id = pcmanfm.desktop
    }
    Button {
      id = lxterminal.desktop
    }
  }
}
Plugin {
  type = space
  Config {
    size = 4
  }
}
Plugin {
  type = taskbar
  Config {
    Icons = 1
    tooltips = 1
  }
}
Plugin {
  type = space
  Config {
    size = 4
  }
}
Plugin {
  type = clock
  Config {
    ClockFmt = %H:%M
    TooltipFmt = %A %d. %B %Y
  }
}
EOF
cat > "$TARGET/root/.config/lxpanel/joys/config" <<'EOF'
[Command]
Logout=systemctl poweroff
Reboot=systemctl reboot
Terminal=lxterminal
[ui]
background=#1a1a2e
EOF
mkdir -p "$TARGET/root/.config/lxpanel/LXDE/panels"
cat > "$TARGET/root/.config/lxpanel/LXDE/panels/panel" <<'EOF'
Global {
  edge=bottom
  align=center
  margin=0
  widthtype=percent
  width=100
  height=30
}
Plugin { type = menu }
Plugin { type = launchbar }
Plugin { type = taskbar }
Plugin { type = clock }
EOF

umount "$TARGET/proc" 2>/dev/null || true
umount "$TARGET/sys" 2>/dev/null || true
umount "$TARGET/dev" 2>/dev/null || true

ok "Desktop installiert (Openbox, lxpanel, pcmanfm, lxterminal, Autologin tty1)"
