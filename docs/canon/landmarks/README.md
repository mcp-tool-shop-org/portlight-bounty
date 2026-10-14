# Landmark canon index

Base commit: 71c6aaa. Status: draft canon, phase 2. Hand-maintained from content.json and voyage.rs; there is no generator.

Phase 2 covers category A (the one shipped `landmark` per port, C:12007-12197) and category B (sea marks named only in the voyage lighthouse event, crates/portlight-sim/src/voyage.rs:594-598). Landmarks have no mechanic [flavour, no mechanic].

| Landmark | Port | Category | Sea mark | Shipped cite | File |
|---|---|---|---|---|---|
| The Grain Exchange | Porto Novo | A | - | C:12007 | [grain_exchange.md](grain_exchange.md) |
| The beacon of Porto Novo | Porto Novo | B | voyage.rs:594 | voyage.rs:594 | [porto_novo_beacon.md](porto_novo_beacon.md) |
| The Spice Bazaar | Al-Manar | A | - | C:12017 | [spice_bazaar.md](spice_bazaar.md) |
| The Master Shipyard | Silva Bay | A | - | C:12027 | [master_shipyard.md](master_shipyard.md) |
| The Cliff Tavern | Corsair's Rest | A | - | C:12037 | [cliff_tavern.md](cliff_tavern.md) |
| The Great Foundry | Ironhaven | A | voyage.rs:595 | C:12047 | [great_foundry.md](great_foundry.md) |
| The Fortress | Stormwall | A | - | C:12057 | [stormwall_fortress.md](stormwall_fortress.md) |
| The Whale Arch | Thornport | A | voyage.rs:598 | C:12067 | [whale_arch.md](whale_arch.md) |
| The Cotton Steps | Sun Harbor | A | - | C:12077 | [cotton_steps.md](cotton_steps.md) |
| The Distillery Row | Palm Cove | A | - | C:12087 | [distillery_row.md](distillery_row.md) |
| The River Mouth Mines | Iron Point | A | - | C:12097 | [river_mouth_mines.md](river_mouth_mines.md) |
| The Blessing Pool | Pearl Shallows | A | - | C:12107 | [blessing_pool.md](blessing_pool.md) |
| The Porcelain Quarter | Jade Port | A | - | C:12117 | [porcelain_quarter.md](porcelain_quarter.md) |
| The Wind Temple | Monsoon Reach | A | voyage.rs:596 | C:12127 | [wind_temple.md](wind_temple.md) |
| The Loom Quarter | Silk Haven | A | - | C:12137 | [loom_quarter.md](loom_quarter.md) |
| The Free Port Bell | Crosswind Isle | A | - | C:12147 | [free_port_bell.md](free_port_bell.md) |
| The Gate Fortress | Dragon's Gate | A | - | C:12157 | [gate_fortress.md](gate_fortress.md) |
| The Hanging Market | Spice Narrows | A | - | C:12167 | [hanging_market.md](hanging_market.md) |
| Ember Peak | Ember Isle | A | voyage.rs:597 | C:12177 | [ember_peak.md](ember_peak.md) |
| The Storm Wall | Typhoon Anchorage | A | - | C:12187 | [storm_wall.md](storm_wall.md) |
| The Coral Palace | Coral Throne | A | - | C:12197 | [coral_palace.md](coral_palace.md) |

## Named lanes (category C)

The 15 named lanes have their own canon from phase 3: see [../lanes/README.md](../lanes/README.md).

## Not in data (future phase)

- No reefs, rocks, shoals, wrecks (other than the Storm Wall), lighthouses (other than the code-only Porto Novo beacon) or forts exist as data objects. Forts exist only as landmark prose.
- No landmark has coordinates, an icon, art or a sound. Placing landmarks on the chart and any landmark art or sound are a future phase and need their own brief.

## Id note

Landmark ids are canon-only, never data ids. Stormwall's 'The Fortress' uses `stormwall_fortress` (not `fortress`) so it cannot be confused with Dragon's Gate's `gate_fortress`.

Sea marks are visual voyage text. Any future sound for the lighthouse event is a proposal and must not reuse port bells or shipped chart cues (sfx_chart_sail_depart).
