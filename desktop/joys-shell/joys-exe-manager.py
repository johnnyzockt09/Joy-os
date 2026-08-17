#!/usr/bin/env python3
# Joys Executable Manager – startet eine .exe doppelklickbar ohne Auswahl.
#
# Wird als Handler für .exe registriert (xdg-open). Startet die Windows-App
# automatisch über joys-win und zeigt den Konsolen-Output in einem kleinen
# Fenster an, wenn es kein GUI-Programm ist.
import gi
import os
import subprocess
import sys
import threading

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk, GLib

BG = "#1b1b2f"
FG = "#e8e8f2"
ACCENT = "#3d7eff"
CSS = f"""
window {{ background-color: {BG}; }}
.title {{ color: {FG}; font-size: 16px; font-weight: bold; }}
.console {{ background-color: #10101c; color: #b8ffb8; font-family: monospace;
           font-size: 12px; }}
.sub {{ color: {FG}; font-size: 12px; }}
""".strip()


def run_helper(exe):
    """Startet die .exe in einem Terminal, falls die Shell selbst läuft.
    Vom joys-shell (GTK) aufgerufen, damit man den Output sieht."""
    subprocess.Popen(
        ["lxterminal", "-e", f"/usr/bin/joys-win run '{exe}'"],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


class ExeOutput(Gtk.Window):
    """Zeigt den Konsolen-Output einer .exe an (falls CUI-Subsystem)."""

    def __init__(self, exe, proc):
        super().__init__(title=f"joys-win – {os.path.basename(exe)}")
        self.set_default_size(640, 420)
        self.set_position(Gtk.WindowPosition.CENTER)
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        box.set_margin_top(14)
        box.set_margin_bottom=14
        box.set_margin_start=14
        box.set_margin_end=14
        self.add(box)

        title = Gtk.Label(label=f"joys-win führt aus: {os.path.basename(exe)}", xalign=0)
        title.get_style_context().add_class("title")
        box.pack_start(title, False, False, 0)

        self.tv = Gtk.TextView()
        self.tv.set_editable(False)
        self.tv.get_style_context().add_class("console")
        sc = Gtk.ScrolledWindow()
        sc.set_min_content_height(320)
        sc.add(self.tv)
        box.pack_start(sc, True, True, 0)

        self._proc = proc
        self._buf = self.tv.get_buffer()
        threading.Thread(target=self._reader, daemon=True).start()

        self.connect("destroy", Gtk.main_quit)
        self.show_all()

    def _reader(self):
        for chunk in iter(lambda: self._proc.stdout.read(2048), b""):
            text = chunk.decode("utf-8", "replace")
            Gdk.threads_add_idle(GLib.PRIORITY_DEFAULT_IDLE,
                                 self._append, text)
        Gdk.threads_add_idle(GLib.PRIORITY_DEFAULT_IDLE, self._append,
                             "\n[Prozess beendet]\n")

    def _append(self, text):
        self._buf.insert(self._buf.get_end_iter(), text)
        return False


def main():
    args = sys.argv[1:]
    if not args:
        print("Verwendung: joys-exe-manager <datei.exe>", file=sys.stderr)
        sys.exit(2)
    exe = args[0]
    # Nur echte Dateien starten, nichts willkürliches.
    if not (exe.lower().endswith(".exe") and os.path.isfile(exe)):
        print(f"Kein gültiges .exe: {exe}", file=sys.stderr)
        sys.exit(3)

    css = Gtk.CssProvider()
    css.load_from_data(CSS.encode())
    Gtk.StyleContext.add_provider_for_screen(
        Gdk.Screen.get_default(), css, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)

    proc = subprocess.Popen(
        ["/usr/bin/joys-win", "run", exe],
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1)

    # Fenster nur anzeigen, wenn das Programm nicht sofort beendet.
    ExeOutput(exe, proc)
    Gtk.main()


if __name__ == "__main__":
    main()
