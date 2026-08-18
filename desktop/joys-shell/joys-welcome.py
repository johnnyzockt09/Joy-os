#!/usr/bin/env python3
# Joys Welcome – professionelles Start-/Live-Menü (erscheint beim ISO-Boot).
# Vollbild mit Joys-Logo und den Optionen Try Live / Install / Recovery.
import gi
import subprocess
import sys

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk, Pango

BG = "#0f0f1a"
FG = "#e8e8f2"
ACCENT = "#3d7eff"

CSS = f"""
window {{ background-color: {BG}; }}
.logo {{ color: {FG}; font-size: 72px; font-weight: bold; letter-spacing: 14px; }}
.logo-glow {{ color: {ACCENT}; font-size: 16px; font-weight: bold; letter-spacing: 6px; }}
.tagline {{ color: rgba(232,232,242,0.6); font-size: 14px; }}
.big {{ background: rgba(255,255,255,0.05); color: {FG}; border-radius: 14px;
       font-size: 16px; padding: 16px 34px; transition: 150ms ease; }}
.big:hover {{ background: rgba(61,126,255,0.28); }}
.big:active {{ background: rgba(61,126,255,0.45); }}
.small {{ background: transparent; color: rgba(232,232,242,0.7); border-radius: 8px;
         font-size: 12px; padding: 8px 14px; transition: 150ms ease; }}
.small:hover {{ background: rgba(255,255,255,0.10); color: {FG}; }}
.footer {{ color: rgba(232,232,242,0.35); font-size: 11px; }}
""".strip()


class Welcome(Gtk.Window):
    def __init__(self):
        super().__init__(type=Gtk.WindowType.TOPLEVEL)
        self.set_decorated(False)
        self.fullscreen()
        self.get_style_context().add_class("welcome")

        outer = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        self.add(outer)

        # Vertikale Mitte.
        mid = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        mid.set_halign(Gtk.Align.CENTER)
        mid.set_valign(Gtk.Align.CENTER)
        outer.pack_start(mid, True, True, 0)

        logo = Gtk.Label(label="JOYS")
        logo.get_style_context().add_class("logo")
        mid.pack_start(logo, False, False, 0)

        glow = Gtk.Label(label="A  F A S T   &   M O D E R N   O S")
        glow.get_style_context().add_class("logo-glow")
        mid.pack_start(glow, False, False, 0)

        tag = Gtk.Label(label="Welcome to Joys OS")
        tag.get_style_context().add_class("tagline")
        mid.pack_start(tag, False, False, 8)

        mid.pack_start(Gtk.Label(), False, False, 6)

        # Hauptoptionen.
        for label, handler in [
            ("Try Joys Live", self.on_try),
            ("Install Joys", self.on_install),
            ("Recovery Mode", self.on_recovery),
        ]:
            b = Gtk.Button(label=label)
            b.get_style_context().add_class("big")
            b.connect("clicked", handler)
            mid.pack_start(b, False, False, 6)

        # Fußleiste: Reboot / Shutdown.
        foot = Gtk.Box(spacing=10)
        foot.set_halign(Gtk.Align.CENTER)
        foot.set_margin_bottom(30)
        for label, cmd in [("Reboot", ["systemctl", "reboot"]),
                           ("Shutdown", ["systemctl", "poweroff"])]:
            b = Gtk.Button(label=label)
            b.get_style_context().add_class("small")
            b.connect("clicked", lambda *_, c=cmd: subprocess.Popen(c))
            foot.pack_start(b, False, False, 0)
        outer.pack_start(foot, False, False, 0)

        ver = Gtk.Label(label="Joys OS 0.1.0")
        ver.get_style_context().add_class("footer")
        ver.set_margin_bottom(10)
        outer.pack_start(ver, False, False, 0)

        self.connect("destroy", Gtk.main_quit)
        self.show_all()

    def on_try(self, _b):
        self.destroy()

    def on_install(self, _b):
        self.destroy()
        subprocess.Popen([sys.executable, "/usr/local/bin/joys-installer.py"])

    def on_recovery(self, _b):
        self.destroy()
        subprocess.Popen([sys.executable, "/usr/local/bin/joys-recovery.py"])


def main():
    css = Gtk.CssProvider()
    css.load_from_data(CSS.encode())
    Gtk.StyleContext.add_provider_for_screen(
        Gdk.Screen.get_default(), css, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
    Gtk.Settings.get_default().set_property("gtk-application-prefer-dark-theme", True)
    Welcome()
    Gtk.main()


if __name__ == "__main__":
    main()
