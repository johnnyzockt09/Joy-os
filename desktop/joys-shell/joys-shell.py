#!/usr/bin/env python3
# Joys Shell – moderne, Windows-11-artige Desktop-Leiste (GTK3).
# Zentrierte Taskbar, Startmenü mit Suche + App-Grid, Uhr, Systemmenü.
import gi
import os
import subprocess
import sys

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk, GLib, Pango

PANEL_HEIGHT = 52
BG = "#1b1b2f"
FG = "#e8e8f2"
ACCENT = "#3d7eff"

APPS = [
    ("Terminal",      "utilities-terminal", ["lxterminal"]),
    ("Dateimanager",  "system-file-manager", ["pcmanfm"]),
    ("Einstellungen", "preferences-system", [sys.executable, "/usr/local/bin/joys-settings.py"]),
    ("joys-core",     "utilities-system-monitor", ["lxterminal", "-e", "/usr/bin/joys-core"]),
    ("joys-win",      "application-x-ms-dos-executable", ["lxterminal", "-e", "/usr/bin/joys-win"]),
    ("Texteditor",    "accessories-text-editor", ["lxterminal", "-e", "/bin/nano"]),
]

CSS = f"""
window.panel {{ background-color: {BG}; }}
.panel-button {{
  background: transparent; color: {FG};
  font-size: 13px; border: none; border-radius: 8px;
  padding: 6px 14px;
}}
.panel-button:hover {{ background-color: rgba(255,255,255,0.08); }}
.panel-button:active {{ background-color: rgba(255,255,255,0.15); }}
.panel-clock {{ color: {FG}; font-size: 13px; padding: 0 10px; }}
.start-menu {{
  background-color: rgba(27,27,47,0.96); color: {FG};
  border-radius: 14px; border: 1px solid rgba(255,255,255,0.10);
}}
.start-search {{
  background-color: rgba(255,255,255,0.07); color: {FG};
  border-radius: 8px; border: none; padding: 8px 12px; font-size: 13px;
}}
.start-search:focus {{ border: 1px solid {ACCENT}; }}
.app-card {{
  background: transparent; color: {FG}; border-radius: 10px;
  font-size: 12px; padding: 10px 6px;
}}
.app-card:hover {{ background-color: rgba(255,255,255,0.10); }}
.power-button {{
  background: transparent; color: {FG}; border-radius: 8px;
  font-size: 13px; padding: 8px 12px;
}}
.power-button:hover {{ background-color: rgba(255,255,255,0.10); }}
.start-title {{ color: {FG}; font-size: 15px; font-weight: bold; }}
""".strip()


def spawn(argv):
    try:
        subprocess.Popen(argv, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except Exception:
        pass


class StartMenu(Gtk.Window):
    def __init__(self):
        super().__init__(type=Gtk.WindowType.POPUP)
        self.set_decorated(False)
        self.set_default_size(560, 480)
        self.get_style_context().add_class("start-menu")
        self.set_position(Gtk.WindowPosition.NONE)

        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        box.set_margin_top(20)
        box.set_margin_bottom(20)
        box.set_margin_left(20)
        box.set_margin_right(20)
        self.add(box)

        title = Gtk.Label(label="Joys")
        title.get_style_context().add_class("start-title")
        title.set_halign(Gtk.Align.START)
        box.pack_start(title, False, False, 0)

        self.search = Gtk.Entry()
        self.search.set_placeholder_text("Suche nach Anwendungen ...")
        self.search.get_style_context().add_class("start-search")
        self.search.set_icon_from_icon_name(Gtk.EntryIconPosition.PRIMARY, "system-search-symbolic")
        self.search.connect("changed", self.on_search)
        self.search.connect("activate", self.on_enter)
        box.pack_start(self.search, False, False, 0)

        self.grid = Gtk.FlowBox()
        self.grid.set_selection_mode(Gtk.SelectionMode.NONE)
        self.grid.set_max_children_per_line(4)
        self.grid.set_homogeneous(True)
        scroller = Gtk.ScrolledWindow()
        scroller.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        scroller.add(self.grid)
        box.pack_start(scroller, True, True, 0)

        sep = Gtk.Separator()
        box.pack_start(sep, False, False, 0)

        power = Gtk.Box(spacing=6)
        reboot = Gtk.Button(label="Neu starten")
        reboot.get_style_context().add_class("power-button")
        reboot.connect("clicked", lambda *_: spawn(["systemctl", "reboot"]))
        poweroff = Gtk.Button(label="Ausschalten")
        poweroff.get_style_context().add_class("power-button")
        poweroff.connect("clicked", lambda *_: spawn(["systemctl", "poweroff"]))
        power.pack_start(reboot, False, False, 0)
        power.pack_start(poweroff, False, False, 0)
        power.pack_start(Gtk.Label(), True, True, 0)
        box.pack_start(power, False, False, 0)

        for name, icon, cmd in APPS:
            self.add_app(name, icon, cmd)
        self._all = list(APPS)

    def add_app(self, name, icon, cmd):
        card = Gtk.Button()
        card.get_style_context().add_class("app-card")
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        ic = Gtk.Image(icon_name=icon, pixel_size=34)
        v.pack_start(ic, False, False, 0)
        lbl = Gtk.Label(label=name, ellipsize=Pango.EllipsizeMode.END)
        v.pack_start(lbl, False, False, 0)
        card.add(v)
        card.connect("clicked", lambda *_: (self.hide(), spawn(cmd)))
        self.grid.add(card)

    def on_search(self, _e):
        q = self.search.get_text().strip().lower()
        for child in self.grid.get_children():
            btn = child.get_child()
            name = btn.get_child().get_children()[1].get_text()
            child.set_visible(not q or q in name.lower())

    def on_enter(self, _e):
        for child in self.grid.get_children():
            if child.get_visible():
                btn = child.get_child()
                btn.clicked()
                return

    def show_at(self, x, y):
        self.show_all()
        w, h = self.get_size()
        self.move(x, y - h - 8)


class SystemMenu(Gtk.Window):
    def __init__(self, anchor):
        super().__init__(type=Gtk.WindowType.POPUP)
        self.set_decorated(False)
        self.get_style_context().add_class("start-menu")
        self._anchor = anchor
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        box.set_margin_top(10)
        box.set_margin_bottom=10
        box.set_margin_left=10
        box.set_margin_right=10
        self.add(box)
        for label, cmd in [
            ("Einstellungen", [sys.executable, "/usr/local/bin/joys-settings.py"]),
            ("Neu starten", ["systemctl", "reboot"]),
            ("Ausschalten", ["systemctl", "poweroff"]),
        ]:
            b = Gtk.Button(label=label)
            b.get_style_context().add_class("power-button")
            b.connect("clicked", lambda *_, c=cmd: (self.hide(), spawn(c)))
            box.pack_start(b, False, False, 0)

    def show_at_anchor(self):
        self.show_all()
        ax, ay = self._anchor.get_window().get_origin()
        w = self.get_allocated_width() or 200
        h = self.get_allocated_height() or 140
        self.move(ax + self._anchor.get_allocated_width() - w, ay - h)


class Panel(Gtk.Window):
    def __init__(self):
        super().__init__(type=Gtk.WindowType.TOPLEVEL)
        self.set_decorated(False)
        self.set_keep_below(True)
        self.set_skip_taskbar_hint(True)
        self.set_resizable(False)
        self.get_style_context().add_class("panel")
        self._menu = None
        self._sysmenu = None

        screen = self.get_screen()
        sw = screen.get_width()
        self.set_default_size(sw, PANEL_HEIGHT)
        self.set_position(Gtk.WindowPosition.NONE)
        self.move(0, screen.get_height() - PANEL_HEIGHT)

        bar = Gtk.Box(spacing=4)
        bar.set_margin_left(8)
        bar.set_margin_right(8)
        self.add(bar)

        # Start-Button
        start = Gtk.Button(label="Joys")
        start.get_style_context().add_class("panel-button")
        start.connect("clicked", self.toggle_menu)
        bar.pack_start(start, False, False, 0)

        bar.pack_start(Gtk.Label(), True, True, 0)

        # Zentrierte Launcher
        center = Gtk.Box(spacing=4)
        for name, icon, cmd in APPS[:3]:
            b = Gtk.Button()
            b.get_style_context().add_class("panel-button")
            b.set_image(Gtk.Image(icon_name=icon, pixel_size=18))
            b.set_tooltip_text(name)
            b.connect("clicked", lambda *_, c=cmd: spawn(c))
            center.pack_start(b, False, False, 0)
        bar.pack_start(center, False, False, 0)

        bar.pack_start(Gtk.Label(), True, True, 0)

        # Uhr
        self.clock = Gtk.Label(label="")
        self.clock.get_style_context().add_class("panel-clock")
        bar.pack_end(self.clock, False, False, 0)
        self.update_clock()
        GLib.timeout_add_seconds(1, self.update_clock)

        # Systemmenü
        sysb = Gtk.Button()
        sysb.get_style_context().add_class("panel-button")
        sysb.set_image(Gtk.Image(icon_name="system-shutdown-symbolic", pixel_size=18))
        sysb.set_tooltip_text("System")
        sysb.connect("clicked", self.toggle_sysmenu)
        bar.pack_end(sysb, False, False, 0)

        self._start = start
        self._sysbtn = sysb

        self.connect("destroy", Gtk.main_quit)
        self.show_all()

    def update_clock(self):
        now = __import__("datetime").datetime.now()
        self.clock.set_text(now.strftime("%H:%M"))
        self.clock.set_tooltip_text(now.strftime("%A, %d. %B %Y"))
        return True

    def toggle_menu(self, _b):
        if self._menu is not None and self._menu.get_visible():
            self._menu.hide()
            return
        if self._menu is None:
            self._menu = StartMenu()
        self._sysmenu and self._sysmenu.hide()
        x, y = self._start.get_window().get_origin()
        self._menu.show_at(x, y + self._start.get_allocated_height())

    def toggle_sysmenu(self, _b):
        if self._sysmenu is not None and self._sysmenu.get_visible():
            self._sysmenu.hide()
            return
        if self._sysmenu is None:
            self._sysmenu = SystemMenu(self._sysbtn)
        self._menu and self._menu.hide()
        self._sysmenu.show_at_anchor()


def main():
    css = Gtk.CssProvider()
    css.load_from_data(CSS.encode())
    Gtk.StyleContext.add_provider_for_screen(
        Gdk.Screen.get_default(), css, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
    Gtk.Settings.get_default().set_property("gtk-theme-name", "Adwaita")
    Gtk.Settings.get_default().set_property("gtk-application-prefer-dark-theme", True)
    Panel()
    Gtk.main()


if __name__ == "__main__":
    main()
