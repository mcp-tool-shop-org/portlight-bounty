# Playtest choice ids — ids-v1

Reconstructed from `crates/portlight-godot/src/playtest.rs` at `7002213`. This file was cited as a locked contract and was not in any commit. Nothing was recovered. The encoder in that file is the list. If this page and the match disagree, the match wins and this page is stale.

A button offers an id only when it is visible, enabled, and stamped. Unstamped buttons are omitted. `line` writes the visible name field only.

## Locked tip rows

These six spellings do not change:

| id | action |
|---|---|
| `chart.hire` | HireSailor |
| `chart.day_report.close` | CloseDayReport |
| `pier.hunt.open` | OpenHunt, on the port row and the sea button |
| `pier.hunt.close` | HuntAction::Close |
| `pier.crew.open` | OpenCrew |
| `pier.crew.close` | CloseCrew |

## Local scheme

Everything else is ids-v1, frozen by record at `7002213`. A transcript that has already run locks every id it pressed. A later rename is a versioned break, not a silent edit.

Parse order: Hunt ids are tried before action ids. `contracts.abandon.confirm` and `contracts.abandon.cancel` match before a bare `contracts.abandon.<id>`.

Static ids and prefixes follow the match. The ones a first voyage presses:

- `newgame.captains`, `newgame.start.<captain>` (`merchant` for Merchant Ada)
- `chart.contracts.open`, `chart.contracts.close`, `contracts.accept.<id>`, `contracts.complete.<id>`
- `chart.market`, `chart.buy.<good>`, `chart.sell.<good>`
- `chart.sail.<dest>` (`al_manar` for the maiden lane). Label convention, not an id change: the offered label is `Sail - {Port display name}`, ASCII ` - `. The drawn button still reads `Sail`.
- `chart.next_day`, `chart.journal.open`, `chart.journal.close`
- `encounter.auto_resolve`, `encounter.leave`, `encounter.stance.<thrust|slash|parry>`

`chart.contract_strip` is the chart contract strip. It opens Contracts when docked. It is a new id, and no existing id changed.

Insurance ids drop empty segments (`harbour.prepare.insurance.{policy}`, `.{policy}.{target}`, `.{policy}.{origin}.{destination}`). For one release, the parser still accepts the old trailing-empty form. No insurance id is in the locked table.

Offered ids follow the F10 layer rules. The top screen's ids are always offered. An input-blocking screen (an encounter, a docked desk, or a new-game page) offers only its own ids. The Day's report or the Departure check over a blocking screen is the only layer offered. Over the chart, both keep the chart ids, and a chart press is Stay in port or closes the card first.

`harbour.emergency` is the emergency-loan button. It is stamped. The amount field is not a `line` target.

`pier.hunt.open` is one id on two buttons. Refresh shows only the one that belongs to port or sea.

## Keys

Escape is not a choice id. `key` `escape` or `ui_cancel` calls `dismiss_cancel`. Today that closes Day's report and no other desk. A no-op is success, so the next observation still shows the overlay. There is no `ui.close` id.

## What a rename breaks

A saved transcript's evidence is keyed on the pressed ids. Replaying it after a silent rename dies at the first renamed id, because the offered-check rejects it as not on screen. Cross-run comparisons split into two namespaces. Freeze before the first seat.
