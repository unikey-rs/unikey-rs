# Fedora Atomic and immutable desktops

Fedora Atomic images (including Silverblue, Kinoite, and COSMIC Atomic) do not
make system package installation a suitable integration mechanism. UniKey RS is
therefore distributed as an AppImage and writes only user-level integration
files.

## Recommended

```sh
mkdir -p ~/.local/bin
cp UniKey_RS-0.1.0-x86_64.AppImage ~/.local/bin/
chmod +x ~/.local/bin/UniKey_RS-0.1.0-x86_64.AppImage
~/.local/bin/UniKey_RS-0.1.0-x86_64.AppImage doctor
```

## Optional desktop entry

```sh
mkdir -p ~/.local/share/applications ~/.local/share/icons/hicolor/scalable/apps
cp packaging/appdir/usr/share/applications/unikey-rs.desktop ~/.local/share/applications/
cp packaging/appdir/usr/share/icons/hicolor/scalable/apps/unikey-rs.svg ~/.local/share/icons/hicolor/scalable/apps/
update-desktop-database ~/.local/share/applications 2>/dev/null || true
gtk-update-icon-cache ~/.local/share/icons/hicolor 2>/dev/null || true
```

Edit `Exec=` in the copied desktop file if the AppImage is stored elsewhere.

## Build portability

The release builder prefers `x86_64-unknown-linux-musl`. If musl is not
available, build on an intentionally old glibc baseline in a container rather
than on the newest host. This avoids producing an AppImage that only runs on
the build machine's unusually new Fedora Atomic image.
