//! Captain skills from `engine/skill_engine.py`.
//!
//! Learning spends the next level's silver cost. Python has no separate
//! skill-point pool; [`crate::session::Session::spend_skill_point`] is that spend.

use std::collections::BTreeMap;

use crate::content::{self, BlacksmithEffectDef};
use crate::model::Skill;
use crate::util::py_trunc;

pub fn skill_level(skills: &[Skill], skill_id: &str) -> i64 {
    skills
        .iter()
        .find(|skill| skill.id == skill_id)
        .map(|skill| skill.level)
        .unwrap_or(0)
}

pub fn skill_display(skills: &[Skill], skill_id: &str) -> String {
    let Some(skill) = content::content().skill(skill_id) else {
        return "Unknown".to_string();
    };
    let level = skill_level(skills, skill_id);
    if level == 0 {
        return "Untrained".to_string();
    }
    skill.levels[(level - 1) as usize].name.clone()
}

/// `None` means the captain can learn the next level.
pub fn can_learn_skill(
    skills: &[Skill],
    silver: i64,
    port_id: &str,
    skill_id: &str,
) -> Option<String> {
    let catalog = content::content();
    let Some(skill) = catalog.skill(skill_id) else {
        return Some(format!("Unknown skill: {skill_id}"));
    };
    let current = skill_level(skills, skill_id);
    if current >= skill.max_level {
        let name = skill
            .levels
            .get((current - 1) as usize)
            .map(|level| level.name.as_str())
            .unwrap_or("");
        return Some(format!("Already at maximum {} level ({name}).", skill.name));
    }
    let trainers = catalog.trainers_at(port_id, Some(skill_id));
    if trainers.is_empty() {
        return Some(format!("No {} trainer at this port.", skill.name));
    }
    let max_teach = trainers
        .iter()
        .map(|trainer| trainer.max_teach_level)
        .max()
        .unwrap_or(0);
    if current + 1 > max_teach {
        // Python `max` keeps the first tie. `max_by_key` keeps the last, so
        // walk the reversed list.
        let best = trainers
            .iter()
            .rev()
            .max_by_key(|trainer| trainer.max_teach_level)
            .expect("trainer");
        return Some(format!(
            "{} can only teach up to level {max_teach}. Find a more skilled trainer.",
            best.name
        ));
    }
    let next = &skill.levels[current as usize];
    if silver < next.silver_cost {
        return Some(format!(
            "{} training costs {} silver. You have {silver}.",
            next.name, next.silver_cost
        ));
    }
    None
}

/// Apply one level. Returns `(remaining_silver, training_days)`.
pub fn learn_skill(skills: &mut Vec<Skill>, silver: i64, skill_id: &str) -> (i64, i64) {
    let current = skill_level(skills, skill_id);
    let skill = content::content()
        .skill(skill_id)
        .expect("learn_skill requires a known skill");
    let next = &skill.levels[current as usize];
    let remaining = silver - next.silver_cost;
    let days = next.training_days;
    if let Some(slot) = skills.iter_mut().find(|skill| skill.id == skill_id) {
        slot.level = current + 1;
    } else {
        skills.push(Skill {
            id: skill_id.to_string(),
            level: current + 1,
        });
    }
    (remaining, days)
}

fn effects(level: i64) -> &'static BlacksmithEffectDef {
    content::content().blacksmith_effect(level)
}

pub fn apply_maintenance_discount(base_cost: i64, blacksmith_level: i64) -> i64 {
    let discount = effects(blacksmith_level).maintenance_discount;
    1.max(py_trunc(base_cost as f64 * (1.0 - discount)))
}

pub fn apply_upgrade_discount(base_cost: i64, blacksmith_level: i64) -> i64 {
    let discount = effects(blacksmith_level).upgrade_discount;
    1.max(py_trunc(base_cost as f64 * (1.0 - discount)))
}

pub fn degrade_threshold_bonus(blacksmith_level: i64) -> i64 {
    py_trunc(10.0 * effects(blacksmith_level).degrade_slow)
}

pub fn can_field_repair(blacksmith_level: i64) -> bool {
    effects(blacksmith_level).field_repair
}

pub fn field_repair_max_quality(blacksmith_level: i64) -> String {
    effects(blacksmith_level).field_max_quality.clone()
}

/// `Ok(())` is Python's `(True, None)`. The error string is the failure case.
pub fn field_repair_weapon(
    weapon_id: &str,
    weapon_quality: &mut BTreeMap<String, String>,
    weapon_usage: &mut BTreeMap<String, i64>,
    blacksmith_level: i64,
) -> Result<(), String> {
    if !can_field_repair(blacksmith_level) {
        return Err("Need Journeyman blacksmith skill to repair at sea.".to_string());
    }
    let max_quality = field_repair_max_quality(blacksmith_level);
    let current = weapon_quality
        .get(weapon_id)
        .cloned()
        .unwrap_or_else(|| "standard".to_string());
    weapon_usage.insert(weapon_id.to_string(), 0);
    if quality_index(&current) < quality_index(&max_quality) {
        weapon_quality.insert(weapon_id.to_string(), max_quality);
    }
    Ok(())
}

fn quality_index(quality: &str) -> i64 {
    match quality {
        "rusted" => 0,
        "worn" => 1,
        "standard" => 2,
        "fine" => 3,
        "masterwork" => 4,
        _ => 2,
    }
}

/// Resolve a skill argument the way `learn-skill` does: id, else display name.
pub fn resolve_skill_id(skill_id: &str) -> String {
    let catalog = content::content();
    if catalog.skill(skill_id).is_some() {
        return skill_id.to_string();
    }
    catalog
        .skill_by_name(skill_id)
        .map(|skill| skill.id.clone())
        .unwrap_or_else(|| skill_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn porto_novo_teaches_apprentice_blacksmith_for_fifty_silver() {
        let mut skills = Vec::new();
        assert!(can_learn_skill(&skills, 49, "porto_novo", "blacksmith")
            .unwrap()
            .contains("50 silver"));
        assert!(can_learn_skill(&skills, 550, "corsairs_rest", "blacksmith")
            .unwrap()
            .contains("No Blacksmithing trainer"));
        let (silver, days) = learn_skill(&mut skills, 550, "blacksmith");
        assert_eq!(silver, 500);
        assert_eq!(days, 3);
        assert_eq!(skill_level(&skills, "blacksmith"), 1);
        assert_eq!(skill_display(&skills, "blacksmith"), "Apprentice");
        assert_eq!(
            degrade_threshold_bonus(1),
            2,
            "int(10 * 0.20) matches CPython"
        );
        assert_eq!(apply_maintenance_discount(7, 1), 5);
        assert_eq!(apply_upgrade_discount(100, 3), 75);
    }

    #[test]
    fn vasquez_cannot_teach_master_and_field_repair_needs_journeyman() {
        let mut skills = vec![Skill {
            id: "blacksmith".to_string(),
            level: 2,
        }];
        let error = can_learn_skill(&skills, 5000, "porto_novo", "blacksmith").unwrap();
        assert_eq!(
            error,
            "Old Vasquez can only teach up to level 2. Find a more skilled trainer."
        );
        assert!(can_learn_skill(&skills, 5000, "ironhaven", "blacksmith").is_none());
        let (silver, days) = learn_skill(&mut skills, 5000, "blacksmith");
        assert_eq!(silver, 4600);
        assert_eq!(days, 8);
        assert!(can_learn_skill(&skills, 5000, "ironhaven", "blacksmith")
            .unwrap()
            .starts_with("Already at maximum Blacksmithing"));

        let mut quality = BTreeMap::new();
        let mut usage = BTreeMap::from([("cutlass".to_string(), 4)]);
        assert!(field_repair_weapon("cutlass", &mut quality, &mut usage, 1).is_err());
        quality.insert("cutlass".to_string(), "worn".to_string());
        field_repair_weapon("cutlass", &mut quality, &mut usage, 2).unwrap();
        assert_eq!(usage["cutlass"], 0);
        assert_eq!(quality["cutlass"], "standard");
    }
}
