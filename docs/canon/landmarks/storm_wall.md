# The Storm Wall (storm_wall)

Base commit: 71c6aaa (content.json and voyage.rs unchanged since 45b7378). Status: draft canon, phase 2. Tables are hand-maintained from content.json; there is no generator.
Cites `C:n` are crates/portlight-sim/data/content.json line n. Legend: [cite] grounded | [invented] | [flavour, no mechanic] | [ascii-folded] | [proposal] | DF-n see ../data-fixes.md | T-n see ../README.md shared timeline.

## 0. At a glance

| Key | Value | Cite |
|---|---|---|
| Category | A (port landmark) | - |
| Port | [Typhoon Anchorage](../ports/typhoon_anchorage.md) | - |
| Region | South Seas | C:1750 |
| Shipped text | "The Storm Wall - a massive breakwater built from the hulls of ships that didn't make it." [ascii-folded] | C:12187 |
| Sea mark | none | voyage.rs:592-607 |
| Player sees it | yes, as the Landmark line on arrival | crates/portlight-sim/src/port_arrival_engine.rs:157-159 |
| Mechanic | none [flavour, no mechanic] | - |
| Also an institution | `ta_wall` | C:11044 |

## 1. Location (all [invented] unless a cite or a named data field is given)

A massive breakwater built from wrecked hulls (landmark), also the governor's seat (C:11044). The Wreck Yard (C:11064) builds from its wrecks.

No coordinates exist for landmarks; the chart position is the port's own map x/y. Placing landmarks on the chart is a future phase.

## 2. Data cites

- dock_scene: "The harbor is carved into the leeward cliff. Ships are chained, not anchored - anchors aren't enough here. Pearl divers work the outer reef between storms." (C:12190)
- atmosphere: "Wind that never fully dies, salt-crusted everything, and the pride of survival." (C:12189)
- tavern_rumor: "A captain carved her name into the Storm Wall seven times - one for each typhoon survived. They call her the Unkillable. She drinks alone." (C:12191)
- local_custom: "Captains who survive a typhoon carve their ship's name into the Storm Wall. It's an honor." (C:12188)
- named in `description` at C:11028
- named in `name` at C:11045
- named in `agenda` at C:11093
- named in `rumor` at C:11097
- named in `description` at C:11155
- named in `agenda` at C:11156
- named in `rumor` at C:11181
- named in `local_custom` at C:12188
- named in `tavern_rumor` at C:12191
- named in `departure_text` at C:13183
- regional festival: Coral Coronation (C:11976)
- regional festival: Fire Walking (C:11990)
- named lane: Typhoon Alley (Monsoon Reach - Typhoon Anchorage, C:2230) - lanes are deferred to a later phase

## 3. History (all [invented] unless cited)

- T20: the first wrecks are lashed into a breakwater after a great typhoon [invented].
- Captains carve their names for each typhoon survived; Rangi's seventh carving (C:11044 area).

## 4. Appearance and palette (art guidance only; commissions nothing; the character hold stands)

A long curve of stacked dark hulls, ribs and keels, chained together, names cut in the timber. [invented]

Palette: drawn from the Typhoon Anchorage port palette ([port file section C.3](../ports/typhoon_anchorage.md)), so the landmark reads as part of its port. Values and statuses are the port file's; judged in sRGB against the envelope (luma 65-141, chroma p98 35; small-area accents up to chroma 64). No sea or water swatch.

| Role | Port swatch | sRGB | Luma / chroma | Status |
|---|---|---|---|---|
| Mass | storm-wet timber (Dominant) | (92, 84, 76) #5C544C | 85 / 16 | in range |
| Detail | typhoon sky grey (Secondary 1) | (104, 108, 112) #686C70 | 107 / 8 | in range |
| Accent | reef stone (Secondary 2) | (110, 104, 96) #6E6860 | 105 / 14 | in range |

Kit: compose from the shipped will-use plates; no named landmark has art today. Any new plate is a gap for the art gap list, not a commission.

## 5. People and use (all [invented] unless cited)

Survivors who carve, the governor, Wreck Yard builders.

## 6. Sound hook (all [invented] sound design unless quoted from a cite)

- Chains straining; surf through hull timbers.
- Fragments: none proposed yet [proposal]. No music, no voice; #45 ear hold unaffected.

## 7. Links to ports

- Owning port: [Typhoon Anchorage](../ports/typhoon_anchorage.md)
- [Monsoon Reach](../ports/monsoon_reach.md)
- [Pearl Shallows](../ports/pearl_shallows.md)
- [Coral Throne](../ports/coral_throne.md)

## 8. Flags

- Not Stormwall the North Atlantic port.
- The exchange 'The Deep Reef' (C:11034) and the pirate 'Deep Reef Brotherhood' (C:2896) share a name. Resolved: the Brotherhood is named for the reef ground; no tie to the Anchorage's exchange.
