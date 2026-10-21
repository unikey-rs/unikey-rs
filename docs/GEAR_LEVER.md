# Gear Lever integration

UniKey RS AppImages can be installed and updated through Gear Lever on Fedora
Atomic and other immutable desktops.

## Integrate the current release

```sh
flatpak run it.mijorus.gearlever --integrate --yes \
  UniKey_RS-0.1.3-x86_64.AppImage
```

Gear Lever copies the AppImage to its managed folder and creates the user-level
desktop entry and icon. Do not create a second manual desktop entry.

## Enable GitHub updates

```sh
flatpak run it.mijorus.gearlever \
  --set-update-source ~/AppImages/unikey_rs.appimage \
  --manager GithubUpdater \
  allow_prereleases=1 \
  repo=unikey-rs/unikey-rs \
  'repo_filename=UniKey_RS-*-x86_64.AppImage'
```

List and apply updates:

```sh
flatpak run it.mijorus.gearlever --list-installed
flatpak run it.mijorus.gearlever --list-updates
flatpak run it.mijorus.gearlever --update ~/AppImages/unikey_rs.appimage --yes
```

The AppImage desktop entry carries `X-AppImage-Version`, so Gear Lever can
display and distinguish releases.
