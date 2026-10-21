---
title: UniKey RS Gear Lever integration
author: Nguyenhuucong
date: 2026-10-21
branch: main
touches:
  - README.md
  - README.vi.md
  - packaging/org.unikeyrs.UniKeyRS.desktop
  - scripts/build-appimage.sh
  - docs/GEAR_LEVER.md
---

## Decided
- Manage and integrate the UniKey RS AppImage through Gear Lever rather than creating duplicate manual desktop entries.
- Ship `X-AppImage-Version` in the AppImage desktop entry so Gear Lever displays and distinguishes release versions.
- Configure Gear Lever's `GithubUpdater` with `repo=unikey-rs/unikey-rs`, prereleases enabled, and `repo_filename=UniKey_RS-*-x86_64.AppImage`.

## Tried and ruled out
- A filename without embedded version caused Gear Lever to use an MD5 suffix and display `Not specified`.
- `--replace` did not merge entries when the old entry lacked version metadata; remove the stale entry and integrate the versioned AppImage instead.

## Watch out
- Gear Lever's update-source config is tied to the managed AppImage path. Reconfigure it after replacing the managed file.
