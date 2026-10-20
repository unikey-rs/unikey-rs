# Architecture

UniKey RS keeps policy, presentation, and compositor integration separate:

1. `src/layout.rs` owns raw keystroke composition for English, Telex, VNI, and
   VIQR. It is deterministic and independent of Wayland or any GUI toolkit.
2. `src/config.rs` stores a forward-compatible key/value configuration.
3. `src/cli.rs` provides the stable agent/automation surface.
4. `src/gui.rs` is a native Wayland `xdg-shell` + `wl_shm` software-rendered
   control panel.
5. `src/wayland/mod.rs` is a small dependency-free Wayland wire transport.
6. `src/wayland/input_method.rs` maps compositor keyboard-grab events into the
   shared composer and emits pre-edit/commit requests.

## Why a dependency-free GUI

Fedora Atomic and other immutable systems vary widely in installed desktop
toolkits. The GUI only depends on the Wayland socket and standard Linux IPC,
so the AppImage does not have to bundle GTK, Qt, or a particular display stack.

## Backend boundary

A Wayland backend receives characters/keys and calls `Composer`; it does not
reimplement accents. New platform backends for Windows, macOS, X11, or a newer
Wayland input-method protocol should retain that boundary.

## Beta/version policy

The project is `0.x` while cross-platform support and protocol integration are
still maturing. `1.0` is reserved for a nearly complete, highly mature release
supporting Linux, macOS, and Windows.
