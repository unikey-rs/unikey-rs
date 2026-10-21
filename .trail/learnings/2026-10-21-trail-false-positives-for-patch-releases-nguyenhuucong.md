---
title: Trail false positives for patch releases and platform labels
author: Nguyenhuucong
date: 2026-10-21
branch: main
touches:
  - CHANGELOG.md
  - Cargo.lock
  - Cargo.toml
  - README.md
  - docs/FEDORA_ATOMIC.md
  - packaging/org.unikeyrs.UniKeyRS.appdata.xml
  - src/cli.rs
  - src/config.rs
  - src/gui.rs
  - src/lib.rs
---

## Decided
- UniKey RS patch releases such as 0.1.2 remain 0.x even when Trail flags a version-text change as 1-to-2 or 0-to-1.
- A doctor/platform label mentioning COSMIC Atomic is compatible with the existing learning; the learning rules out a v1-only input backend, not COSMIC support.

## Watch out
- Before changing version or backend policy in response to Trail, inspect the exact span and distinguish semantic version digits from protocol/feature text.
