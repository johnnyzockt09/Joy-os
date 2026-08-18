#!/usr/bin/env python3
# Joys Shell – moderne, Windows-11-artige Desktop-Leiste (GTK3).
# Zentrierte Taskbar, Startmenü mit Suche + App-Grid (Slide-Animation),
# Uhr, Systemmenü – dunkles Theme mit Hover-Transitions.
import gi
import os
import subprocess
import sys

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk, GLib, Pango
PANEL_HEIGHT = 54
BG = "#1b1b2f"
FG = "#e8e8f2"
ACCENT = "#3d7eff"

APPS = [
    ("Terminal",      "utilities-terminal", ["lxterminal"]),
    ("Joys Files",    "system-file-manager", ["pcmanfm"]),
    ("Einstellungen", "preferences-system", [sys.executable, "/usr/local/bin/joys-settings.py"]),
    ("Joys Store",    "system-software-install", [sys.executable, "/usr/local/bin/joys-store.py"]),
    ("joys-core",     "utilities-system-monitor", ["lxterminal", "-e", "/usr/bin/joys-core"]),
    ("joys-win",      "application-x-ms-dos-executable", ["lxterminal", "-e", "/usr/bin/joys-win"]),
    ("joys-update",   "software-update-available", ["lxterminal", "-e", "/usr/bin/joys-update"]),
    ("Joys Installer","drive-harddisk", [sys.executable, "/usr/local/bin/joys-installer.py"]),
    ("Joys Recovery", "system-run", [sys.executable, "/usr/local/bin/joys-recovery.py"]),
    ("Texteditor",    "accessories-text-editor", ["lxterminal", "-e", "/bin/nano"]),
]

CSS = f"""
window.panel {{ background-color: {BG}; }}
.panel-button {{
  background: transparent; color: {FG};
  font-size: 13px; border: none; border-radius: 8px;
  padding: 6px 14px; transition: 150ms ease;
}}
.panel-button:hover {{ background-color: rgba(255,255,255,0.08); }}
.panel-button:active {{ background-color: rgba(255,255,255,0.16);
                       background-image: none; }}
.panel-clock {{ color: {FG}; font-size: 13px; padding: 0 10px; }}
.start-menu {{
  background-color: rgba(24,24,42,0.97); color: {FG};
  border-radius: 16px; border: 1px solid rgba(255,255,255,0.10);
}}
.start-search {{
  background-color: rgba(255,255,255,0.07); color: {FG};
  border-radius: 8px; border: none; padding: 8px 12px; font-size: 13px;
  transition: 150ms ease;
}}
.start-search:focus {{ border: 1px solid {ACCENT}; }}
.app-card {{
  background: transparent; color: {FG}; border-radius: 12px;
  font-size: 12px; padding: 12px 6px; transition: 150ms ease;
}}
.app-card:hover {{ background-color: rgba(255,255,255,0.10); }}
.app-card:active {{ background-color: rgba(61,126,255,0.25); }}
.power-button {{
  background: transparent; color: {FG}; border-radius: 8px;
  font-size: 13px; padding: 8px 12px; transition: 150ms ease;
}}
.power-button:hover {{ background-color: rgba(255,255,255,0.10); }}
.start-title {{ color: {FG}; font-size: 16px; font-weight: bold; }}
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
        self.set_default_size(580, 500)
        self.get_style_context().add_class("start-menu")

        self.revealer = Gtk.Revealer()
        self.revealer.set_transition_type(Gtk.RevealerTransitionType.SLIDE_DOWN)
        self.revealer.set_transition_duration(220)
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        box.set_margin_top(20)
        box.set_margin_bottom(20)
        box.set_margin_left(20)
        box.set_margin_right(20)
        self.revealer.add(box)
        self.add(self.revealer)

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
        self.grid.set_row_spacing(4)
        self.grid.set_column_spacing(4)
        scroller = Gtk.ScrolledWindow()
        scroller.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        scroller.add(self.grid)
        box.pack_start(scroller, True, True, 0)

        sep = Gtk.Separator()
        box.pack_start(sep, False, False, 0)

        power = Gtk.Box(spacing=6)
        reboot = Gtk.Button(label="Neu starten")
        reboot.get_style_context().add_class("power-button")
        reboot.connect("clicked", lambda *_: (self.hide(), spawn(["systemctl", "reboot"])))
        poweroff = Gtk.Button(label="Ausschalten")
        poweroff.get_style_context().add_class("power-button")
        poweroff.connect("clicked", lambda *_: (self.hide(), spawn(["systemctl", "poweroff"])))
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
        ic = Gtk.Image(icon_name=icon, pixel_size=36)
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
                child.get_child().clicked()
                return

    def show_at(self, x, y):
        self.show_all()
        self.move(x, y - 508 - 10)
        self.revealer.set_reveal_child(True)


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
        self.set_default_size(screen.get_width(), PANEL_HEIGHT)
        self.move(0, screen.get_height() - PANEL_HEIGHT)

        bar = Gtk.Box(spacing=4)
        bar.set_margin_left(8)
        bar.set_margin_right(8)
        self.add(bar)

        start = Gtk.Button(label="Joys")
        start.get_style_context().add_class("panel-button")
        start.connect("clicked", self.toggle_menu)
        bar.pack_start(start, False, False, 0)

        bar.pack_start(Gtk.Label(), True, True, 0)

        center = Gtk.Box(spacing=4)
        for name, icon, cmd in APPS[:3]:
            b = Gtk.Button()
            b.get_style_context().add_class("panel-button")
            b.set_image(Gtk.Image(icon_name=icon, pixel_size=18))
            b.set_tooltip_text(name)
            b.connect("clicked", lambda *_, c=cmd: spawn(c))
            center.pack_start(b, False, False, 0)
        bar.pack_start(center, False, False, 0)

        # Laufende Fenster (EWMH) – Mitte der Taskbar.
        self.wins = Gtk.Box(spacing=4)
        bar.pack_start(self.wins, False, False, 0)

        bar.pack_start(Gtk.Label(), True, True, 0)

        self.clock = Gtk.Label(label="")
        self.clock.get_style_context().add_class("panel-clock")
        bar.pack_end(self.clock, False, False, 0)
        self.update_clock()
        GLib.timeout_add_seconds(1, self.update_clock)

        sysb = Gtk.Button()
        sysb.get_style_context().add_class("panel-button")
        sysb.set_image(Gtk.Image(icon_name="preferences-system-symbolic", pixel_size=18))
        sysb.set_tooltip_text("Quick Settings")
        sysb.connect("clicked", self.toggle_sysmenu)
        bar.pack_end(sysb, False, False, 0)

        self._start = start
        self._sysbtn = sysb
        self.connect("destroy", Gtk.main_quit)
        self.show_all()
        # Fenster-Switcher: offene Fenster periodisch aktualisieren.
        GLib.timeout_add_seconds(2, self.update_windows)

    def update_windows(self):
        # Offene Fenster via EWMH (_NET_CLIENT_LIST) ermitteln.
        try:
            out = subprocess.run(
                ["xprop", "-root", "_NET_CLIENT_LIST"],
                capture_output=True, text=True, timeout=3).stdout
        except Exception:
            return True
        win_ids = []
        if "_NET_CLIENT_LIST(WINDOW):" in out:
            rest = out.split("window id #", 1)[-1]
            for part in rest.split(","):
                ids = part.split()
                win_ids.extend(w for w in ids if w.startswith("0x"))
        # Aktuelle Buttons entfernen (nur bei Änderung, sonst einfach neu).
        for child in self.wins.get_children():
            self.wins.remove(child)
        for wid in win_ids[:10]:
            name = self._window_name(wid)
            b = Gtk.Button(label=name[:14])
            b.get_style_context().add_class("panel-button")
            b.set_tooltip_text(name)
            b.connect("clicked", lambda *_, w=wid: self._focus(w))
            self.wins.pack_start(b, False, False, 0)
        self.wins.show_all()
        return True

    def _window_name(self, wid):
        try:
            out = subprocess.run(
                ["xprop", "-id", wid, "_NET_WM_NAME"],
                capture_output=True, text=True, timeout=2).stdout
            if '"' in out:
                return out.split('"', 1)[1].rsplit('"', 1)[0]
        except Exception:
            pass
        return "Fenster"

    def _focus(self, wid):
        try:
            subprocess.run(["xdotool", "windowactivate", wid],
                           capture_output=True, timeout=2)
        except Exception:
            pass

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
        self._menu.show_at(x, y)

    def toggle_sysmenu(self, _b):
        if self._sysmenu is None:
            self._sysmenu = self._build_sysmenu()
        if self._sysmenu.get_visible():
            self._sysmenu.hide()
            return
        self._menu and self._menu.hide()
        self._sysmenu.show_all()
        ax, ay = self._sysbtn.get_window().get_origin()
        w = self._sysmenu.get_allocated_width() or 320
        h = self._sysmenu.get_allocated_height() or 300
        self._sysmenu.move(ax + self._sysbtn.get_allocated_width() - w,
                           ay - h)

    def _build_sysmenu(self):
        m = Gtk.Window(type=Gtk.WindowType.POPUP)
        m.set_decorated(False)
        m.get_style_context().add_class("start-menu")
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        box.set_margin_top(12)
        box.set_margin_bottom(12)
        box.set_margin_left(14)
        box.set_margin_right(14)
        m.add(box)

        # Notifications.
        notif = Gtk.Label(label="Notifications", xalign=0)
        notif.get_style_context().add_class("start-title")
        box.pack_start(notif, False, False, 0)
        for text in ("✓ Joys is up to date", "Network connected"):
            n = Gtk.Label(label=text, xalign=0)
            n.get_style_context().add_class("app-card")
            box.pack_start(n, False, False, 0)

        sep = Gtk.Separator()
        box.pack_start(sep, False, False, 0)

        qs = Gtk.Label(label="Quick Settings", xalign=0)
        qs.get_style_context().add_class("start-title")
        box.pack_start(qs, False, False, 0)

        def toggle_row(text, active):
            row = Gtk.Box(spacing=10)
            lbl = Gtk.Label(label=text, xalign=0)
            lbl.get_style_context().add_class("app-card")
            sw = Gtk.Switch()
            sw.set_active(active)
            row.pack_start(lbl, True, True, 0)
            row.pack_end(sw, False, False, 0)
            box.pack_start(row, False, False, 0)

        toggle_row("Wi-Fi", True)
        toggle_row("Bluetooth", False)

        volrow = Gtk.Box(spacing=10)
        vlab = Gtk.Label(label="Volume", xalign=0)
        vlab.get_style_context().add_class("app-card")
        vol = Gtk.Scale.new_with_range(Gtk.Orientation.HORIZONTAL, 0, 100, 5)
        vol.set_value(60)
        vol.set_size_request(140, -1)
        volrow.pack_start(vlab, True, True, 0)
        volrow.pack_end(vol, False, False, 0)
        box.pack_start(volrow, False, False, 0)

        sep2 = Gtk.Separator()
        box.pack_start(sep2, False, False, 0)

        for label, cmd in [
            ("Einstellungen", [sys.executable, "/usr/local/bin/joys-settings.py"]),
            ("Neu starten", ["systemctl", "reboot"]),
            ("Ausschalten", ["systemctl", "poweroff"]),
        ]:
            b = Gtk.Button(label=label)
            b.get_style_context().add_class("power-button")
            b.connect("clicked", lambda *_, c=cmd: (m.hide(), spawn(c)))
            box.pack_start(b, False, False, 0)
        return m


def main():
    css = Gtk.CssProvider()
    css.load_from_data(CSS.encode())
    Gtk.StyleContext.add_provider_for_screen(
        Gdk.Screen.get_default(), css, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
    Gtk.Settings.get_default().set_property("gtk-application-prefer-dark-theme", True)
    Panel()
    Gtk.main()


if __name__ == "__main__":
    main()
