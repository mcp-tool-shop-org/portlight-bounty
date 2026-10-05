//! Companion recruitment, morale, and bonuses from `engine/companion_engine.py`.

use crate::content;
use crate::model::{Captain, Companion};
use crate::pyrand::PyRandom;
use crate::util::py_trunc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepartureEvent {
    pub companion_id: String,
    pub companion_name: String,
    pub reason: String,
    pub departure_line: String,
}

pub struct PartyCombatBonus {
    pub damage_bonus: i64,
    pub interception_chance: f64,
    pub speed_bonus: f64,
    pub danger_reduction: f64,
    pub heal_rate_bonus: f64,
    pub inspection_evasion: f64,
    pub trade_bonus: f64,
}

impl Default for PartyCombatBonus {
    fn default() -> Self {
        Self {
            damage_bonus: 0,
            interception_chance: 0.0,
            speed_bonus: 0.0,
            danger_reduction: 0.0,
            heal_rate_bonus: 0.0,
            inspection_evasion: 0.0,
            trade_bonus: 0.0,
        }
    }
}

pub struct PartyMemberSummary {
    pub id: String,
    pub name: String,
    pub role: String,
    pub morale: i64,
    pub morale_status: String,
    pub personality: String,
    pub joined_day: i64,
}

/// `None` means the companion can be recruited.
pub fn can_recruit(captain: &Captain, companion_id: &str, port_id: &str) -> Option<String> {
    let Some(comp) = content::content().companion(companion_id) else {
        return Some(format!("Unknown companion: {companion_id}"));
    };
    if comp.home_port_id != port_id {
        return Some(format!("{} is not at this port.", comp.name));
    }
    if captain
        .party
        .companions
        .iter()
        .any(|member| member.companion_id == companion_id)
    {
        return Some(format!("{} is already in your party.", comp.name));
    }
    if captain.party.departed.iter().any(|id| id == companion_id) {
        return Some(format!(
            "{} already left your crew. They won't come back.",
            comp.name
        ));
    }
    if captain.party.companions.len() as i64 >= captain.party.max_size {
        return Some(format!(
            "Party is full (max {} companions).",
            captain.party.max_size
        ));
    }
    if captain.silver < comp.hire_cost {
        return Some(format!(
            "{} costs {} silver to recruit. You have {}.",
            comp.name, comp.hire_cost, captain.silver
        ));
    }
    let standing = captain.standing.regional_of(&comp.region);
    if standing < comp.required_standing {
        return Some(format!(
            "Need {} standing in {} to recruit {}. You have {standing}.",
            comp.required_standing, comp.region, comp.name
        ));
    }
    None
}

/// Add the companion. The caller subtracts `hire_cost`, matching the CLI.
pub fn recruit(captain: &mut Captain, companion_id: &str, current_day: i64) -> Companion {
    let comp = content::content()
        .companion(companion_id)
        .expect("recruit requires a known companion");
    let state = Companion {
        companion_id: companion_id.to_string(),
        role_id: comp.role_id.clone(),
        morale: 70,
        joined_day: current_day,
        personality: comp.personality.clone(),
    };
    captain.party.companions.push(state.clone());
    state
}

pub fn dismiss(captain: &mut Captain, companion_id: &str) -> Option<String> {
    let Some(index) = captain
        .party
        .companions
        .iter()
        .position(|member| member.companion_id == companion_id)
    else {
        return Some(format!("No companion with id {companion_id} in party."));
    };
    captain.party.companions.remove(index);
    None
}

/// `(companion_id, morale_delta, flavor)`.
pub fn apply_morale_trigger(captain: &mut Captain, trigger: &str) -> Vec<(String, i64, String)> {
    let Some(row) = content::content().morale_reaction(trigger).cloned() else {
        return Vec::new();
    };
    let mut results = Vec::new();
    for member in &mut captain.party.companions {
        let base_delta = row.deltas.get(&member.role_id).copied().unwrap_or(0);
        if base_delta == 0 {
            continue;
        }
        let pers_delta = content::content()
            .personality_modifier(&member.personality)
            .and_then(|mods| mods.deltas.get(trigger).copied())
            .unwrap_or(0);
        let total_delta = base_delta + pers_delta;
        if total_delta == 0 {
            continue;
        }
        member.morale = (member.morale + total_delta).clamp(0, 100);
        let name = content::content()
            .companion(&member.companion_id)
            .map(|comp| comp.name.clone())
            .unwrap_or_else(|| member.companion_id.clone());
        let flavor = if total_delta > 0 {
            format!("{name} approves. (+{total_delta} morale)")
        } else {
            format!("{name} disapproves. ({total_delta} morale)")
        };
        results.push((member.companion_id.clone(), total_delta, flavor));
    }
    results
}

pub fn check_departures(captain: &mut Captain) -> Vec<DepartureEvent> {
    let mut departures = Vec::new();
    let mut remaining = Vec::new();
    let members = std::mem::take(&mut captain.party.companions);
    for member in members {
        if member.morale <= 10 {
            let comp = content::content().companion(&member.companion_id);
            let name = comp
                .map(|comp| comp.name.clone())
                .unwrap_or_else(|| member.companion_id.clone());
            let line = comp
                .map(|comp| comp.departure_line.clone())
                .unwrap_or_else(|| "They leave without a word.".to_string());
            captain.party.departed.push(member.companion_id.clone());
            departures.push(DepartureEvent {
                companion_id: member.companion_id,
                companion_name: name,
                reason: "Morale collapsed".to_string(),
                departure_line: line,
            });
        } else {
            remaining.push(member);
        }
    }
    captain.party.companions = remaining;
    departures
}

fn morale_mult(morale: i64) -> f64 {
    if morale >= 30 {
        1.0
    } else {
        morale as f64 / 30.0
    }
}

pub fn party_combat_bonus(captain: &Captain) -> PartyCombatBonus {
    let mut bonus = PartyCombatBonus::default();
    for member in &captain.party.companions {
        let Some(role) = content::content().companion_role(&member.role_id) else {
            continue;
        };
        let mult = morale_mult(member.morale);
        bonus.damage_bonus += py_trunc(role.combat_damage_bonus as f64 * mult);
        bonus.interception_chance += role.combat_interception_chance * mult;
        bonus.speed_bonus += role.speed_bonus * mult;
        bonus.danger_reduction += role.danger_reduction * mult;
        bonus.heal_rate_bonus += role.heal_rate_bonus * mult;
        bonus.inspection_evasion += role.inspection_evasion * mult;
        bonus.trade_bonus += role.trade_bonus * mult;
    }
    bonus
}

pub fn roll_interception(captain: &mut Captain, rng: &mut PyRandom) -> (bool, String) {
    for member in &mut captain.party.companions {
        let Some(role) = content::content().companion_role(&member.role_id) else {
            continue;
        };
        if role.combat_interception_chance <= 0.0 {
            continue;
        }
        let chance = role.combat_interception_chance * morale_mult(member.morale);
        if rng.random() < chance {
            let name = content::content()
                .companion(&member.companion_id)
                .map(|comp| comp.name.clone())
                .unwrap_or_else(|| "Your companion".to_string());
            member.morale = (member.morale - 2).max(0);
            return (
                true,
                format!("{name} throws themselves in the way! They take the hit for you."),
            );
        }
    }
    (false, String::new())
}

pub fn party_summary(captain: &Captain) -> Vec<PartyMemberSummary> {
    captain
        .party
        .companions
        .iter()
        .map(|member| {
            let comp = content::content().companion(&member.companion_id);
            let role = content::content().companion_role(&member.role_id);
            let morale_status = if member.morale >= 60 {
                "happy"
            } else if member.morale >= 30 {
                "concerned"
            } else if member.morale > 10 {
                "unhappy"
            } else {
                "leaving"
            };
            PartyMemberSummary {
                id: member.companion_id.clone(),
                name: comp
                    .map(|comp| comp.name.clone())
                    .unwrap_or_else(|| member.companion_id.clone()),
                role: role
                    .map(|role| role.name.clone())
                    .unwrap_or_else(|| member.role_id.clone()),
                morale: member.morale,
                morale_status: morale_status.to_string(),
                personality: member.personality.clone(),
                joined_day: member.joined_day,
            }
        })
        .collect()
}

pub fn cohesion(captain: &Captain) -> i64 {
    if captain.party.companions.is_empty() {
        return 0;
    }
    let sum: i64 = captain
        .party
        .companions
        .iter()
        .map(|member| member.morale)
        .sum();
    sum / captain.party.companions.len() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::new_game;

    #[test]
    fn red_tomas_joins_at_corsairs_rest_and_leaves_at_morale_ten() {
        let mut captain = new_game("Ada", "merchant", 1, Some("corsairs_rest"))
            .unwrap()
            .captain;
        assert!(can_recruit(&captain, "red_tomas", "porto_novo")
            .unwrap()
            .contains("is not at this port"));
        assert!(can_recruit(&captain, "iron_marta", "corsairs_rest").is_some());
        captain.silver -= 60;
        let joined = recruit(&mut captain, "red_tomas", 1);
        assert_eq!(joined.morale, 70);
        assert_eq!(joined.role_id, "marine");
        assert!(can_recruit(&captain, "red_tomas", "corsairs_rest")
            .unwrap()
            .contains("already in your party"));

        let reactions = apply_morale_trigger(&mut captain, "fled_combat");
        assert_eq!(reactions[0].1, -5, "pragmatic marine has no extra modifier");
        captain.party.companions[0].personality = "reckless".to_string();
        captain.party.companions[0].morale = 70;
        let reactions = apply_morale_trigger(&mut captain, "fled_combat");
        assert_eq!(reactions[0].1, -8, "marine -5 plus reckless -3");
        captain.party.companions[0].morale = 10;
        let left = check_departures(&mut captain);
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].reason, "Morale collapsed");
        assert!(captain.party.companions.is_empty());
        assert!(can_recruit(&captain, "red_tomas", "corsairs_rest")
            .unwrap()
            .contains("won't come back"));
    }

    #[test]
    fn a_happy_marine_adds_one_damage_and_can_intercept() {
        let mut captain = new_game("Ada", "merchant", 1, Some("corsairs_rest"))
            .unwrap()
            .captain;
        captain.silver -= 60;
        recruit(&mut captain, "red_tomas", 1);
        let bonus = party_combat_bonus(&captain);
        assert_eq!(bonus.damage_bonus, 1);
        assert!((bonus.interception_chance - 0.25).abs() < 1e-9);
        let mut rng = PyRandom::from_seed(1);
        let (hit, flavor) = roll_interception(&mut captain, &mut rng);
        let _ = (hit, flavor);
        assert!(cohesion(&captain) > 0);
        assert_eq!(party_summary(&captain)[0].morale_status, "happy");
    }
}
