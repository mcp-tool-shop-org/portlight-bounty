---
title: Play the chart
description: Rust 1.98.1, Godot 4.7.2, and the first voyage as Ada.
sidebar:
  order: 1
---

## What you need

- Rust 1.98.1. `rust-toolchain.toml` pins it, and `cargo` will install that toolchain. That pin builds the simulation and the Godot extension.
- Godot 4.7.2, the official build. Linux CI uses that editor. A Windows build of the extension has been run. A macOS build has not.

The Godot CI job sets `RUSTUP_TOOLCHAIN=stable`, which overrides the pin for that job only. The editor version in CI is still 4.7.2.

## Build and open

From the repository root:

```bash
cargo build -p portlight-godot
godot --path godot
```

The extension looks for the debug library under `target/debug/`. A release-only build does not satisfy the editor. There is no export preset in this release, so you play from the editor.

## The first voyage

New game. Choose The Merchant. Leave the name as Ada. That is the voyage in the screenshots: Porto Novo, then the lanes the chart will draw.

Sail uses the lane list. Days on the picker are the ship's raw speed. Crew, morale, season, and captain modifiers are not in that estimate. A lane can be listed and then refused when you sail it. A rank gap of 1 is a warning. A gap of 2 or more is blocked. Both of those match the Python game. Leave them.

## Save and load

Save sits with New game and writes a version-12 slot. Load lists the captain and the day. A loaded custom captain keeps the name, the type, the silver, and the day. Version 12 does not store the custom template, so prices and voyage modifiers after a load are the merchant's. That is the same limit as Python.

The chart writes `saves/` under the working directory when you launch Godot from this checkout. That folder is gitignored.

## If the window is empty

Headless Godot cannot draw. A screenshot needs a real GL context. On Linux CI that is llvmpipe under Xvfb. Do not treat a headless run that says the viewport was empty as a broken chart.
