#!/usr/bin/env python3
# Joys Installer – professioneller grafischer Installer.
# Schritte: Welcome → Sprache → Tastatur → Zeitzone → Benutzer → Disk
#           → Summary → Installation → Fertig.
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
.step {{ color: {ACCENT}; font-size: 12px; font-weight: bold; letter-spacing: 1px; }}
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

LANGUAGES = [("Deutsch", "de"), ("English", "en")]
KEYBOARDS = [("German", "de"), ("US", "us"), ("UK (English)", "gb")]
TIMEZONES = ["Europe/Berlin", "Europe/Vienna", "Europe/Zurich", "Europe/London",
             "Europe/Paris", "America/New_York", "America/Los_Angeles",
             "UTC"]


def shell(cmd):
    try:
        return subprocess.run(cmd, shell=True, capture_output=True, text=True).stdout.strip()
    except Exception:
        return "?"


def disks():
    out = shell("lsblk -d -n -o NAME,SIZE,MODEL")
    return [f"/dev/{line.split()[0]}  {line.split()[1]}"
            for line in out.splitlines()
            if line.split() and not line.startswith("sr")]


class Installer(Gtk.Window):
    def __init__(self):
        super().__init__(title="Joys Installer")
        self.set_default_size(720, 580)
        self.set_position(Gtk.WindowPosition.CENTER)
        self._lang = "de"
        self._kbd = "de"
        self._tz = "Europe/Berlin"
        self._disk = ""
        self._user = "joys"
        self._host = "joys"
        self._fullname = "Joys User"
        self._confirmed = False

        root = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
        root.set_margin_top(26)
        root.set_margin_bottom(26)
        root.set_margin_left(36)
        root.set_margin_right(36)
        self.add(root)

        self.step_label = Gtk.Label(label="WILLKOMMEN", xalign=0)
        self.step_label.get_style_context().add_class("step")
        root.pack_start(self.step_label, False, False, 0)

        self.stack = Gtk.Stack()
        self.stack.set_transition_type(Gtk.StackTransitionType.SLIDE_LEFT_RIGHT)
        self.stack.set_transition_duration(250)
        root.pack_start(self.stack, True, True, 0)

        self._build_welcome()
        self._build_language()
        self._build_keyboard()
        self._build_timezone()
        self._build_user()
        self._build_disk()
        self._build_summary()
        self._build_progress()
        self._build_done()
        self._pages = ["welcome", "language", "keyboard", "timezone", "user",
                       "disk", "summary", "progress", "done"]
        self._page = 0
        self.stack.set_visible_child_name("welcome")

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

        self.update_nav()
        self.connect("destroy", Gtk.main_quit)
        self.show_all()

    # ---- Seiten ----
    def _wrap(self, child, title):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        t = Gtk.Label(label=title, xalign=0)
        t.get_style_context().add_class("title")
        v.pack_start(t, False, False, 0)
        v.pack_start(child, True, True, 0)
        return v

    def _build_welcome(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        lbl = Gtk.Label(label="Welcome to Joys\n\nA fast and modern operating system.\n\n"
                              "Diese Oberfläche richtet Joys auf deiner Festplatte ein. "
                              "Das gewählte Laufwerk wird überschrieben.",
                        wrap=True, justify=Gtk.Justification.LEFT)
        lbl.get_style_context().add_class("sub")
        v.pack_start(lbl, False, False, 0)
        self.stack.add_named(self._wrap(v, "Joys"), "welcome")

    def _build_language(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.lang = Gtk.ListBox()
        self.lang.get_style_context().add_class("card")
        for label, code in LANGUAGES:
            r = Gtk.ListBoxRow()
            b = Gtk.Label(label=label, xalign=0)
            b.get_style_context().add_class("field")
            b.set_margin_start=16
            b.set_margin_top=12
            b.set_margin_bottom=12
            r.add(b)
            self.lang.add(r)
        self.lang.connect("row-selected", self.on_lang)
        self.stack.add_named(self._wrap(v, "Language"), "language")

    def _build_keyboard(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.kbd = Gtk.ListBox()
        self.kbd.get_style_context().add_class("card")
        for label, code in KEYBOARDS:
            r = Gtk.ListBoxRow()
            b = Gtk.Label(label=label, xalign=0)
            b.get_style_context().add_class("field")
            b.set_margin_start=16
            b.set_margin_top=12
            b.set_margin_bottom=12
            r.add(b)
            self.kbd.add(r)
        self.kbd.connect("row-selected", self.on_kbd)
        self.stack.add_named(self._wrap(v, "Keyboard"), "keyboard")

    def _build_timezone(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.tz = Gtk.ComboBoxText()
        for t in TIMEZONES:
            self.tz.append_text(t)
        self.tz.set_active(0)
        self.tz.connect("changed", self.on_tz)
        v.pack_start(self.tz, False, False, 0)
        self.stack.add_named(self._wrap(v, "Timezone"), "timezone")

    def _build_user(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        self.fullname = self._entry("Full name", "Joys User")
        self.username = self._entry("Username", "joys")
        self.hostname = self._entry("Computer name", "joys")
        self.passwd = Gtk.Entry()
        self.passwd.set_placeholder_text("Password")
        self.passwd.set_visibility(False)
        v.pack_start(self.passwd, False, False, 0)
        self.stack.add_named(self._wrap(v, "Create your account"), "user")

    def _entry(self, placeholder, default=""):
        e = Gtk.Entry()
        e.set_placeholder_text(placeholder)
        if default:
            e.set_text(default)
        return e

    def _build_disk(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        self.disk_combo = Gtk.ComboBoxText()
        for d in disks():
            self.disk_combo.append_text(d)
        if self.disk_combo.get_active() == -1:
            self.disk_combo.set_active(0)
        self.disk_combo.connect("changed", self.on_disk)
        v.pack_start(self.disk_combo, False, False, 0)
        warn = Gtk.Label(label="Es wird „Erase disk and install Joys“ ausgeführt. "
                               "Alle Daten auf dem Laufwerk gehen verloren!",
                         wrap=True, xalign=0)
        warn.get_style_context().add_class("sub")
        v.pack_start(warn, False, False, 0)
        self.stack.add_named(self._wrap(v, "Installation Drive"), "disk")

    def _build_summary(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        self.summary = Gtk.Label(label="", wrap=True, xalign=0)
        self.summary.get_style_context().add_class("sub")
        v.pack_start(self.summary, False, False, 0)
        self.confirm = Gtk.CheckButton(label="Ich verstehe: Dieses Laufwerk wird vollständig gelöscht.")
        self.confirm.get_style_context().add_class("field")
        self.confirm.connect("toggled", lambda *_: self.update_nav())
        v.pack_start(self.confirm, False, False, 0)
        self.stack.add_named(self._wrap(v, "Ready to install Joys"), "summary")

    def _build_progress(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        self.status = Gtk.Label(label="Vorbereitung ...", xalign=0)
        self.status.get_style_context().add_class("sub")
        v.pack_start(self.status, False, False, 0)
        self.progress = Gtk.ProgressBar()
        self.progress.set_pulse_step(0.05)
        v.pack_start(self.progress, False, False, 0)
        self.log = Gtk.TextView()
        self.log.set_editable(False)
        self.log.set_cursor_visible(False)
        self.log.get_style_context().add_class("log")
        sc = Gtk.ScrolledWindow()
        sc.set_min_content_height(200)
        sc.add(self.log)
        v.pack_start(sc, True, True, 0)
        self.stack.add_named(self._wrap(v, "Installing Joys"), "progress")

    def _build_done(self):
        v = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
        check = Gtk.Image(icon_name="emblem-ok-symbolic", pixel_size=60)
        v.pack_start(check, False, False, 0)
        lbl = Gtk.Label(label="Joys has been installed!\n\nYour system is ready.\n"
                              "Remove the installation media and restart.",
                        wrap=True, justify=Gtk.Justification.CENTER)
        lbl.get_style_context().add_class("sub")
        v.pack_start(lbl, False, False, 0)
        self.stack.add_named(self._wrap(v, "Installation complete"), "done")

    # ---- Navigation ----
    def update_nav(self):
        page = self._pages[self._page]
        self.back_btn.set_sensitive(self._page > 0)
        if page == "done":
            self.next_btn.set_label("Restart now")
            self.next_btn.set_sensitive(True)
            self.next_btn.connect("clicked", lambda *_: subprocess.Popen(["systemctl", "reboot"]))
        elif page == "progress":
            self.next_btn.set_sensitive(False)
        elif page == "summary":
            self.next_btn.set_label("Install Joys")
            self.next_btn.set_sensitive(self.confirm.get_active())
        else:
            self.next_btn.set_label("Weiter")
            self.next_btn.set_sensitive(True)

    def on_back(self, _b):
        if self._page > 0:
            self._page -= 1
            self._show()

    def on_next(self, _b):
        if self._pages[self._page] == "progress":
            return
        if self._pages[self._page] == "disk":
            if not self._disk:
                return
        if self._page + 1 < len(self._pages):
            self._page += 1
            self._show()
            if self._pages[self._page] == "progress":
                self.start_install()

    def _show(self):
        name = self._pages[self._page]
        self.stack.set_visible_child_name(name)
        labels = {"welcome": "Willkommen", "language": "Sprache",
                  "keyboard": "Tastatur", "timezone": "Zeitzone",
                  "user": "Benutzer", "disk": "Laufwerk",
                  "summary": "Zusammenfassung", "progress": "Installation",
                  "done": "Fertig"}
        self.step_label.set_text(labels.get(name, name).upper())
        self.update_nav()
        if name == "summary":
            self.refresh_summary()

    def refresh_summary(self):
        disk = self._disk or (self.disk_combo.get_active_text() or "?")
        self.summary.set_text(
            f"Language:  {self._lang}\n"
            f"Keyboard:  {self._kbd}\n"
            f"Timezone:  {self._tz}\n"
            f"Disk:      {disk}\n"
            f"Username:  {self._user}\n"
            f"Installation: Erase disk\n"
            f"Computer:  {self._host}")

    # ---- Callbacks ----
    def on_lang(self, lb, row):
        if row:
            self._lang = LANGUAGES[row.get_index()][1]

    def on_kbd(self, lb, row):
        if row:
            self._kbd = KEYBOARDS[row.get_index()][1]

    def on_tz(self, _c):
        self._tz = self.tz.get_active_text() or "Europe/Berlin"

    def on_disk(self, _c):
        sel = self.disk_combo.get_active_text() or ""
        self._disk = sel.split()[0] if sel else ""

    # ---- Installation ----
    def append_log(self, text):
        buf = self.log.get_buffer()
        buf.insert(buf.get_end_iter(), text + "\n")
        adj = self.log.get_parent().get_vadjustment()
        adj.set_value(adj.get_upper())

    def start_install(self):
        self._user = self.username.get_text() or "joys"
        self._host = self.hostname.get_text() or "joys"
        self._fullname = self.fullname.get_text() or "Joys User"
        password = self.passwd.get_text() or "joys"
        disk = self._disk
        if not disk:
            disk = (self.disk_combo.get_active_text() or "").split()[0]
        self.status.set_text("Kopiere Systemdateien ...")
        self.append_log(f"Starte Installation auf {disk} ...")
        GLib.timeout_add(80, self._pulse)

        def run():
            proc = subprocess.Popen(
                ["/usr/local/bin/joys-install.sh", disk, self._user,
                 self._host, self._kbd, self._tz, self._fullname, password],
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            for line in proc.stdout:
                Gdk.threads_add_idle(GLib.PRIORITY_DEFAULT_IDLE,
                                     lambda l=line: self.append_log(l.rstrip()))
            proc.wait()
            Gdk.threads_add_idle(GLib.PRIORITY_DEFAULT_IDLE, self.on_done)

        threading.Thread(target=run, daemon=True).start()

    def _pulse(self):
        self.progress.pulse()
        return self._pages[self._page] == "progress"

    def on_done(self):
        self.append_log("=== FERTIG: Joys wurde installiert. ===")
        self.progress.set_fraction(1.0)
        self._page = self._pages.index("done")
        self.stack.set_visible_child_name("done")
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
