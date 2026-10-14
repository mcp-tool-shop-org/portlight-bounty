# Lane canon index

Base commit: ab03ace. Status: draft canon, phase 3. Hand-maintained from content.json `routes[].lore_name`; there is no generator.

Phase 3 covers the routes that carry a `lore_name` (15 of 43). The name is player-visible on the chart lane label (crates/portlight-chart/src/view.rs:213-223, drawn by crates/portlight-godot/src/chart_canvas.rs:228) and keys a route encounter table (sea_culture.rs:121-136). The other 28 lanes stay unnamed in canon.

| Lane | Ends | Distance | Danger | Min class | Days | Cite | Encounters | File |
|---|---|---|---|---|---|---|---|---|
| The Grain Road | [Porto Novo](../ports/porto_novo.md) - [Al-Manar](../ports/al_manar.md) | 24 | 0.12 | sloop | 3 | C:1933 | 9 (C:12210) | [grain_road.md](grain_road.md) |
| The Timber Run | [Porto Novo](../ports/porto_novo.md) - [Silva Bay](../ports/silva_bay.md) | 16 | 0.08 | sloop | 2 | C:1942 | 4 (C:12260) | [timber_run.md](timber_run.md) |
| The Shadow Lane | [Porto Novo](../ports/porto_novo.md) - [Corsair's Rest](../ports/corsairs_rest.md) | 18 | 0.18 | sloop | 2 | C:1960 | 5 (C:12285) | [shadow_lane.md](shadow_lane.md) |
| The Iron Strait | [Ironhaven](../ports/ironhaven.md) - [Stormwall](../ports/stormwall.md) | 20 | 0.09 | sloop | 2 | C:1978 | 4 (C:12315) | [iron_strait.md](iron_strait.md) |
| The Tea and Tobacco Road | [Stormwall](../ports/stormwall.md) - [Thornport](../ports/thornport.md) | 18 | 0.1 | sloop | 2 | C:1996 | 4 (C:12340) | [tea_and_tobacco_road.md](tea_and_tobacco_road.md) |
| The Cotton Crossing | [Porto Novo](../ports/porto_novo.md) - [Sun Harbor](../ports/sun_harbor.md) | 40 | 0.12 | cutter | 4 | C:2032 | 5 (C:12365) | [cotton_crossing.md](cotton_crossing.md) |
| The Long Crossing | [Sun Harbor](../ports/sun_harbor.md) - [Crosswind Isle](../ports/crosswind_isle.md) | 64 | 0.18 | galleon | 16 | C:2104 | 7 (C:12395) | [long_crossing.md](long_crossing.md) |
| The Porcelain Lane | [Crosswind Isle](../ports/crosswind_isle.md) - [Jade Port](../ports/jade_port.md) | 28 | 0.1 | sloop | 4 | C:2131 | 4 (C:12435) | [porcelain_lane.md](porcelain_lane.md) |
| The Silk Road by Sea | [Jade Port](../ports/jade_port.md) - [Silk Haven](../ports/silk_haven.md) | 18 | 0.07 | sloop | 2 | C:2176 | 4 (C:12460) | [silk_road_by_sea.md](silk_road_by_sea.md) |
| Typhoon Alley | [Monsoon Reach](../ports/monsoon_reach.md) - [Typhoon Anchorage](../ports/typhoon_anchorage.md) | 52 | 0.2 | galleon | 13 | C:2230 | 6 (C:12485) | [typhoon_alley.md](typhoon_alley.md) |
| The Volcanic Passage | [Spice Narrows](../ports/spice_narrows.md) - [Ember Isle](../ports/ember_isle.md) | 48 | 0.18 | galleon | 12 | C:2239 | 5 (C:12520) | [volcanic_passage.md](volcanic_passage.md) |
| The Monsoon Shortcut | [Al-Manar](../ports/al_manar.md) - [Monsoon Reach](../ports/monsoon_reach.md) | 72 | 0.22 | galleon | 18 | C:2284 | 5 (C:12550) | [monsoon_shortcut.md](monsoon_shortcut.md) |
| The Smuggler's Run | [Corsair's Rest](../ports/corsairs_rest.md) - [Spice Narrows](../ports/spice_narrows.md) | 80 | 0.25 | galleon | 20 | C:2293 | 6 (C:12580) | [smugglers_run.md](smugglers_run.md) |
| The Northern Passage | [Ironhaven](../ports/ironhaven.md) - [Jade Port](../ports/jade_port.md) | 76 | 0.22 | galleon | 19 | C:2302 | 5 (C:12615) | [northern_passage.md](northern_passage.md) |
| The Deep South Run | [Pearl Shallows](../ports/pearl_shallows.md) - [Coral Throne](../ports/coral_throne.md) | 68 | 0.2 | galleon | 17 | C:2311 | 5 (C:12645) | [deep_south_run.md](deep_south_run.md) |

Days = max(1, round(distance / minimum-class speed)) (voyage.rs:726-728); faster ships take fewer days. The game shows days for the player's own ship.

## Rules

- Lane ids are canon-only, never data ids. The data key is the lore_name string itself (C:12210 area); renaming a lane in data breaks its encounter table, so canon never proposes renames.
- Superlatives in data ('oldest', 'longest', 'gentlest') are checked against the 43 routes; contradictions are DF-23 to DF-25.
- Saves drop lane names (DF-22). Canon describes the named-lane experience of a fresh session.
- Long-haul open-ocean reading of three west-east lanes stands (DF-21).

## Not in data (future phase)

- No drawn courses, waypoints, hazards or lane art. No lane sound.
- The 28 unnamed lanes get no canon names; naming one is a sim PR first.
