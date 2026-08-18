#!/usr/bin/env python3
# Joys Store – moderner App-Store (Basis). Kategorien + bekannte Pakete.
# Installation via apt (falls verfügbar) – ehrlich: nicht installierte
# Pakete sind mit "Install" markiert; unbekannte nur "Coming Soon".
import gi
import subprocess

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk

BG = "#1b1b2f"
FG = "#e8e8f2"
ACCENT = "#3d7eff"

CSS = f"""
window {{ background-color: {BG}; }}
.title {{ color: {FG}; font-size: 24px; font-weight: bold; }}
.sub {{ color: rgba(232,232,242,0.6); font-size: 13px; }}
.cat {{ background: transparent; color: {FG}; border-radius: 10px;
       padding: 10px 16px; font-size: 13px; transition: 150ms ease; }}
.cat:hover {{ background: rgba(255,255,255,0.08); }}
.cat:checked {{ background: {ACCENT}; color: white; }}
.card {{ background: rgba(255,255,255,0.05); border-radius: 14px;
        padding: 14px; transition: 150ms ease; }}
.card:hover {{ background: rgba(61,126,255,0.18); }}
.card-title {{ color: {FG}; font-size: 15px; font-weight: bold; }}
.card-desc {{ color: rgba(232,232,242,0.6); font-size: 12px; }}
.install {{ background: {ACCENT}; color: white; border-radius: 8px;
          font-size: 13px; padding: 8px 18px; transition: 150ms ease; }}
.install:hover {{ background: #5a94ff; }}
.soon {{ background: rgba(255,255,255,0.08); color: rgba(232,232,242,0.8);
        border-radius: 8px; font-size: 13px; padding: 8px 18px; }}
.search {{ background: rgba(255,255,255,0.07); color: {FG}; border-radius: 8px;
          border: none; padding: 8px 12px; font-size: 13px; }}
""".strip()

# (name, kategorie, beschreibung, apt-paket oder None)
APPS = [
    ("Terminal",       "System",  "Kommandozeile",            "lxterminal"),
    ("Dateimanager",   "System",  "Dateien verwalten",        "pcmanfm"),
    ("Firefox",        "Internet", "Web-Browser",             "firefox"),
    ("LibreOffice",    "Büro",   "Office-Suite",             "libreoffice-writer"),
    ("GIMP",           "Multimedia", "Bildbearbeitung",       "gimp"),
    ("VLC",            "Multimedia", "Video-Player",          "vlc"),
    ("Steam",          "Spiele", "Gaming-Plattform",         None),
    ("Codeblocks",     "Entwicklung", "C/C++-IDE",           "codeblocks"),
    ("Git",            "Entwicklung", "Versionskontrolle",    "git"),
]
CATEGORIES = ["Alle", "System", "Internet", "Büro", "Multimedia", "Spiele", "Entwicklung"]


class Store(Gtk.Window):
    def __init__(self):
        super().__init__(title="Joys Store")
        self.set_default_size(860, 600)
        self.set_position(Gtk.WindowPosition.CENTER)

        root = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        root.set_margin_top=22
        root.set_margin_bottom=22
        root.set_margin_left=30
        root.set_margin_right=30
        self.add(root)

        title = Gtk.Label(label="Joys Store", xalign=0)
        title.get_style_context().add_class("title")
        root.pack_start(title, False, False, 0)

        top = Gtk.Box(spacing=8)
        self.search = Gtk.Entry()
        self.search.set_placeholder_text("Apps suchen ...")
        self.search.get_style_context().add_class("search")
        self.search.connect("changed", self.apply_filter)
        top.pack_start(self.search, True, True, 0)
        root.pack_start(top, False, False, 0)

        cats = Gtk.Box(spacing=4)
        self._cat_buttons = []
        for c in CATEGORIES:
            b = Gtk.ToggleButton(label=c)
            b.get_style_context().add_class("cat")
            b.set_name(c)
            b.connect("toggled", self.on_cat)
            cats.pack_start(b, False, False, 0)
            self._cat_buttons.append(b)
        root.pack_start(cats, False, False, 0)
        self._cat_buttons[0].set_active(True)

        self._grid = Gtk.FlowBox()
        self._grid.set_selection_mode(Gtk.SelectionMode.NONE)
        self._grid.set_max_children_per_line(3)
        self._grid.set_homogeneous(True)
        for name, cat, desc, pkg in APPS:
            self._grid.add(self._card(name, cat, desc, pkg))
        sc = Gtk.ScrolledWindow()
        sc.add(self._grid)
        root.pack_start(sc, True, True, 0)

        self._cat = "Alle"
        self.connect("destroy", Gtk.main_quit)
        self.show_all()

    def _card(self, name, cat, desc, pkg):
        card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        card.get_style_context().add_class("card")
        t = Gtk.Label(label=name, xalign=0)
        t.get_style_context().add_class("card-title")
        d = Gtk.Label(label=desc, xalign=0, wrap=True)
        d.get_style_context().add_class("card-desc")
        card.pack_start(t, False, False, 0)
        card.pack_start(d, False, False, 0)
        if pkg:
            b = Gtk.Button(label="Install")
            b.get_style_context().add_class("install")
            b.connect("clicked", lambda *_, p=pkg, n=name: self.install(p, n))
            card.pack_start(b, False, False, 0)
        else:
            b = Gtk.Label(label="Coming Soon")
            b.get_style_context().add_class("soon")
            b.set_halign(Gtk.Align.START)
            card.pack_start(b, False, False, 0)
        return card

    def install(self, pkg, name):
        subprocess.Popen(
            ["lxterminal", "-e",
             f"sudo apt-get install -y {pkg} && echo '=== {name} installiert ===' && sleep 3"])

    def on_cat(self, btn):
        if btn.get_active():
            self._cat = btn.get_name()
            for b in self._cat_buttons:
                if b is not btn:
                    b.set_active(False)
            self.apply_filter()

    def apply_filter(self, *_):
        q = self.search.get_text().strip().lower()
        for i, (name, cat, _d, _p) in enumerate(APPS):
            child = self._grid.get_children()[i]
            visible = True
            if self._cat != "Alle" and cat != self._cat:
                visible = False
            if q and q not in name.lower():
                visible = False
            child.set_visible(visible)


def main():
    css = Gtk.CssProvider()
    css.load_from_data(CSS.encode())
    Gtk.StyleContext.add_provider_for_screen(
        Gdk.Screen.get_default(), css, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
    Gtk.Settings.get_default().set_property("gtk-application-prefer-dark-theme", True)
    Store()
    Gtk.main()


if __name__ == "__main__":
    main()
