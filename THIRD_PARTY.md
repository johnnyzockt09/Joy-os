# THIRD_PARTY.md – Third-Party References & Licenses

> **Lizenz-Status von Joys selbst:** Der eigene Joys-Code (joys-core,
> joys-win, Shell, Installer, Skripte) ist **proprietär** lizenziert
> (siehe [LICENSE](LICENSE)). Die unten genannten Drittanbieter-Komponenten,
> auf denen Joys aufbaut, bleiben unter ihren jeweiligen Open-Source-Lizenzen.

This file documents every third-party project that is **referenced**, **studied**
or **used** by Joys OS, together with its license and the parts we rely on.

> Rule: Joys must never violate a license. Code may only be copied verbatim when
> the license explicitly permits it AND it is attributed here. In general Joys
> implements its own solutions and uses third-party projects only as behavioral
> references.

## Runtime / Build dependencies

| Project | URL | License | Used for |
|---------|-----|---------|----------|
| Linux Kernel | https://www.kernel.org | GPL-2.0 | Foundation of Joys OS |
| GNU GRUB | https://www.gnu.org/software/grub/ | GPL-3.0 | UEFI bootloader |
| Rust toolchain | https://rustup.rs | MIT/Apache-2.0 | Primary language for joys-core / joys-win |
| GTK (experimental) | https://www.gtk.org | LGPL-2.1 | Native Joys desktop widgets (future) |
| QEMU | https://www.qemu.org | GPL-2.0 | Testing the ISO (not shipped) |

## Behavioral references (NOT copied into Joys)

These projects are studied for API semantics, ABI details and system behavior.
No code is copied from them without explicit license review.

| Project | URL | License | Referenced for |
|---------|-----|---------|----------------|
| Wine | https://www.winehq.org | LGPL-2.1 | Windows API semantics, PE loading behavior (development comparison ONLY, never the runtime of Joys) |
| ReactOS | https://www.reactos.org | GPL-2.0 | Windows API behavior, registry behavior, process model, Win32 semantics |
| MSYS2 / mingw-w64 | https://www.msys2.org | various | Cross-compiling Windows test programs (hello.exe etc.) |
| FreeDesktop | https://www.freedesktop.org | MIT | Desktop entry spec, .desktop files, XDG dirs |
| Openbox | http://openbox.org | GPL-2.0 | Lightweight window manager (candidate) |
| PipeWire | https://www.pipewire.org | MIT | Audio backend (future) |
| Mesa / Vulkan | https://www.mesa3d.org | MIT | Graphics compatibility layer (future) |

## Design reference

| Project | URL | License | Used for |
|---------|-----|---------|----------|
| lttthedev / lttthedev.github.io | https://github.com/lttthedev/lttthedev.github.io | EPL-2.0 | Visual / design reference for the Joys UI. The website is NOT copied; only design ideas are adapted. EPL-2.0 obligations (if any derived work) are respected. |

## Notes

- No Microsoft logos or trademarks are used anywhere in Joys.
- If a component listed above is later shipped inside the Joys ISO, it must be
  added to the appropriate NOTICE/LICENSE file of the corresponding package.
- Last reviewed: 2026-08-16
