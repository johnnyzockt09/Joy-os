#!/usr/bin/env python3
# Joys Einstellungen – modernes Einstellungsfenster (GTK3).
import gi
import subprocess

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk

BG = "#1b1b2f"
FG = "#e8e8f2"
ACCENT = "#3d7eff"
CSS = f"""
window {{ background-color: {BG}; }}
.sidebar {{ background-color: rgba(0,0,0,0.15); }}
.nav {{ background: transparent; color: {FG}; border: none; border-radius: 8px;
       padding: 10px 16px; font-size: 13px; }}
.nav:hover {{ background-color: rgba(255,255,255,0.08); }}
.nav:checked {{ background-color: {ACCENT}; color: white; }}
.card {{ background-color: rgba(255,255,255,0.06); border-radius: 12px; }}
.title {{ color: {FG}; font-size: 20px; font-weight: bold; }}
.field {{ color: {FG}; font-size: 13px; }}
.label-dim {{ color: rgba(232,232,242,0.6); font-size: 12px; }}
""".strip()


def shell(cmd):
    try:
        return subprocess.run(cmd, shell=True, capture_output=True, text=True).stdout.strip()
    except Exception:
        return "?"


class InfoCard(Gtk.Box):
    def __init__(self, label, value):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        self.get_style_context().add_class("card")
        self.set_margin_top(10)
        self.set_margin_bottom(10)
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


class Settings(Gtk.Window):
    def __init__(self):
        super().__init__(title="Joys Einstellungen")
        self.set_default_size(760, 500)
        self.set_position(Gtk.WindowPosition.CENTER)

        main = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=0)
        self.add(main)

        sidebar = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        sidebar.get_style_context().add_class("sidebar")
        sidebar.set_size_request(210, -1)
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
        main.pack_start(self.content, True, True, 0)

        self._nav("Über", self._page_about())
        self._nav("Design", self._page_design())
        self._nav("System", self._page_system())

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

    def _wrap(self, page):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
        v.set_margin_top(24)
        v.set_margin_bottom(24)
        v.set_margin_start(30)
        v.set_margin_end(30)
        v.pack_start(page, False, False, 0)
        return v

    def _page_about(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        t = Gtk.Label(label="Joys OS", xalign=0)
        t.get_style_context().add_class("title")
        v.pack_start(t, False, False, 0)
        for label, val in [
            ("Version", "0.1.0"),
            ("Architektur", shell("uname -m") or "?"),
            ("Kernel", shell("uname -r") or "?"),
            ("Hostname", shell("hostname") or "?"),
        ]:
            v.pack_start(InfoCard(label, val), False, False, 0)
        return self._wrap(v)

    def _page_design(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        t = Gtk.Label(label="Design", xalign=0)
        t.get_style_context().add_class("title")
        v.pack_start(t, False, False, 0)
        row = Gtk.Box(spacing=12)
        lbl = Gtk.Label(label="Dunkles Design")
        lbl.get_style_context().add_class("field")
        switch = Gtk.Switch()
        switch.set_active(True)
        row.pack_start(lbl, False, False, 0)
        row.pack_start(switch, False, False, 0)
        v.pack_start(row, False, False, 0)
        note = Gtk.Label(label="Joys verwendet ein modernes, dunkles "
                               "Windows-11-artiges Design.", xalign=0, wrap=True)
        note.get_style_context().add_class("label-dim")
        v.pack_start(note, False, False, 0)
        return self._wrap(v)

    def _page_system(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        t = Gtk.Label(label="System", xalign=0)
        t.get_style_context().add_class("title")
        v.pack_start(t, False, False, 0)
        self.cpu = InfoCard("CPU", "?")
        self.ram = InfoCard("Arbeitsspeicher", "?")
        self.disk = InfoCard("Festplatte", "?")
        self.upt = InfoCard("Laufzeit", "?")
        for c in (self.cpu, self.ram, self.disk, self.upt):
            v.pack_start(c, False, False, 0)
        refresh = Gtk.Button(label="Aktualisieren")
        refresh.connect("clicked", self.refresh_system)
        v.pack_start(refresh, False, False, 0)
        self.refresh_system()
        return self._wrap(v)

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
