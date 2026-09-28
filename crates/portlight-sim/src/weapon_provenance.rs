//! Weapon history, epithets, and recognition. Port of `engine/weapon_provenance.py`.

use crate::model::WeaponProvenance;
use crate::pyrand::PyRandom;

pub const RELIC_TIERS: [&str; 4] = ["unnamed", "bloodied", "reaper", "legendary"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recognition {
    pub recognized: bool,
    pub weapon_name: String,
    pub relic_tier: String,
    pub fear_bonus: i64,
    pub respect_bonus: i64,
    pub flavor: String,
}

pub fn relic_tier(kills: i64) -> &'static str {
    if kills >= 25 {
        "legendary"
    } else if kills >= 10 {
        "reaper"
    } else if kills >= 3 {
        "bloodied"
    } else {
        "unnamed"
    }
}

pub fn create_provenance(
    weapon_id: &str,
    port_id: &str,
    region: &str,
    day: i64,
) -> WeaponProvenance {
    WeaponProvenance {
        weapon_id: weapon_id.to_string(),
        acquired_port: port_id.to_string(),
        acquired_day: day,
        acquired_region: region.to_string(),
        kills: 0,
        named_kills: Vec::new(),
        epithet: None,
        custom_name: None,
        times_recognized: 0,
    }
}

/// Record a kill. The first named captain brands the weapon
/// `"{captain_name}'s Bane"`. Three named kills rename it.
pub fn record_kill(
    provenance: &mut WeaponProvenance,
    captain_id: Option<&str>,
    captain_name: Option<&str>,
) -> (Option<String>, Option<String>) {
    provenance.kills += 1;
    let old_tier = relic_tier(provenance.kills - 1);
    let new_tier = relic_tier(provenance.kills);
    let tier_change = (new_tier != old_tier).then(|| new_tier.to_string());
    let mut new_epithet = None;
    if let Some(captain_id) = captain_id {
        if !provenance.named_kills.iter().any(|id| id == captain_id) {
            provenance.named_kills.push(captain_id.to_string());
            if provenance.epithet.is_none() {
                if let Some(name) = captain_name {
                    provenance.epithet = Some(format!("{name}'s Bane"));
                    new_epithet = provenance.epithet.clone();
                }
            } else if provenance.named_kills.len() >= 3
                && provenance
                    .epithet
                    .as_deref()
                    .is_some_and(|epithet| epithet.contains("Bane"))
            {
                provenance.epithet = Some(legendary_name(provenance));
                new_epithet = provenance.epithet.clone();
            }
        }
    }
    (tier_change, new_epithet)
}

fn legendary_name(provenance: &WeaponProvenance) -> String {
    let kill_count = provenance.named_kills.len();
    if kill_count >= 5 {
        "Drinker of Captains".to_string()
    } else if kill_count >= 3 {
        "The Captain Killer".to_string()
    } else {
        provenance.epithet.clone().unwrap_or_default()
    }
}

pub fn check_recognition(
    provenance: Option<&mut WeaponProvenance>,
    weapon_name: &str,
    enemy_captain_id: &str,
    enemy_familiarity: i64,
    rng: &mut PyRandom,
) -> Recognition {
    let Some(provenance) = provenance else {
        return Recognition::none();
    };
    let tier = relic_tier(provenance.kills);
    if tier == "unnamed" {
        return Recognition::none();
    }
    let base = match tier {
        "bloodied" => 0.30,
        "reaper" => 0.60,
        "legendary" => 0.90,
        _ => 0.0,
    };
    let mut chance = base + enemy_familiarity as f64 * 0.01;
    if provenance
        .named_kills
        .iter()
        .any(|id| id == enemy_captain_id)
    {
        chance = 1.0;
    }
    chance = chance.min(0.95);
    if rng.random() > chance {
        return Recognition::none();
    }
    provenance.times_recognized += 1;
    let display = provenance
        .custom_name
        .clone()
        .or_else(|| provenance.epithet.clone())
        .unwrap_or_else(|| weapon_name.to_string());
    let fear_bonus = match tier {
        "bloodied" => 3,
        "reaper" => 7,
        "legendary" => 15,
        _ => 0,
    };
    let respect_bonus = match tier {
        "bloodied" => 2,
        "reaper" => 5,
        "legendary" => 10,
        _ => 0,
    };
    let flavor = if provenance
        .named_kills
        .iter()
        .any(|id| id == enemy_captain_id)
    {
        "Their eyes fix on your weapon. They KNOW that blade — it took one of their own."
            .to_string()
    } else if tier == "legendary" {
        format!("The pirate's grip tightens on their own sword. They've heard of {display}.")
    } else if tier == "reaper" {
        "A flicker of recognition — they've heard stories about that weapon.".to_string()
    } else {
        "They notice the notches on your blade. This weapon has seen blood.".to_string()
    };
    Recognition {
        recognized: true,
        weapon_name: display,
        relic_tier: tier.to_string(),
        fear_bonus,
        respect_bonus,
        flavor,
    }
}

impl Recognition {
    fn none() -> Self {
        Self {
            recognized: false,
            weapon_name: String::new(),
            relic_tier: "unnamed".to_string(),
            fear_bonus: 0,
            respect_bonus: 0,
            flavor: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_named_kill_earns_bane_and_three_rename_it() {
        let mut prov = create_provenance("cutlass", "porto_novo", "Mediterranean", 4);
        let (tier, epithet) = record_kill(&mut prov, Some("gnaw"), Some("Gnaw"));
        assert!(tier.is_none());
        assert_eq!(epithet.as_deref(), Some("Gnaw's Bane"));
        record_kill(&mut prov, Some("gnaw"), Some("Gnaw"));
        assert_eq!(prov.named_kills.len(), 1);
        assert_eq!(prov.kills, 2);
        record_kill(&mut prov, Some("old_coral"), Some("Old Coral"));
        let (_, epithet) = record_kill(&mut prov, Some("the_diver"), Some("The Diver"));
        assert_eq!(epithet.as_deref(), Some("The Captain Killer"));
        assert_eq!(relic_tier(prov.kills), "bloodied");
        record_kill(&mut prov, None, None);
        record_kill(&mut prov, Some("raj_the_quiet"), Some("Raj"));
        record_kill(&mut prov, Some("typhoon_mei"), Some("Mei"));
        assert_eq!(prov.epithet.as_deref(), Some("The Captain Killer"));
        assert_eq!(prov.named_kills.len(), 5);
    }

    #[test]
    fn unnamed_weapons_are_never_recognized() {
        let mut prov = create_provenance("cutlass", "", "", 0);
        let mut rng = PyRandom::from_seed(1);
        let result = check_recognition(Some(&mut prov), "Cutlass", "gnaw", 50, &mut rng);
        assert!(!result.recognized);
        prov.kills = 3;
        let mut rng = PyRandom::from_seed(1);
        let _ = check_recognition(Some(&mut prov), "Cutlass", "someone", 0, &mut rng);
        assert!(prov.times_recognized >= 0);
    }
}
