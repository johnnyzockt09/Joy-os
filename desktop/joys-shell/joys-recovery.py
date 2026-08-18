#!/usr/bin/env python3
# Joys Recovery – grundlegender Wiederherstellungsmodus (GUI).
# Optionen: normal booten, Terminal, Bootloader reparieren, herunterfahren.
import gi
import subprocess
import sys

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk

BG = "#0f0f1a"
FG = "#e8e8f2"
ACCENT = "#3d7eff"

CSS = f"""
window {{ background-color: {BG}; }}
.title {{ color: {FG}; font-size: 30px; font-weight: bold; letter-spacing: 8px; }}
.sub {{ color: rgba(232,232,242,0.6); font-size: 13px; }}
.big {{ background: rgba(255,255,255,0.05); color: {FG}; border-radius: 12px;
       font-size: 14px; padding: 14px 28px; transition: 150ms ease; }}
.big:hover {{ background: rgba(61,126,255,0.28); }}
.small {{ background: transparent; color: rgba(232,232,242,0.7); border-radius: 8px;
         font-size: 12px; padding: 8px 14px; transition: 150ms ease; }}
.small:hover {{ background: rgba(255,255,255,0.10); color: {FG}; }}
.footer {{ color: rgba(232,232,242,0.35); font-size: 11px; }}
""".strip()


def run_in_terminal(cmd):
    subprocess.Popen(["lxterminal", "-e", cmd])


class Recovery(Gtk.Window):
    def __init__(self):
        super().__init__(title="Joys Recovery")
        self.set_default_size(640, 520)
        self.set_position(Gtk.WindowPosition.CENTER)

        outer = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16)
        outer.set_margin_top(34)
        outer.set_margin_bottom(34)
        outer.set_margin_left(40)
        outer.set_margin_right(40)
        self.add(outer)

        t = Gtk.Label(label="JOYS RECOVERY", xalign=0)
        t.get_style_context().add_class("title")
        outer.pack_start(t, False, False, 0)
        s = Gtk.Label(label="Repariere dein System oder starte neu.", xalign=0)
        s.get_style_context().add_class("sub")
        outer.pack_start(s, False, False, 0)

        outer.pack_start(Gtk.Label(), False, False, 8)

        def opt(label, cmd, terminal=False):
            b = Gtk.Button(label=label)
            b.get_style_context().add_class("big")
            b.connect("clicked", lambda *_: run_in_terminal(cmd) if terminal
                      else subprocess.Popen(cmd, shell=True))
            outer.pack_start(b, False, False, 6)

        opt("Boot normally", "systemctl reboot")
        opt("Safe Mode", "systemctl reboot")
        opt("Repair bootloader", "grub-install --target=x86_64-efi --efi-directory=/boot/efi --bootloader-id=Joys; update-grub", terminal=True)
        opt("Terminal", "bash", terminal=True)
        opt("Shutdown", "systemctl poweroff")

        foot = Gtk.Label(label="Manche Recovery-Optionen benötigen ein installiertes System.", xalign=0)
        foot.get_style_context().add_class("footer")
        outer.pack_start(foot, False, False, 0)

        self.connect("destroy", Gtk.main_quit)
        self.show_all()


def main():
    css = Gtk.CssProvider()
    css.load_from_data(CSS.encode())
    Gtk.StyleContext.add_provider_for_screen(
        Gdk.Screen.get_default(), css, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
    Gtk.Settings.get_default().set_property("gtk-application-prefer-dark-theme", True)
    Recovery()
    Gtk.main()


if __name__ == "__main__":
    main()
