#!/usr/bin/env python3
# Joys Installer – moderne GTK-Oberfläche für scripts/joys-install.sh.
import gi
import os
import subprocess
import threading

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk

BG = "#1b1b2f"
FG = "#e8e8f2"
ACCENT = "#3d7eff"
CSS = f"""
window {{ background-color: {BG}; }}
.title {{ color: {FG}; font-size: 22px; font-weight: bold; }}
.sub {{ color: {FG}; font-size: 13px; }}
.field {{ color: {FG}; font-size: 13px; }}
.step {{ color: {ACCENT}; font-size: 12px; font-weight: bold; }}
.card {{ background-color: rgba(255,255,255,0.06); border-radius: 12px; }}
.power-button {{ background: transparent; color: {FG}; border-radius: 8px;
                font-size: 14px; padding: 10px 20px; }}
.power-button:hover {{ background-color: rgba(255,255,255,0.10); }}
.accent-button {{ background-color: {ACCENT}; color: white; border-radius: 8px;
                 font-size: 14px; padding: 10px 20px; }}
.log {{ background-color: #10101c; color: #b8ffb8; font-family: monospace;
       font-size: 11px; }}
""".strip()


def shell(cmd):
    try:
        return subprocess.run(cmd, shell=True, capture_output=True, text=True).stdout.strip()
    except Exception:
        return "?"


def disks():
    out = shell("lsblk -d -n -o NAME,SIZE,MODEL,TYPE")
    result = []
    for line in out.splitlines():
        parts = line.split(None, 3)
        if len(parts) >= 2 and parts[0] not in ("sr",):
            result.append(f"/dev/{parts[0]}  {parts[1]}")
    return result


class Installer(Gtk.Window):
    def __init__(self):
        super().__init__(title="Joys Installer")
        self.set_default_size(640, 520)
        self.set_position(Gtk.WindowPosition.CENTER)

        root = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16)
        root.set_margin_top(28)
        root.set_margin_bottom(28)
        root.set_margin_start(34)
        root.set_margin_end(34)
        self.add(root)

        title = Gtk.Label(label="Joys OS installieren", xalign=0)
        title.get_style_context().add_class("title")
        root.pack_start(title, False, False, 0)
        sub = Gtk.Label(label="Installiere Joys auf deine Festplatte. "
                              "Das Laufwerk wird vollständig überschrieben.",
                        xalign=0, wrap=True)
        sub.get_style_context().add_class("sub")
        root.pack_start(sub, False, False, 0)

        step = Gtk.Label(label="SCHRITT 1 – Festplatte wählen", xalign=0)
        step.get_style_context().add_class("step")
        root.pack_start(step, False, False, 0)

        self.disk_combo = Gtk.ComboBoxText()
        for d in disks():
            self.disk_combo.append_text(d)
        if self.disk_combo.get_active() == -1:
            self.disk_combo.set_active(0)
        root.pack_start(self.disk_combo, False, False, 0)

        step2 = Gtk.Label(label="SCHRITT 2 – Benutzer", xalign=0)
        step2.get_style_context().add_class("step")
        root.pack_start(step2, False, False, 0)

        urow = Gtk.Box(spacing=10)
        ul = Gtk.Label(label="Benutzername", xalign=0)
        ul.get_style_context().add_class("field")
        self.user_entry = Gtk.Entry(text="joys")
        urow.pack_start(ul, False, False, 0)
        urow.pack_start(self.user_entry, True, True, 0)
        root.pack_start(urow, False, False, 0)

        self.log = Gtk.TextView()
        self.log.set_editable(False)
        self.log.set_cursor_visible(False)
        self.log.get_style_context().add_class("log")
        scroller = Gtk.ScrolledWindow()
        scroller.set_min_content_height(180)
        scroller.add(self.log)
        root.pack_start(scroller, True, True, 0)

        btnrow = Gtk.Box(spacing=10)
        self.install_btn = Gtk.Button(label="Installieren")
        self.install_btn.get_style_context().add_class("accent-button")
        self.install_btn.connect("clicked", self.on_install)
        self.reboot_btn = Gtk.Button(label="Neu starten")
        self.reboot_btn.get_style_context().add_class("power-button")
        self.reboot_btn.set_sensitive(False)
        self.reboot_btn.connect("clicked", lambda *_: subprocess.Popen(["systemctl", "reboot"]))
        btnrow.pack_start(self.install_btn, False, False, 0)
        btnrow.pack_start(self.reboot_btn, False, False, 0)
        root.pack_start(btnrow, False, False, 0)

        self.connect("destroy", Gtk.main_quit)
        self.show_all()

    def append_log(self, text):
        buf = self.log.get_buffer()
        buf.insert(buf.get_end_iter(), text + "\n")
        adj = self.log.get_parent().get_vadjustment()
        adj.set_value(adj.get_upper())

    def on_install(self, _b):
        sel = self.disk_combo.get_active_text() or ""
        disk = sel.split()[0]
        user = self.user_entry.get_text() or "joys"
        self.install_btn.set_sensitive(False)
        self.append_log(f"Starte Installation auf {disk} (Benutzer {user}) ...")

        def run():
            proc = subprocess.Popen(
                ["/usr/local/bin/joys-install.sh", disk, user],
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            for line in proc.stdout:
                Gdk.threads_add_idle(GLib.PRIORITY_DEFAULT_IDLE, lambda l=line: self.append_log(l.rstrip()))
            proc.wait()
            Gdk.threads_add_idle(GLib.PRIORITY_DEFAULT_IDLE, self.on_done)

        threading.Thread(target=run, daemon=True).start()

    def on_done(self):
        self.append_log("=== FERTIG. Joys wurde installiert. ===")
        self.reboot_btn.set_sensitive(True)
        return False


from gi.repository import GLib


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
