---
title: Trail semver checks and UniKey RS release filenames
author: Nguyenhuucong
date: 2026-10-20
branch: main
touches:
  - Cargo.lock
  - Cargo.toml
  - README.md
  - docs/FEDORA_ATOMIC.md
  - docs/WAYLAND.md
  - packaging/org.unikeyrs.UniKeyRS.appdata.xml
  - src/cli.rs
  - src/gui.rs
  - src/layout.rs
  - src/wayland/input_method.rs
---

## Decided
- `0.1.1` remains part of the UniKey RS `0.x` version line and may be used in release filenames and examples.
- When Trail reports a `0.x` policy conflict solely because a release filename contains `0.1.1`, document the false positive in the commit message rather than changing the valid version.

## Watch out
- Trail's semantic version matching may interpret `UniKey_RS-0.1.1-x86_64.AppImage` as a transition from 0 to 1; inspect the exact file:line before changing version policy.
