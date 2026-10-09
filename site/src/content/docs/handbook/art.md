---
title: Pictures and the license
description: What MIT covers, and what it does not.
sidebar:
  order: 4
---

The MIT license covers the source code: the Rust crates, the Godot scripts, the tools, and the prose. It does not cover the pictures. That is `godot/assets/` and `docs/screenshots/`. You may look at them in this repository. You may not take them for another project. No art license is offered.

## What we know

The landing bundle is manifest 0.4.3. The manifest file is the list.

The three quay flags were painted on a local ComfyUI setup. The recorded pipeline is a procedural base, an img2img pass with a Qwen Image weight and the Salt Road LoRA, then a fixed color map applied with no new generation. The manifest does not call that pass Comfy Cloud.

The five deck props (barrel, bollard, cart, crate, and torch) are Blender 5.2 EEVEE plates that were vendored in, not remade. The commit that landed them does not name a modeler and does not name a license. Until that record exists, they stay outside the MIT grant with the rest of the pictures.

There is no ElevenLabs voice in this release. Account terms at a generator bind the account that made a file. They are not permission for anyone else.

## Credit

Code and art in this release are credited to mcp-tool-shop contributors. That is the public name. The copyright line on the code is mcp-tool-shop, 2026.

## A batch that is not in the game

A later starter batch of twelve pictures (props, tiles, and three UI pieces) was generated for review and is not in this tag. Several of those UI pieces were not game-ready, and the props were on a grey background. They are not part of 0.1.0.
