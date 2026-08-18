#!/usr/bin/env python3
# Joys Einstellungen – professionelles Einstellungsfenster mit Kategorien.
import gi
import subprocess

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk

BG = "#1b1b2f"
FG = "#e8e8f2"
ACCENT = "#3d7eff"
CSS = f"""
window {{ background-color: {BG}; }}
.sidebar {{ background-color: rgba(0,0,0,0.18); }}
.nav {{ background: transparent; color: {FG}; border: none; border-radius: 8px;
       padding: 9px 14px; font-size: 13px; transition: 150ms ease; }}
.nav:hover {{ background-color: rgba(255,255,255,0.08); }}
.nav:checked {{ background-color: {ACCENT}; color: white; }}
.card {{ background-color: rgba(255,255,255,0.06); border-radius: 14px; }}
.title {{ color: {FG}; font-size: 20px; font-weight: bold; }}
.field {{ color: {FG}; font-size: 13px; }}
.label-dim {{ color: rgba(232,232,242,0.6); font-size: 12px; }}
.switch-label {{ color: {FG}; font-size: 13px; }}
""".strip()


def shell(cmd):
    try:
        return subprocess.run(cmd, shell=True, capture_output=True, text=True).stdout.strip()
    except Exception:
        return "?"


class Card(Gtk.Box):
    def __init__(self, label, value):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=3)
        self.get_style_context().add_class("card")
        self.set_margin_top(8)
        self.set_margin_bottom(8)
        self.set_margin_start(14)
        self.set_margin_end(14)
        lbl = Gtk.Label(label=label, xalign=0)
        lbl.get_style_context().add_class("label-dim")
        self._val = Gtk.Label(label=value, xalign=0, wrap=True)
        self._val.get_style_context().add_class("field")
        self.pack_start(lbl, False, False, 0)
        self.pack_start(self._val, False, False, 0)

    def set(self, value):
        self._val.set_text(value)


class Row(Gtk.Box):
    """Toggle-Zeile."""
    def __init__(self, label, active=True):
        super().__init__(spacing=10)
        self.get_style_context().add_class("card")
        self.set_margin_top(8)
        self.set_margin_bottom(8)
        self.set_margin_start(14)
        self.set_margin_end(14)
        lbl = Gtk.Label(label=label, xalign=0)
        lbl.get_style_context().add_class("switch-label")
        self.switch = Gtk.Switch()
        self.switch.set_active(active)
        self.pack_start(lbl, True, True, 0)
        self.pack_end(self.switch, False, False, 0)


class Settings(Gtk.Window):
    def __init__(self):
        super().__init__(title="Joys Einstellungen")
        self.set_default_size(840, 560)
        self.set_position(Gtk.WindowPosition.CENTER)

        main = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        self.add(main)

        sidebar = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=3)
        sidebar.get_style_context().add_class("sidebar")
        sidebar.set_size_request(230, -1)
        title = Gtk.Label(label="Joys", xalign=0)
        title.get_style_context().add_class("title")
        title.set_margin_top(18)
        title.set_margin_bottom(10)
        title.set_margin_start(18)
        sidebar.pack_start(title, False, False, 0)
        self._nav_buttons = []
        self._sidebar = sidebar
        main.pack_start(sidebar, False, False, 0)

        self.content = Gtk.Stack()
        self.content.set_transition_type(Gtk.StackTransitionType.CROSSFADE)
        self.content.set_transition_duration(180)
        main.pack_start(self.content, True, True, 0)

        self._nav("Personalization", self._page_personalization())
        self._nav("Performance", self._page_performance())
        self._nav("System", self._page_system())
        self._nav("Windows", self._page_windows())
        self._nav("Updates", self._page_updates())
        self._nav("About", self._page_about())

        self._nav_buttons[0].set_active(True)
        self.connect("destroy", Gtk.main_quit)
        self.show_all()

    def _nav(self, name, page):
        b = Gtk.ToggleButton(label=name)
        b.get_style_context().add_class("nav")
        b.set_name(name)
        b.connect("toggled", self._on_nav)
        b.set_margin_start(10)
        b.set_margin_end(10)
        b.set_margin_top(2)
        b.set_margin_bottom(2)
        self.content.add_named(page, name)
        self._sidebar.pack_start(b, False, False, 0)
        self._nav_buttons.append(b)

    def _on_nav(self, btn):
        if btn.get_active():
            self.content.set_visible_child_name(btn.get_name())
            for b in self._nav_buttons:
                if b is not btn:
                    b.set_active(False)

    def _wrap(self, page, title_text):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        v.set_margin_top(24)
        v.set_margin_bottom(24)
        v.set_margin_start=30
        v.set_margin_end=30
        t = Gtk.Label(label=title_text, xalign=0)
        t.get_style_context().add_class("title")
        v.pack_start(t, False, False, 0)
        v.pack_start(page, True, True, 0)
        return v

    def _page_personalization(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        dark = Row("Dunkles Design", True)
        v.pack_start(dark, False, False, 0)
        anim = Row("Animationen", True)
        v.pack_start(anim, False, False, 0)

        acc = Gtk.Box(spacing=10)
        acc.get_style_context().add_class("card")
        acc.set_margin_top(8)
        acc.set_margin_bottom(8)
        acc.set_margin_start(14)
        acc.set_margin_end(14)
        al = Gtk.Label(label="Akzentfarbe", xalign=0)
        al.get_style_context().add_class("switch-label")
        combo = Gtk.ComboBoxText()
        for c in ["Blau", "Grün", "Orange", "Lila"]:
            combo.append_text(c)
        combo.set_active(0)
        acc.pack_start(al, True, True, 0)
        acc.pack_end(combo, False, False, 0)
        v.pack_start(acc, False, False, 0)

        wl = Gtk.Label(label="Wallpaper: Joys Standard", xalign=0)
        wl.get_style_context().add_class("label-dim")
        v.pack_start(wl, False, False, 0)
        return self._wrap(v, "Personalization")

    def _page_performance(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        lbl = Gtk.Label(label="Performance Mode", xalign=0)
        lbl.get_style_context().add_class("field")
        v.pack_start(lbl, False, False, 0)
        modes = Gtk.ListBox()
        modes.get_style_context().add_class("card")
        for m in ["Power Saving", "Balanced", "Performance", "Low RAM"]:
            r = Gtk.ListBoxRow()
            b = Gtk.Label(label=m, xalign=0)
            b.get_style_context().add_class("switch-label")
            b.set_margin_start(16)
            b.set_margin_top(10)
            b.set_margin_bottom(10)
            r.add(b)
            modes.add(r)
        modes.select_row(modes.get_row_at_index(1))
        v.pack_start(modes, False, False, 0)

        note = Gtk.Label(label="Bei ≤ 2 GB RAM wird automatisch ein reduzierter "
                               "UI-Modus (weniger Effekte) verwendet.", xalign=0, wrap=True)
        note.get_style_context().add_class("label-dim")
        v.pack_start(note, False, False, 0)
        return self._wrap(v, "Performance")

    def _page_system(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.cpu = Card("CPU", "?")
        self.ram = Card("Arbeitsspeicher", "?")
        self.disk = Card("Festplatte", "?")
        self.upt = Card("Laufzeit", "?")
        for c in (self.cpu, self.ram, self.disk, self.upt):
            v.pack_start(c, False, False, 0)
        refresh = Gtk.Button(label="Aktualisieren")
        refresh.connect("clicked", self.refresh_system)
        v.pack_start(refresh, False, False, 0)
        self.refresh_system()
        return self._wrap(v, "System")

    def _page_windows(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        card.get_style_context().add_class("card")
        card.set_margin_top(10)
        card.set_margin_bottom(10)
        card.set_margin_start(14)
        card.set_margin_end(14)
        t = Gtk.Label(label="joys-win (Windows-Kompatibilität)", xalign=0)
        t.get_style_context().add_class("field")
        card.pack_start(t, False, False, 0)
        s = Gtk.Label(label=".exe-Programme laufen direkt über joys-win – ohne Wine.\n"
                            "21 kernel32-, user32-, gdi32-, ws2_32- und winmm-Funktionen aktiv.",
                      xalign=0, wrap=True)
        s.get_style_context().add_class("label-dim")
        card.pack_start(s, False, False, 0)
        v.pack_start(card, False, False, 0)
        run = Gtk.Button(label="hello.exe testen")
        run.connect("clicked", lambda *_: subprocess.Popen(
            ["lxterminal", "-e", "/usr/bin/joys-win run /root/hello.exe"]))
        v.pack_start(run, False, False, 0)
        return self._wrap(v, "Windows Compatibility")

    def _page_updates(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        ver = Card("Installierte Version", "0.1.0")
        v.pack_start(ver, False, False, 0)
        btn = Gtk.Button(label="Nach Updates suchen")
        btn.connect("clicked", lambda *_: subprocess.Popen(
            ["lxterminal", "-e", "/usr/bin/joys-update --check JohnnyZockt09/Joy-os"]))
        v.pack_start(btn, False, False, 0)
        return self._wrap(v, "Updates")

    def _page_about(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        logo = Gtk.Label(label="JOYS", xalign=0)
        logo.get_style_context().add_class("title")
        v.pack_start(logo, False, False, 0)
        for label, val in [
            ("Version", "0.1.0"),
            ("Kernel", shell("uname -r") or "?"),
            ("Architektur", shell("uname -m") or "?"),
            ("Hostname", shell("hostname") or "?"),
            ("A fast, modern operating system.", ""),
        ]:
            if val:
                v.pack_start(Card(label, val), False, False, 0)
            else:
                t = Gtk.Label(label=label, xalign=0)
                t.get_style_context().add_class("label-dim")
                v.pack_start(t, False, False, 0)
        return self._wrap(v, "About")

    def refresh_system(self, *_):
        nproc = shell("nproc") or "?"
        total = shell("grep MemTotal /proc/meminfo | awk '{print int($2/1024)}'") or "?"
        avail = shell("grep MemAvailable /proc/meminfo | awk '{print int($2/1024)}'") or "?"
        disk = shell("df -h / | awk 'NR==2{print $3\"/\"$2\" frei \"$4}'") or "?"
        uptime = shell("uptime -p") or "?"
        self.cpu.set(f"{nproc} Kerne")
        self.ram.set(f"{total} MB gesamt, {avail} MB verfügbar")
        self.disk.set(disk)
        self.upt.set(uptime)


def main():
    css = Gtk.CssProvider()
    css.load_from_data(CSS.encode())
    Gtk.StyleContext.add_provider_for_screen(
        Gdk.Screen.get_default(), css, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
    Gtk.Settings.get_default().set_property("gtk-application-prefer-dark-theme", True)
    Settings()
    Gtk.main()


if __name__ == "__main__":
    main()
