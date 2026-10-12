---
title: Portlight Bounty
description: What this game is, and what version 0.1.0 includes.
sidebar:
  order: 0
---

Portlight Bounty is a trade game. You buy goods, sail a lane, take a contract, and try to arrive with silver still in the box. The Rust simulation is the rules. The Godot chart is the game you sit down at.

This is the handbook for version 0.1.0, the first GitHub release. The Python game that the rules are checked against is [Portlight](https://github.com/mcp-tool-shop-org/portlight) at commit `9b02494`.

## Start here

1. [Play the chart](./getting-started/) if you want a ship on the water.
2. [The voyage](./play/) if you want to know what the buttons do.
3. [The simulation](./simulation/) if you are checking a number.
4. [Pictures and the license](./art/) before you reuse anything you see.
5. [Commands](./reference/) for the script runner.

## What 0.1.0 includes

A Mediterranean chart. New game and load. Sail, the market, contracts, crew, the yard, the harbour desk, hunt, the journal, the day's report, and a fight at sea.

The plates in the landing bundle are listed in `godot/assets/landing/MANIFEST.json` (manifest 0.4.3). An older note that said "70 PNGs, v0.3.0" does not describe this release.

Draft pull requests that add extra fight and contract lines are not in this tag.

## What it does not do

It does not phone home. It does not have an account. It does not send telemetry. A macOS build has not been run for this release.
