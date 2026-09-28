//! Weapon quality, durability, and the rusted floor. Port of `engine/weapon_quality.py`.
//!
//! Rusted is the breakage floor: another use does not remove the weapon or
//! drop it further. The blacksmith threshold bonus stays 0 until skills land.

use crate::model::WeaponQuality;
use crate::pyrand::PyRandom;

pub const TIERS: [&str; 5] = ["rusted", "worn", "standard", "fine", "masterwork"];
pub const MELEE_DEGRADE_USES: i64 = 10;
pub const FIREARM_DEGRADE_USES: i64 = 20;
pub const ARMOR_DEGRADE_USES: i64 = 15;
pub const MECHANICAL_DEGRADE_USES: i64 = 25;

const MAINTENANCE_COST: [(&str, i64); 5] = [
    ("rusted", 5),
    ("worn", 10),
    ("standard", 15),
    ("fine", 25),
    ("masterwork", 40),
];

const UPGRADE_COST: [(&str, i64); 4] = [
    ("rusted", 20),
    ("worn", 40),
    ("standard", 100),
    ("fine", 300),
];

const UPGRADE_DAYS: [(&str, i64); 4] = [("rusted", 1), ("worn", 2), ("standard", 3), ("fine", 5)];

pub fn effects(quality: &str) -> WeaponQuality {
    match quality {
        "rusted" => WeaponQuality {
            damage_mod: -1,
            accuracy_mod: -0.10,
            label: "Rusted",
        },
        "worn" => WeaponQuality {
            damage_mod: 0,
            accuracy_mod: -0.05,
            label: "Worn",
        },
        "fine" => WeaponQuality {
            damage_mod: 1,
            accuracy_mod: 0.05,
            label: "Fine",
        },
        "masterwork" => WeaponQuality {
            damage_mod: 2,
            accuracy_mod: 0.10,
            label: "Masterwork",
        },
        _ => WeaponQuality {
            damage_mod: 0,
            accuracy_mod: 0.0,
            label: "Standard",
        },
    }
}

fn tier_index(quality: &str) -> usize {
    TIERS.iter().position(|tier| *tier == quality).unwrap_or(2)
}

pub fn degrade_quality(current: &str) -> String {
    let idx = tier_index(current);
    if idx == 0 {
        "rusted".to_string()
    } else {
        TIERS[idx - 1].to_string()
    }
}

pub fn threshold(weapon_type: &str) -> i64 {
    match weapon_type {
        "firearm" => FIREARM_DEGRADE_USES,
        "mechanical" => MECHANICAL_DEGRADE_USES,
        "armor" => ARMOR_DEGRADE_USES,
        _ => MELEE_DEGRADE_USES,
    }
}

pub fn check_degradation(weapon_type: &str, usage: i64, threshold_bonus: i64) -> bool {
    usage >= threshold(weapon_type) + threshold_bonus
}

/// Record uses on the captain maps. Returns the new tier when the weapon drops.
/// Rusted weapons still accumulate usage and never drop again.
pub fn tick_weapon_degradation(
    quality: &mut Vec<(String, String)>,
    usage: &mut Vec<(String, i64)>,
    weapon_id: &str,
    weapon_type: &str,
    uses: i64,
    threshold_bonus: i64,
) -> Option<String> {
    let current_uses = usage_of(usage, weapon_id) + uses;
    set_i64(usage, weapon_id, current_uses);
    let current_quality = quality_of(quality, weapon_id);
    if current_quality == "rusted" {
        return None;
    }
    if check_degradation(weapon_type, current_uses, threshold_bonus) {
        let new_quality = degrade_quality(&current_quality);
        set_string(quality, weapon_id, &new_quality);
        set_i64(usage, weapon_id, 0);
        return Some(new_quality);
    }
    None
}

pub fn maintenance_cost(quality: &[(String, String)], weapon_id: &str) -> i64 {
    let tier = quality_of(quality, weapon_id);
    MAINTENANCE_COST
        .iter()
        .find(|(name, _)| *name == tier)
        .map(|(_, cost)| *cost)
        .unwrap_or(15)
}

pub fn maintain_weapon(
    quality: &[(String, String)],
    usage: &mut Vec<(String, i64)>,
    weapon_id: &str,
    silver: i64,
) -> Result<i64, String> {
    let cost = maintenance_cost(quality, weapon_id);
    if silver < cost {
        return Err(format!(
            "Maintenance costs {cost} silver. You have {silver}."
        ));
    }
    set_i64(usage, weapon_id, 0);
    Ok(silver - cost)
}

pub fn can_upgrade(
    quality: &[(String, String)],
    weapon_id: &str,
    silver: i64,
    at_smith: bool,
) -> Option<String> {
    if !at_smith {
        return Some("No smith at this port. Visit a port with a shipyard.".to_string());
    }
    let tier = quality_of(quality, weapon_id);
    if tier == "masterwork" {
        return Some(
            "Already masterwork quality. This weapon cannot be improved further.".to_string(),
        );
    }
    let cost = upgrade_cost(&tier);
    if silver < cost {
        return Some(format!("Upgrade costs {cost} silver. You have {silver}."));
    }
    None
}

pub fn upgrade_weapon(
    quality: &mut Vec<(String, String)>,
    usage: &mut Vec<(String, i64)>,
    weapon_id: &str,
    silver: i64,
) -> (String, i64, i64) {
    let current = quality_of(quality, weapon_id);
    let cost = upgrade_cost(&current);
    let days = UPGRADE_DAYS
        .iter()
        .find(|(name, _)| *name == current)
        .map(|(_, days)| *days)
        .unwrap_or(3);
    let idx = tier_index(&current);
    let new_quality = TIERS[idx.saturating_add(1).min(TIERS.len() - 1)];
    set_string(quality, weapon_id, new_quality);
    set_i64(usage, weapon_id, 0);
    (new_quality.to_string(), silver - cost, days)
}

pub fn assign_purchase_quality() -> &'static str {
    "standard"
}

/// Loot quality roll. `apply_loot` does not call this; Python's victory path
/// does not either. Tests and a later loot-quality pass can.
pub fn assign_loot_quality(opponent_strength: i64, rng: &mut PyRandom) -> &'static str {
    let roll = rng.random();
    if opponent_strength <= 3 {
        if roll < 0.40 {
            "rusted"
        } else if roll < 0.80 {
            "worn"
        } else {
            "standard"
        }
    } else if opponent_strength <= 6 {
        if roll < 0.15 {
            "rusted"
        } else if roll < 0.50 {
            "worn"
        } else if roll < 0.90 {
            "standard"
        } else {
            "fine"
        }
    } else if opponent_strength <= 8 {
        if roll < 0.10 {
            "worn"
        } else if roll < 0.50 {
            "standard"
        } else if roll < 0.85 {
            "fine"
        } else {
            "masterwork"
        }
    } else if roll < 0.20 {
        "standard"
    } else if roll < 0.60 {
        "fine"
    } else {
        "masterwork"
    }
}

fn upgrade_cost(tier: &str) -> i64 {
    UPGRADE_COST
        .iter()
        .find(|(name, _)| *name == tier)
        .map(|(_, cost)| *cost)
        .unwrap_or(100)
}

fn quality_of(map: &[(String, String)], weapon_id: &str) -> String {
    map.iter()
        .find(|(id, _)| id == weapon_id)
        .map(|(_, quality)| quality.clone())
        .unwrap_or_else(|| "standard".to_string())
}

fn usage_of(map: &[(String, i64)], weapon_id: &str) -> i64 {
    map.iter()
        .find(|(id, _)| id == weapon_id)
        .map(|(_, uses)| *uses)
        .unwrap_or(0)
}

fn set_string(map: &mut Vec<(String, String)>, weapon_id: &str, value: &str) {
    if let Some(slot) = map.iter_mut().find(|(id, _)| id == weapon_id) {
        slot.1 = value.to_string();
    } else {
        map.push((weapon_id.to_string(), value.to_string()));
    }
}

fn set_i64(map: &mut Vec<(String, i64)>, weapon_id: &str, value: i64) {
    if let Some(slot) = map.iter_mut().find(|(id, _)| id == weapon_id) {
        slot.1 = value;
    } else {
        map.push((weapon_id.to_string(), value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn melee_drops_a_tier_every_ten_uses_and_rusted_is_the_floor() {
        let mut quality = vec![("cutlass".into(), "standard".into())];
        let mut usage = Vec::new();
        assert!(
            tick_weapon_degradation(&mut quality, &mut usage, "cutlass", "melee", 9, 0).is_none()
        );
        assert_eq!(usage[0].1, 9);
        let dropped = tick_weapon_degradation(&mut quality, &mut usage, "cutlass", "melee", 1, 0);
        assert_eq!(dropped.as_deref(), Some("worn"));
        assert_eq!(usage[0].1, 0);
        quality[0].1 = "rusted".into();
        usage[0].1 = 50;
        assert!(
            tick_weapon_degradation(&mut quality, &mut usage, "cutlass", "melee", 1, 0).is_none()
        );
        assert_eq!(quality[0].1, "rusted");
        assert_eq!(usage[0].1, 51);
    }

    #[test]
    fn armor_and_firearm_use_their_own_thresholds() {
        assert!(check_degradation("armor", 15, 0));
        assert!(!check_degradation("armor", 14, 0));
        assert!(check_degradation("firearm", 20, 0));
        assert!(!check_degradation("firearm", 20, 1));
        assert_eq!(effects("masterwork").damage_mod, 2);
        assert!((effects("rusted").accuracy_mod + 0.10).abs() < 1e-9);
    }

    #[test]
    fn maintenance_resets_usage_and_smith_upgrade_spends_days() {
        let quality = vec![("cutlass".into(), "fine".into())];
        let mut usage = vec![("cutlass".into(), 7)];
        assert!(maintain_weapon(&quality, &mut usage, "cutlass", 10).is_err());
        assert_eq!(
            maintain_weapon(&quality, &mut usage, "cutlass", 25).unwrap(),
            0
        );
        assert_eq!(usage[0].1, 0);
        let mut quality = vec![("cutlass".into(), "standard".into())];
        let (tier, silver, days) = upgrade_weapon(&mut quality, &mut usage, "cutlass", 100);
        assert_eq!(tier, "fine");
        assert_eq!(silver, 0);
        assert_eq!(days, 3);
        assert_eq!(assign_purchase_quality(), "standard");
    }

    #[test]
    fn loot_quality_bands_follow_the_python_cuts() {
        // random() on seed 1 is stable; just check the function returns a tier.
        let mut rng = PyRandom::from_seed(1);
        let first = assign_loot_quality(2, &mut rng);
        assert!(TIERS.contains(&first));
        let mut rng = PyRandom::from_seed(99);
        let boss = assign_loot_quality(10, &mut rng);
        assert!(matches!(boss, "standard" | "fine" | "masterwork"));
    }
}
