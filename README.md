# UniKey RS

[Tiếng Việt](README.vi.md) | English

**Beta identity: `UniKey RS`. Version line: `0.x`.**

UniKey RS is a Rust text composer with a unified CLI and a dependency-free,
native Wayland GUI. It supports English/US pass-through plus Vietnamese Telex,
VNI, and VIQR through one tested composition engine.

> This repository is an early public beta. It deliberately remains `0.x`.
> A `1.0` release is reserved for a nearly finished, highly mature product with
> first-class Linux, macOS, and Windows support.

## Highlights

- One engine shared by CLI, GUI, and the Wayland input-method bridge.
- Progressive composition, backspace, uppercase Vietnamese, tone placement,
  shape marks, and repeated-modifier restore keys.
- Native Wayland `xdg-shell` + `wl_shm` GUI without GTK/Qt.
- Native Wayland `zwp_input_method_v1` keyboard-grab backend where the
  compositor advertises it.
- User-level configuration in `~/.config/unikey-rs/config`.
- AppImage packaging designed for immutable distributions such as Fedora
  Atomic, Silverblue, Kinoite, and COSMIC Atomic.

## Quick start

```sh
cargo run -- layouts
cargo run -- compose --layout telex 'Xin chaof the gioiw'
printf 'a61' | cargo run -- compose --layout vni
cargo run -- doctor
cargo run -- gui --layout telex
cargo run -- input-method --layout telex
```

CLI composition reads raw keystrokes and writes rendered text:

```sh
unikey-rs compose --layout telex 'awf'
# ằ

unikey-rs compose --layout viqr "a^'"
# ấ

unikey-rs compose --layout vni 'a61'
# ấ
```

Use `--no-restore-keys` when you prefer literal suffixes instead of UniKey-style
repeated-modifier restore (`ass` -> `as` by default).

## GUI

The GUI is a native Wayland window with clickable layout buttons and a live
raw/output preview. It does not require a desktop toolkit. `1`–`4` select
English/Telex/VNI/VIQR; Backspace edits the preview.

## Wayland input method

Run `unikey-rs doctor` first. It reports the real compositor globals instead of
assuming GNOME, KDE, COSMIC, Sway, or another desktop.

`unikey-rs input-method` prefers `zwp_input_method_v2` (including the COSMIC
implementation), which provides a keyboard grab for a progressive composer. It
falls back to `zwp_input_method_v1` for older compositors. Some
compositors do not expose an input-method protocol or may prefer
`zwp_input_method_v2`/experimental `xx_input_method_v1`; the architecture keeps
those as replaceable backends over the same `layout::Composer`.

Wayland security intentionally prevents ordinary clients from injecting
keystrokes into unrelated applications. Use the input-method path or a
compositor-supported virtual-input protocol—never an X11-style global injector.

## Fedora Atomic / immutable Linux

No system installation is required. Download the AppImage, make it executable,
and run it from your home directory:

```sh
chmod +x UniKey_RS-0.1.3-x86_64.AppImage
./UniKey_RS-0.1.3-x86_64.AppImage doctor
./UniKey_RS-0.1.3-x86_64.AppImage gui
```

The build prefers a static `x86_64-unknown-linux-musl` binary when available.
The AppImage never writes to immutable `/usr`; optional desktop integration is
documented in `docs/FEDORA_ATOMIC.md`.

## Configuration

```sh
unikey-rs config-show
unikey-rs config-set language vi
unikey-rs config-set layout telex
unikey-rs config-set restore_keys true
```

`UNIKEY_RS_CONFIG` may override the config path for tests, containers, or
portable use.

## Development

```sh
cargo test --locked
cargo clippy --all-targets -- -D warnings
scripts/build-appimage.sh
```

See `docs/ARCHITECTURE.md` and `docs/WAYLAND.md` for protocol boundaries and
portability constraints.

## License

MIT. See `LICENSE`.

For Gear Lever installation and GitHub updates, see [docs/GEAR_LEVER.md](docs/GEAR_LEVER.md).
