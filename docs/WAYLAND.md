# Wayland notes

## Native window

The GUI uses these core globals directly:

- `wl_compositor` / `wl_surface`
- `wl_shm` / `wl_shm_pool` / `wl_buffer`
- `wl_seat` / `wl_pointer` / `wl_keyboard`
- `xdg_wm_base` / `xdg_surface` / `xdg_toplevel`

This keeps the GUI native to Wayland rather than routing through XWayland.

## Input method

`unikey-rs input-method` prefers `zwp_input_method_manager_v2` and the
corresponding `zwp_input_method_v2` keyboard grab. It falls back to
`zwp_input_method_v1`, waits for the compositor-created context, and uses that
protocol's keyboard grab. Both backends compose pressed keys and send
pre-edit/commit strings.

Wayland compositors differ substantially:

- some advertise no input-method protocol;
- some expose `zwp_input_method_v2`;
- experimental `xx_input_method_v1` has a different activation/commit model;
- desktop portals or compositor settings may also control which input method
  is allowed.

Always run `unikey-rs doctor`. Its global list is authoritative for the current
session.

## Security

Ordinary Wayland clients cannot type into unrelated clients. That isolation is
a feature. Do not add a root/evdev injector to bypass it; add a supported
input-method, text-input, virtual-keyboard, or platform accessibility backend
instead.

## Keyboard maps

Compositor XKB keymaps are parsed by `src/wayland/xkb.rs`. UniKey RS reads the
keymap file descriptor delivered through SCM_RIGHTS and resolves the normal and
Shift keysym levels. The built-in evdev/QWERTY map is used only when a keymap is
missing or unsupported.
