#!/usr/bin/env bash
# Installiert den Joys-Plymouth-Bootscreen (Thema + Konfiguration).
set -euo pipefail

TARGET="${1:?rootfs}"
THEME="$TARGET/usr/share/plymouth/themes/joys"
mkdir -p "$THEME"

# Bootscreen-Animation: vier Punkte, die aufleuchten + "Starting Joys...".
cat > "$THEME/joys.plymouth" <<'EOF'
[Plymouth Theme]
Name=Joys
Description=Joys OS Bootscreen
ModuleName=script
[script]
ImageDir=/usr/share/plymouth/themes/joys
ScriptFile=/usr/share/plymouth/themes/joys/joys.script
EOF

# Einfache Pixelgrafiken (2x2 Punkte als PNG via printf - minimale Validierung).
# Punkte: 0 = dunkel, 1 = hell. Wir nutzen stattdessen reine Skript-Zeichnung.
cat > "$THEME/joys.script" <<'EOF'
// Joys Bootscreen – "JOYS" + 4 Punkte + "Starting Joys..."
window.SetBackgroundColor(0.06, 0.06, 0.10);

fun joystext (x, y, r, g, b) {
    text_color = window.NewGradientTexture (0, 0, 0, 0);
}

// Punkte in einer Reihe aufleuchten lassen (Animation).
dots_x = Window.GetWidth() / 2 - 60;
dots_y = Window.GetHeight() / 2 - 10;
radius = 7.0;
spacing = 40.0;

for (i = 0; i < 4; i++) {
    for (s = 0; s < 1; s++) { }
}

// Zentrales JOYS-Logo per Text-Texture (Fallback: großer Punkt).
logo = Window.NewTextTexture ("JOYS");
logo.SetPosition (Window.GetWidth() / 2 - 120, Window.GetHeight() / 2 - 90);
logo.SetColor (0.91, 0.91, 0.95, 1.0);
logo.SetScale (3.0, 3.0);
logo.Show ();

// Vier animierte Punkte.
for (i = 0; i < 4; i++) {
    dot[i] = Window.NewRectangle (radius * 2, radius * 2);
    dot[i].SetPosition (dots_x + i * spacing, dots_y);
    dot[i].SetColor (0.24, 0.49, 1.0, 1.0);
    dot[i].Show ();
    dot[i].SetOpacity (0.2);
}

// "Starting Joys..." Text.
text = Window.NewTextTexture ("Starting Joys...");
text.SetPosition (Window.GetWidth() / 2 - 120, dots_y + 30);
text.SetColor (0.6, 0.6, 0.7, 1.0);
text.Show ();

// Pulse-Animation der Punkte (nacheinander aufleuchten).
counter = 0;
fun pulse () {
    for (i = 0; i < 4; i++) {
        k = (counter + i) % 4;
        dot[i].SetOpacity (0.2 + 0.8 * (4 - (counter + i + 4) % 4) / 4.0);
    }
    counter = (counter + 1) % 4;
    return (100);
}
SetTimeout (pulse, 100);
EOF

# Theme aktivieren.
if [ -x "$TARGET/usr/sbin/plymouth-set-default-theme" ]; then
    chroot "$TARGET" /usr/bin/env bash -c '
        export PATH=/usr/sbin:/usr/bin:/bin
        plymouth-set-default-theme joys || true
    ' 2>/dev/null || true
fi

# cmdline: Plymouth an den Konsolen zeigen (falls gesetzt).
mkdir -p "$TARGET/etc/plymouth"
cat > "$TARGET/etc/plymouth/plymouthd.conf" <<'EOF'
[Daemon]
Theme=joys
ShowDelay=0
EOF

echo "Plymouth-Joys-Theme installiert"
