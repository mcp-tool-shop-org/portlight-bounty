---
title: Pictures and the license
description: The pictures are MIT, and where the deck props came from.
sidebar:
  order: 4
---

The MIT license covers this repository, including the pictures in `godot/assets/` and `docs/screenshots/`.

## What we know

The landing bundle is manifest 0.4.3. The manifest file is the list.

The barrel, bollard, cart, crate, and torch are built by `build_prop()` in ai-rpg-stage, file `assets/dimetric/camera/dimetric_60_45.py`, from Blender primitives. That function landed in commit `55ac9b2`. The plates were copied into this bundle and not remade. The manifest called that copy a vendor drop.

The three quay flags were painted on a local ComfyUI setup. The recorded pipeline is a procedural base, an img2img pass with a Qwen Image weight and the Salt Road LoRA, then a fixed color map applied with no new generation. The manifest does not call that pass Comfy Cloud.

There is no ElevenLabs voice in this release.

## Credit

Code and art in this release are credited to mcp-tool-shop contributors. That is the public name. The copyright line on the code is mcp-tool-shop, 2026.

## A batch that is not in the game

A later starter batch of twelve pictures (props, tiles, and three UI pieces) was generated for review and is not in this tag. Several of those UI pieces were not game-ready, and the props were on a grey background. They are not part of 0.1.0.
