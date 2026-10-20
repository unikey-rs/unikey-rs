# Changelog

All releases remain on the `0.x` line during active development.

## 0.1.1 — 2026-10-20

- Parse compositor-provided XKB keymaps for Wayland GUI and input-method input.
- Respect saved `layout` and `restore_keys` values in CLI, GUI, and input method.
- Add XKB parsing and configuration fallback integration tests.
- Correct repeated tone-key semantics when restore keys are disabled.

## 0.1.0 — 2026-10-20

- Initial public beta.
- English, Telex, VNI, and VIQR composition engine.
- Native Wayland GUI and Input Method v2/v1 backends.
- CLI, user configuration, Fedora Atomic documentation, and AppImage packaging.
