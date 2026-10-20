---
title: UniKey RS Wayland and immutable Linux implementation choices
author: Nguyenhuucong
date: 2026-10-20
branch: master
touches:
  - docs/FEDORA_ATOMIC.md
  - packaging/org.unikeyrs.UniKeyRS.appdata.xml
  - packaging/unikey-rs.desktop
  - scripts/build-appimage.sh
  - src/gui.rs
  - src/wayland/mod.rs
  - packaging/org.unikeyrs.UniKeyRS.desktop
---

## Decided
- Keep layout composition independent from every CLI/GUI/compositor backend.
- Prefer `zwp_input_method_manager_v2`/`zwp_input_method_v2` when available, including COSMIC, and fall back to `zwp_input_method_v1`.
- Use a native dependency-free `xdg-shell` + `wl_shm` GUI so immutable distributions do not need GTK/Qt packages.
- Package `0.x` releases as AppImages and never modify Fedora Atomic's immutable `/usr`.
- Use a separate counter for filesystem names; Wayland client object IDs must only be allocated for protocol objects.

## Tried and ruled out
- The current COSMIC Atomic session does not advertise `zwp_input_method_v1`; a v1-only implementation cannot activate there.
- Allocating a Wayland object ID for a temporary shm filename caused `Invalid new_id` and broke `wl_shm_pool.create_buffer`.

## Watch out
- `cmsghdr` control buffers must be pointer-aligned and `msg_controllen` must equal `CMSG_SPACE(sizeof(int))` (24 bytes on x86_64), or Linux returns EINVAL for SCM_RIGHTS.
- Compositor input-method globals vary; `doctor` must be the source of truth.
