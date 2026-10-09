---
title: The voyage
description: The chart, the port row, contracts, and how a fight counts closes.
sidebar:
  order: 2
---

The chart draws the current port's lanes. Sail, Next day, and the market call the simulation. They do not invent a price or a legal lane on their own.

While you are docked, one row holds Market, Contracts, Hire, Stores, Work, Shipyard, Harbour, Crew, and Hunt. At sea that row hides. Hunt then sits with Next day, Save, and Journal.

## The two frames in the README

The docked chart at Al-Manar, market open, is `docs/screenshots/chart-1280.png`.

The active contract strip, two contracts with cargo progress and days left, is `docs/screenshots/contract-strip-active.png`. Both are the frames CI compares. They are the right pictures for this release.

![Docked chart at Al-Manar](https://github.com/mcp-tool-shop-org/portlight-bounty/raw/main/docs/screenshots/chart-1280.png)

![Active contract strip](https://github.com/mcp-tool-shop-org/portlight-bounty/raw/main/docs/screenshots/contract-strip-active.png)

## Contracts

The strip shows a contract you have accepted: the name, how much of the cargo you have delivered, how much you owe, and the days left. The Contracts desk is where you accept one and, after a confirm, abandon one. Abandoning throws away the progress on that contract.

## A fight

Negotiate, flee, guns, and boarding are on the encounter screen. The chart itself does not resolve a boarding.

Boarding progress rises when either ship closes and the other does not evade. Your close counts. Their close counts. If both close on the same turn, the count moves by two. Evade keeps the other ship's close from counting.

The line that opens the fight names the threshold as a number of close actions. It does not say whose closes those are. Read the count as either ship, not as your button alone.

## The day's report

When the day has something to say, the report opens over the chart. A press on the chart closes the card first. A quiet day hides it. The report can also open over the Contracts desk, which is dimmed and does not take clicks until the card closes.

## Work, stores, hunt

Work pays a few silver and does not tick the market. Stores buy provisions. Hunt forages, and the bounty board is a separate list of captains. Claiming a bounty waits until that captain is defeated.
