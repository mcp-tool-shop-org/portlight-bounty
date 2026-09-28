//! Victory loot. Port of `engine/loot.py`.
//!
//! `apply_loot` does not roll weapon quality. That matches Python: the victory
//! path calls `roll_loot` then `apply_loot` and never `assign_loot_quality`.

use crate::content::{self, LootTableDef};
use crate::model::{Armor, Captain, CargoItem, Weapon};
use crate::pyrand::PyRandom;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootDrop {
    pub item_type: String,
    pub item_id: String,
    pub quantity: i64,
}

pub fn roll_loot(
    opponent_strength: i64,
    opponent_id: Option<&str>,
    rng: &mut PyRandom,
    num_drops: i64,
) -> Vec<LootDrop> {
    let catalog = content::content();
    let table = opponent_id
        .and_then(|id| catalog.captain_loot(id))
        .or_else(|| catalog.loot_table_for_strength(opponent_strength));
    let Some(table) = table else {
        return Vec::new();
    };
    let num_drops = if opponent_strength >= 9 {
        3
    } else if opponent_strength >= 7 {
        num_drops.max(2)
    } else {
        num_drops
    };
    roll_table(table, rng, num_drops)
}

fn roll_table(table: &LootTableDef, rng: &mut PyRandom, num_drops: i64) -> Vec<LootDrop> {
    let weights: Vec<f64> = table.entries.iter().map(|entry| entry.weight).collect();
    let mut drops = Vec::new();
    let mut seen_silver = false;
    if let Some(silver) = table
        .entries
        .iter()
        .find(|entry| entry.item_type == "silver")
    {
        let qty = rng.randint(silver.quantity_min, silver.quantity_max);
        drops.push(LootDrop {
            item_type: "silver".to_string(),
            item_id: "silver".to_string(),
            quantity: qty,
        });
        seen_silver = true;
    }
    for _ in 0..num_drops {
        if table.entries.is_empty() {
            break;
        }
        let chosen = &table.entries[rng.choices_weighted(&weights)];
        if chosen.item_type == "silver" && seen_silver {
            if let Some(first) = drops.first_mut() {
                first.quantity += rng.randint(chosen.quantity_min, chosen.quantity_max);
            }
            continue;
        }
        let qty = rng.randint(chosen.quantity_min, chosen.quantity_max);
        if chosen.item_type == "silver" {
            seen_silver = true;
        }
        drops.push(LootDrop {
            item_type: chosen.item_type.clone(),
            item_id: chosen.item_id.clone(),
            quantity: qty,
        });
    }
    drops
}

pub fn apply_loot(captain: &mut Captain, loot: &[LootDrop]) -> Vec<String> {
    let catalog = content::content();
    let mut messages = Vec::new();
    for drop in loot {
        match drop.item_type.as_str() {
            "silver" => {
                captain.silver += drop.quantity;
                messages.push(format!("+{} silver", drop.quantity));
            }
            "cargo" => {
                if let Some(existing) = captain
                    .cargo
                    .iter_mut()
                    .find(|item| item.good_id == drop.item_id)
                {
                    existing.quantity += drop.quantity;
                } else {
                    captain.cargo.push(CargoItem {
                        good_id: drop.item_id.clone(),
                        quantity: drop.quantity,
                        cost_basis: 0,
                        acquired_port: "loot".to_string(),
                        acquired_region: "pirate".to_string(),
                        acquired_day: 0,
                    });
                }
                let name = catalog
                    .good(&drop.item_id)
                    .map(|good| good.name.as_str())
                    .unwrap_or(drop.item_id.as_str());
                messages.push(format!("+{} {name}", drop.quantity));
            }
            "melee_weapon" => {
                if let Some(def) = catalog.melee_weapon(&drop.item_id) {
                    captain.melee = Some(Weapon {
                        id: def.id.clone(),
                        name: def.name.clone(),
                        kind: "melee".to_string(),
                        quality: captain.quality_of(&def.id).to_string(),
                        ammo: 0,
                    });
                    messages.push(format!("Found: {}", def.name));
                }
            }
            "armor" => {
                if let Some(def) = catalog.armor(&drop.item_id) {
                    captain.armor = Some(Armor {
                        id: def.id.clone(),
                        name: def.name.clone(),
                        armor_type: def.armor_type.clone(),
                        damage_reduction: def.damage_reduction,
                        dodge_penalty: def.dodge_penalty,
                        stamina_penalty: def.stamina_penalty,
                        quality: captain.quality_of(&def.id).to_string(),
                    });
                    messages.push(format!("Found: {}", def.name));
                }
            }
            "ranged_weapon" => {
                if let Some(def) = catalog.ranged_weapon(&drop.item_id) {
                    let weapon = Weapon {
                        id: def.id.clone(),
                        name: def.name.clone(),
                        kind: def.weapon_type.clone(),
                        quality: captain.quality_of(&def.id).to_string(),
                        ammo: 0,
                    };
                    if def.weapon_type == "firearm" {
                        captain.firearm = Some(weapon);
                    } else if def.weapon_type == "mechanical" {
                        captain.mechanical = Some(weapon);
                    }
                    messages.push(format!("Found: {}", def.name));
                }
            }
            "ammo" => {
                if let Some(ammo) = catalog.ammo(&drop.item_id) {
                    let gained = ammo.quantity * drop.quantity;
                    if ammo.weapon_type == "firearm" {
                        if let Some(weapon) = captain.firearm.as_mut() {
                            weapon.ammo += gained;
                        }
                    } else if ammo.weapon_type == "mechanical" {
                        if let Some(weapon) = captain.mechanical.as_mut() {
                            weapon.ammo += gained;
                        }
                    }
                    messages.push(format!("+{gained} {}", ammo.name));
                }
            }
            _ => {}
        }
    }
    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::new_game;

    #[test]
    fn named_captain_override_always_includes_silver() {
        let mut rng = PyRandom::from_seed(4);
        let drops = roll_loot(6, Some("raj_the_quiet"), &mut rng, 2);
        assert!(drops.iter().any(|drop| drop.item_type == "silver"));
        assert!(!drops.is_empty());
    }

    #[test]
    fn cargo_loot_merges_the_first_stack_and_stamps_pirate_origin() {
        let mut captain = new_game("Ada", "merchant", 1, None).unwrap().captain;
        captain.cargo.push(CargoItem {
            good_id: "grain".into(),
            quantity: 2,
            cost_basis: 8,
            acquired_port: "porto_novo".into(),
            acquired_region: "Mediterranean".into(),
            acquired_day: 1,
        });
        let drops = vec![
            LootDrop {
                item_type: "silver".into(),
                item_id: "silver".into(),
                quantity: 12,
            },
            LootDrop {
                item_type: "cargo".into(),
                item_id: "grain".into(),
                quantity: 3,
            },
            LootDrop {
                item_type: "cargo".into(),
                item_id: "rum".into(),
                quantity: 1,
            },
        ];
        let notes = apply_loot(&mut captain, &drops);
        assert_eq!(captain.silver, 550 + 12);
        assert_eq!(captain.cargo[0].quantity, 5);
        assert_eq!(captain.cargo[0].acquired_port, "porto_novo");
        assert_eq!(captain.cargo[1].acquired_port, "loot");
        assert_eq!(captain.cargo[1].acquired_region, "pirate");
        assert_eq!(captain.cargo[1].cost_basis, 0);
        assert!(notes.iter().any(|note| note.contains("silver")));
    }
}
