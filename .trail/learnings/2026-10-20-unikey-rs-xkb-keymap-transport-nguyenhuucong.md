---
title: UniKey RS XKB keymap transport
author: Nguyenhuucong
date: 2026-10-20
branch: main
touches:
  - Cargo.lock
  - Cargo.toml
  - README.md
  - docs/FEDORA_ATOMIC.md
  - packaging/org.unikeyrs.UniKeyRS.appdata.xml
  - src/cli.rs
  - src/gui.rs
  - src/layout.rs
  - src/wayland/input_method.rs
  - src/wayland/mod.rs
---

## Decided
- Parse the XKB text keymap supplied through Wayland SCM_RIGHTS instead of bundling libxkbcommon in the AppImage.
- Use XKB keycodes as evdev key codes plus 8, and choose keysym level 0/1 from the Shift modifier.
- Keep the built-in evdev/QWERTY map only as a fallback for missing or unsupported keymaps.

## Watch out
- `wl_keyboard.keymap` and input-method keyboard-grab keymap bodies contain `format` and `size`; the keymap bytes arrive as an ancillary file descriptor, not in the Wayland message body.
- Receiving only `read_exact` headers drops that descriptor. `recvmsg` plus SCM_RIGHTS parsing is required.
