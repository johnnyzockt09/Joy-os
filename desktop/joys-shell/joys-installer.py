#!/usr/bin/env python3
# Joys Installer – moderne GTK-Oberfläche (Schritte + animierter Fortschritt).
import gi
import subprocess
import threading

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk, GLib

BG = "#1b1b2f"
FG = "#e8e8f2"
ACCENT = "#3d7eff"
CSS = f"""
window {{ background-color: {BG}; }}
.title {{ color: {FG}; font-size: 26px; font-weight: bold; }}
.sub {{ color: {FG}; font-size: 13px; }}
.step-dot {{ color: {ACCENT}; font-size: 13px; }}
.field {{ color: {FG}; font-size: 13px; }}
.card {{ background-color: rgba(255,255,255,0.06); border-radius: 14px; }}
.primary {{ background-color: {ACCENT}; color: white; border-radius: 8px;
           font-size: 14px; padding: 10px 22px; transition: 150ms ease; }}
.primary:hover {{ background-color: #5a94ff; }}
.secondary {{ background: transparent; color: {FG}; border-radius: 8px;
             font-size: 14px; padding: 10px 22px; transition: 150ms ease; }}
.secondary:hover {{ background-color: rgba(255,255,255,0.08); }}
.log {{ background-color: #10101c; color: #b8ffb8; font-family: monospace;
       font-size: 11px; }}
""".strip()


def shell(cmd):
    try:
        return subprocess.run(cmd, shell=True, capture_output=True, text=True).stdout.strip()
    except Exception:
        return "?"


def disks():
    out = shell("lsblk -d -n -o NAME,SIZE,MODEL")
    return [f"/dev/{line.split()[0]}  {line.split()[1]}" for line in out.splitlines()
            if line.split() and not line.startswith("sr")]


class Installer(Gtk.Window):
    def __init__(self):
        super().__init__(title="Joys Installer")
        self.set_default_size(680, 560)
        self.set_position(Gtk.WindowPosition.CENTER)

        root = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16)
        root.set_margin_top(30)
        root.set_margin_bottom(30)
        root.set_margin_left=36
        root.set_margin_right=36
        self.add(root)

        title = Gtk.Label(label="Joys OS installieren", xalign=0)
        title.get_style_context().add_class("title")
        root.pack_start(title, False, False, 0)

        # Schritt-Indikator (Punkte mit Übergang).
        self.step_label = Gtk.Label(label="Schritt 1 von 4", xalign=0)
        self.step_label.get_style_context().add_class("step-dot")
        root.pack_start(self.step_label, False, False, 0)

        # Stack mit Seitenübergängen.
        self.stack = Gtk.Stack()
        self.stack.set_transition_type(Gtk.StackTransitionType.SLIDE_LEFT_RIGHT)
        self.stack.set_transition_duration(250)
        root.pack_start(self.stack, True, True, 0)

        self._build_welcome()
        self._build_disk()
        self._build_progress()
        self._build_done()
        self.stack.set_visible_child_name("welcome")

        # Navigation.
        nav = Gtk.Box(spacing=8)
        self.back_btn = Gtk.Button(label="Zurück")
        self.back_btn.get_style_context().add_class("secondary")
        self.back_btn.connect("clicked", self.on_back)
        self.next_btn = Gtk.Button(label="Weiter")
        self.next_btn.get_style_context().add_class("primary")
        self.next_btn.connect("clicked", self.on_next)
        nav.pack_start(self.back_btn, False, False, 0)
        nav.pack_end(self.next_btn, False, False, 0)
        root.pack_start(nav, False, False, 0)

        self._page = 0
        self._pages = ["welcome", "disk", "progress", "done"]
        self.update_nav()
        self.connect("destroy", Gtk.main_quit)
        self.show_all()

    def _wrap(self, child):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
        v.pack_start(child, True, True, 0)
        return v

    def _build_welcome(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
        lbl = Gtk.Label(label="Willkommen bei Joys OS.\n\nDiese Installationsoberfläche "
                              "richtet Joys auf deiner Festplatte ein.\nDas gewählte Laufwerk "
                              "wird dabei vollständig überschrieben.",
                        wrap=True, justify=Gtk.Justification.LEFT)
        lbl.get_style_context().add_class("sub")
        v.pack_start(lbl, False, False, 0)
        self.stack.add_named(self._wrap(v), "welcome")

    def _build_disk(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        lbl = Gtk.Label(label="Wähle das Ziellaufwerk", xalign=0)
        lbl.get_style_context().add_class("field")
        v.pack_start(lbl, False, False, 0)
        self.disk_combo = Gtk.ComboBoxText()
        for d in disks():
            self.disk_combo.append_text(d)
        if self.disk_combo.get_active() == -1:
            self.disk_combo.set_active(0)
        v.pack_start(self.disk_combo, False, False, 0)

        lbl2 = Gtk.Label(label="Benutzername", xalign=0)
        lbl2.get_style_context().add_class("field")
        v.pack_start(lbl2, False, False, 0)
        self.user_entry = Gtk.Entry(text="joys")
        v.pack_start(self.user_entry, False, False, 0)
        self.stack.add_named(self._wrap(v), "disk")

    def _build_progress(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
        lbl = Gtk.Label(label="Installation läuft ...", xalign=0)
        lbl.get_style_context().add_class("sub")
        v.pack_start(lbl, False, False, 0)
        self.progress = Gtk.ProgressBar()
        self.progress.set_pulse_step(0.05)
        v.pack_start(self.progress, False, False, 0)
        self.log = Gtk.TextView()
        self.log.set_editable(False)
        self.log.set_cursor_visible(False)
        self.log.get_style_context().add_class("log")
        sc = Gtk.ScrolledWindow()
        sc.set_min_content_height(220)
        sc.add(self.log)
        v.pack_start(sc, True, True, 0)
        self.stack.add_named(self._wrap(v), "progress")

    def _build_done(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
        check = Gtk.Image(icon_name="emblem-ok-symbolic", pixel_size=56)
        v.pack_start(check, False, False, 0)
        lbl = Gtk.Label(label="Fertig! Joys wurde installiert.\n"
                              "Starte jetzt neu, um Joys zu verwenden.",
                        wrap=True, justify=Gtk.Justification.CENTER)
        lbl.get_style_context().add_class("sub")
        v.pack_start(lbl, False, False, 0)
        self.stack.add_named(self._wrap(v), "done")

    def update_nav(self):
        self.back_btn.set_sensitive(self._page > 0)
        if self._page == 3:
            self.next_btn.set_label("Neu starten")
            self.next_btn.connect("clicked", lambda *_: subprocess.Popen(["systemctl", "reboot"]))
        elif self._page == 2:
            self.next_btn.set_sensitive(False)
        else:
            self.next_btn.set_label("Weiter")
            self.next_btn.set_sensitive(True)

    def on_back(self, _b):
        if self._page > 0:
            self._page -= 1
            self.stack.set_visible_child_name(self._pages[self._page])
            self.step_label.set_text(f"Schritt {self._page + 1} von 4")
            self.update_nav()

    def on_next(self, _b):
        if self._page == 2:
            return
        self._page += 1
        self.stack.set_visible_child_name(self._pages[self._page])
        self.step_label.set_text(f"Schritt {self._page + 1} von 4")
        self.update_nav()
        if self._page == 2:
            self.start_install()

    def append_log(self, text):
        buf = self.log.get_buffer()
        buf.insert(buf.get_end_iter(), text + "\n")
        adj = self.log.get_parent().get_vadjustment()
        adj.set_value(adj.get_upper())

    def start_install(self):
        sel = self.disk_combo.get_active_text() or ""
        disk = sel.split()[0]
        user = self.user_entry.get_text() or "joys"
        self.append_log(f"Starte Installation auf {disk} (Benutzer {user}) ...")
        GLib.timeout_add(80, self._pulse)

        def run():
            proc = subprocess.Popen(
                ["/usr/local/bin/joys-install.sh", disk, user],
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            for line in proc.stdout:
                Gdk.threads_add_idle(GLib.PRIORITY_DEFAULT_IDLE,
                                     lambda l=line: self.append_log(l.rstrip()))
            proc.wait()
            Gdk.threads_add_idle(GLib.PRIORITY_DEFAULT_IDLE, self.on_done)

        threading.Thread(target=run, daemon=True).start()

    def _pulse(self):
        self.progress.pulse()
        return self._page == 2

    def on_done(self):
        self.append_log("=== FERTIG: Joys wurde installiert. ===")
        self.progress.set_fraction(1.0)
        self._page = 3
        self.stack.set_visible_child_name("done")
        self.step_label.set_text("Installation abgeschlossen")
        self.update_nav()
        return False


def main():
    css = Gtk.CssProvider()
    css.load_from_data(CSS.encode())
    Gtk.StyleContext.add_provider_for_screen(
        Gdk.Screen.get_default(), css, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
    Gtk.Settings.get_default().set_property("gtk-application-prefer-dark-theme", True)
    Installer()
    Gtk.main()


if __name__ == "__main__":
    main()
