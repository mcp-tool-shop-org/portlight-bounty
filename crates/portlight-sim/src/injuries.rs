//! Injury rolls, healing, and permanent effects. Port of `engine/injuries.py`.
//!
//! A surgeon on the roster does not change the heal rate. Python defines
//! `surgeon_death_reduction` and never calls it from the day tick. The
//! recovery bonus is the surgeon's bay upgrade, which lets healing run at sea.

use crate::content::{self, InjuryDef};
use crate::model::Injury;
use crate::pyrand::PyRandom;

pub const INJURY_DAMAGE_THRESHOLD: i64 = 4;
pub const INJURY_BASE_CHANCE: f64 = 0.15;

/// Aggregated combat modifiers from every active wound.
#[derive(Debug, Clone)]
pub struct Effects {
    pub melee_damage_mod: i64,
    pub stamina_max_mod: i64,
    pub hp_max_mod: i64,
    pub ranged_accuracy_mod: f64,
    pub can_dodge: bool,
    pub can_use_firearms: bool,
    pub thrust_multiplier: f64,
}

pub fn effects(injury_ids: &[String]) -> Effects {
    let mut effects = Effects {
        melee_damage_mod: 0,
        stamina_max_mod: 0,
        hp_max_mod: 0,
        ranged_accuracy_mod: 0.0,
        can_dodge: true,
        can_use_firearms: true,
        thrust_multiplier: 1.0,
    };
    let catalog = content::content();
    for id in injury_ids {
        let Some(def) = catalog.injury(id) else {
            continue;
        };
        effects.melee_damage_mod += def.melee_damage_mod;
        effects.stamina_max_mod += def.stamina_max_mod;
        effects.hp_max_mod += def.hp_max_mod;
        effects.ranged_accuracy_mod += def.ranged_accuracy_mod;
        if !def.can_dodge {
            effects.can_dodge = false;
        }
        if !def.can_use_firearms {
            effects.can_use_firearms = false;
        }
        effects.thrust_multiplier *= def.thrust_multiplier;
    }
    effects
}

fn pool(attack_type: &str) -> Vec<&InjuryDef> {
    content::content()
        .injuries
        .iter()
        .filter(|def| def.attack_types.iter().any(|kind| kind == attack_type))
        .collect()
}

/// Roll one wound after a hit. `None` when the damage is under the threshold,
/// the chance misses, or the attack type has an empty pool.
pub fn roll_injury(
    damage: i64,
    attack_type: &str,
    rng: &mut PyRandom,
    injury_bonus: f64,
) -> Option<String> {
    if damage < INJURY_DAMAGE_THRESHOLD {
        return None;
    }
    let chance = ((damage - INJURY_DAMAGE_THRESHOLD + 1) as f64 * INJURY_BASE_CHANCE
        + injury_bonus)
        .min(0.90);
    if rng.random() >= chance {
        return None;
    }
    let pool = pool(attack_type);
    if pool.is_empty() {
        return None;
    }
    Some(pool[rng.choice_index(pool.len())].id.clone())
}

pub fn create_injury(injury_id: &str, current_day: i64) -> Injury {
    let heal_remaining = content::content()
        .injury(injury_id)
        .and_then(|def| def.heal_days);
    Injury {
        injury_id: injury_id.to_string(),
        acquired_day: current_day,
        heal_remaining,
        treated: false,
    }
}

/// Advance healing. Permanent wounds (`heal_remaining == None`) stay.
/// Medicines double the rate and mark the wound treated. Healing is skipped
/// when `in_port` is false; the caller passes true for a surgeon's bay at sea.
pub fn heal_injury_tick(
    injuries: &[Injury],
    days: i64,
    in_port: bool,
    has_medicines: bool,
) -> Vec<Injury> {
    if !in_port {
        return injuries.to_vec();
    }
    let mut result = Vec::new();
    for injury in injuries {
        let Some(remaining) = injury.heal_remaining else {
            result.push(injury.clone());
            continue;
        };
        let heal_rate = if has_medicines { days * 2 } else { days };
        let remaining = 0.max(remaining - heal_rate);
        if remaining <= 0 {
            continue;
        }
        result.push(Injury {
            injury_id: injury.injury_id.clone(),
            acquired_day: injury.acquired_day,
            heal_remaining: Some(remaining),
            treated: injury.treated || has_medicines,
        });
    }
    result
}

/// Halve the remaining days with integer division, minimum 1. Permanent and
/// already-treated wounds return the Python sentence.
pub fn treat_injury(injury: &Injury, silver: i64) -> Result<(Injury, i64), String> {
    let Some(def) = content::content().injury(&injury.injury_id) else {
        return Err(format!("Unknown injury: {}", injury.injury_id));
    };
    if injury.heal_remaining.is_none() {
        return Err(format!("{} is permanent and cannot be treated.", def.name));
    }
    if injury.treated {
        return Err(format!("{} has already been treated.", def.name));
    }
    let cost = def.heal_silver;
    if silver < cost {
        return Err(format!("Treatment costs {cost} silver. You have {silver}."));
    }
    let new_remaining = 1.max(injury.heal_remaining.unwrap_or(0) / 2);
    Ok((
        Injury {
            injury_id: injury.injury_id.clone(),
            acquired_day: injury.acquired_day,
            heal_remaining: Some(new_remaining),
            treated: true,
        },
        silver - cost,
    ))
}

pub fn blocks_style(injury_ids: &[String], required_body_parts: &[String]) -> bool {
    let catalog = content::content();
    for id in injury_ids {
        let Some(def) = catalog.injury(id) else {
            continue;
        };
        if def
            .blocked_body_parts
            .iter()
            .any(|part| required_body_parts.iter().any(|need| need == part))
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pyrand::PyRandom;

    #[test]
    fn damage_under_threshold_does_not_draw() {
        let mut rng = PyRandom::from_seed(1);
        assert!(roll_injury(3, "slash", &mut rng, 0.0).is_none());
        assert_eq!(rng.random(), PyRandom::from_seed(1).random());
    }

    #[test]
    fn slash_pool_matches_catalog_order() {
        let ids: Vec<_> = pool("slash")
            .into_iter()
            .map(|def| def.id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec![
                "cut_hand",
                "bruised_ribs",
                "deep_slash",
                "shattered_knee",
                "blinded_eye",
                "broken_sword_arm",
                "severed_fingers",
            ]
        );
    }

    #[test]
    fn permanent_wounds_never_heal_and_medicines_double_the_rate() {
        let permanent = create_injury("blinded_eye", 3);
        assert!(permanent.heal_remaining.is_none());
        let mut timed = create_injury("cut_hand", 3);
        assert_eq!(timed.heal_remaining, Some(10));
        let kept = heal_injury_tick(&[permanent.clone(), timed.clone()], 1, false, true);
        assert_eq!(kept.len(), 2);
        let healed = heal_injury_tick(&[permanent, timed.clone()], 4, true, true);
        assert_eq!(healed.len(), 2);
        assert_eq!(healed[0].injury_id, "blinded_eye");
        assert_eq!(healed[1].heal_remaining, Some(2));
        timed.heal_remaining = Some(6);
        let stepped = heal_injury_tick(&[timed], 2, true, true);
        assert_eq!(stepped[0].heal_remaining, Some(2));
        assert!(stepped[0].treated);
    }

    #[test]
    fn treatment_halves_with_integer_division() {
        let injury = create_injury("broken_sword_arm", 1);
        let err = treat_injury(&injury, 0).unwrap_err();
        assert!(err.contains("Treatment costs"));
        let (treated, silver) = treat_injury(&injury, 10_000).unwrap();
        assert_eq!(treated.heal_remaining, Some(30));
        assert!(treated.treated);
        assert!(silver < 10_000);
        assert!(treat_injury(&treated, silver).is_err());
        let permanent = create_injury("severed_fingers", 1);
        assert!(treat_injury(&permanent, 10_000)
            .unwrap_err()
            .contains("permanent"));
    }

    #[test]
    fn effects_stack_and_block_styles() {
        let fx = effects(&["shattered_knee".into(), "broken_sword_arm".into()]);
        assert!(!fx.can_dodge);
        assert_eq!(fx.melee_damage_mod, -2);
        assert!((fx.thrust_multiplier - 0.5).abs() < 1e-9);
        assert!(blocks_style(
            &["cut_hand".into()],
            &["hand".into(), "arm".into()]
        ));
        assert!(!blocks_style(&["cut_hand".into()], &["leg".into()]));
    }
}
