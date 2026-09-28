//! Pirate-captain memory from `engine/captain_memory.py`.
//!
//! `tick_captain_agency` is not called from `GameSession.advance`. The CLI
//! calls it from `tick_sea_captain_agency`, so [`crate::session::Session::advance`]
//! does not draw it.

use md5::{Digest, Md5};

use crate::content;
use crate::model::{CaptainMemory, CaptainRelationship, EncounterMemory};
use crate::pyrand::PyRandom;

pub struct CaptainGoal {
    pub verb: String,
    pub priority: f64,
    pub reason: String,
}

pub struct CaptainAction {
    pub captain_id: String,
    pub captain_name: String,
    pub verb: String,
    pub message: String,
    pub effect_type: String,
    pub effect_value: i64,
}

pub struct RelationshipSummary {
    pub breakpoint: String,
    pub respect: i64,
    pub fear: i64,
    pub grudge: i64,
    pub familiarity: i64,
    pub encounters: usize,
    pub times_spared: i64,
    pub last_seen_day: i64,
}

const BREAKPOINTS: [&str; 5] = ["ally", "rival", "prey", "nemesis", "neutral"];

pub fn breakpoints() -> &'static [&'static str] {
    &BREAKPOINTS
}

pub fn derive_breakpoint(rel: &CaptainRelationship) -> &'static str {
    if rel.respect >= 50 && rel.grudge < 20 {
        return "ally";
    }
    if rel.grudge >= 50 || (rel.grudge >= 30 && rel.fear < 20) {
        return "nemesis";
    }
    if rel.fear >= 50 && rel.grudge < 40 {
        return "prey";
    }
    if rel.respect >= 30 && rel.familiarity >= 40 {
        return "rival";
    }
    "neutral"
}

fn clamp(value: i64, lo: i64, hi: i64) -> i64 {
    value.max(lo).min(hi)
}

fn outcome_deltas(key: &str) -> (i64, i64, i64, i64) {
    match key {
        "player_won" => (15, 10, 5, 10),
        "player_won_spared" => (25, 5, -10, 15),
        "player_lost" => (5, -5, 0, 10),
        "ship_sunk" => (5, 20, 15, 10),
        "fled" => (-10, -10, 0, 5),
        "negotiated" => (5, 0, 0, 5),
        "traded" => (10, 0, -5, 10),
        _ => (0, 0, 0, 5),
    }
}

pub fn record_encounter(
    memory: &mut CaptainMemory,
    day: i64,
    region: &str,
    outcome: &str,
    player_spared: bool,
    player_used_firearm: bool,
    crew_killed: i64,
) -> EncounterMemory {
    let key = if player_spared && outcome == "player_won" {
        "player_won_spared"
    } else {
        outcome
    };
    let (mut respect, mut fear, mut grudge, familiarity) = outcome_deltas(key);
    if player_used_firearm {
        respect += -5;
        fear += 10;
        grudge += 5;
    }
    if crew_killed >= 5 {
        fear += 15;
        grudge += 20;
    }
    let enc = EncounterMemory {
        day,
        region: region.to_string(),
        outcome: outcome.to_string(),
        player_spared,
        player_used_firearm,
        crew_killed,
        respect_delta: respect,
        fear_delta: fear,
        grudge_delta: grudge,
        familiarity_delta: familiarity,
    };
    let rel = &mut memory.relationship;
    rel.respect = clamp(rel.respect + respect, -100, 100);
    rel.fear = clamp(rel.fear + fear, 0, 100);
    rel.grudge = clamp(rel.grudge + grudge, 0, 100);
    rel.familiarity = clamp(rel.familiarity + familiarity, 0, 100);
    memory.encounters.push(enc.clone());
    memory.last_seen_day = day;
    memory.last_seen_region = region.to_string();
    if player_spared {
        memory.times_spared += 1;
    }
    if matches!(outcome, "player_won" | "player_won_spared" | "ship_sunk") {
        memory.times_defeated_by_player += 1;
    }
    if outcome == "player_lost" {
        memory.times_defeated_player += 1;
    }
    if outcome == "ship_sunk" {
        memory.player_sank_their_ship = true;
    }
    enc
}

pub fn derive_goals(
    memory: &CaptainMemory,
    captain_region: &str,
    player_region: &str,
    player_silver: i64,
    current_day: i64,
) -> Vec<CaptainGoal> {
    let rel = &memory.relationship;
    let breakpoint = derive_breakpoint(rel);
    let mut goals = Vec::new();
    let same_region = captain_region == player_region;
    let days_since = if memory.last_seen_day > 0 {
        current_day - memory.last_seen_day
    } else {
        999
    };
    match breakpoint {
        "ally" => {
            if same_region {
                goals.push(goal(
                    "warn",
                    0.7,
                    "Ally in your waters — wants to keep you safe",
                ));
            }
            if rel.respect >= 40 {
                goals.push(goal("trade_offer", 0.5, "Respects you enough to deal"));
            }
            if rel.respect >= 60 && days_since > 20 {
                goals.push(goal(
                    "gift",
                    0.3,
                    "Hasn't seen you in a while — sending regards",
                ));
            }
        }
        "rival" => {
            if same_region && rel.familiarity >= 40 {
                goals.push(goal("challenge", 0.6, "Wants to test you again"));
            }
            if rel.respect >= 30 {
                goals.push(goal(
                    "trade_offer",
                    0.4,
                    "Rival's respect — a deal between equals",
                ));
            }
        }
        "nemesis" => {
            if same_region {
                goals.push(goal("ambush", 0.85, "Hunting you"));
            }
            if rel.grudge >= 60 {
                goals.push(goal("bounty", 0.5, "Put a price on your head"));
            }
        }
        "prey" => {
            if same_region {
                goals.push(goal("retreat", 0.7, "Fears you — avoiding your waters"));
            }
        }
        _ => {
            if same_region && player_silver > 300 {
                goals.push(goal("extort", 0.4, "Sees a fat purse"));
            }
        }
    }
    if rel.familiarity >= 30 {
        goals.push(goal("rumor", 0.2, "Your name comes up in conversation"));
    }
    goals.sort_by(|left, right| right.priority.total_cmp(&left.priority));
    goals.truncate(2);
    goals
}

fn goal(verb: &str, priority: f64, reason: &str) -> CaptainGoal {
    CaptainGoal {
        verb: verb.to_string(),
        priority,
        reason: reason.to_string(),
    }
}

pub fn resolve_action(
    goal: &CaptainGoal,
    memory: &CaptainMemory,
    captain_name: &str,
    faction_name: &str,
) -> Option<CaptainAction> {
    let cid = memory.captain_id.clone();
    let name = captain_name.to_string();
    let action = match goal.verb.as_str() {
        "warn" => CaptainAction {
            captain_id: cid,
            captain_name: name.clone(),
            verb: "warn".to_string(),
            message: format!(
                "Word reaches you from {captain_name}: \"Watch your back in these waters. {faction_name} isn't the only danger.\""
            ),
            effect_type: "message".to_string(),
            effect_value: 0,
        },
        "trade_offer" => CaptainAction {
            captain_id: cid,
            captain_name: name,
            verb: "trade_offer".to_string(),
            message: format!(
                "{captain_name} has left word at the dockmaster's office — a private trade arrangement, captain to captain."
            ),
            effect_type: "contract".to_string(),
            effect_value: 0,
        },
        "ambush" => CaptainAction {
            captain_id: cid,
            captain_name: name,
            verb: "ambush".to_string(),
            message: format!(
                "Sails on the horizon — {captain_name} has found you. There will be no negotiation this time."
            ),
            effect_type: "encounter".to_string(),
            effect_value: 0,
        },
        "challenge" => CaptainAction {
            captain_id: cid,
            captain_name: name,
            verb: "challenge".to_string(),
            message: format!(
                "A messenger arrives: {captain_name} challenges you to a duel. \"Meet me at sea. Let's settle this properly.\""
            ),
            effect_type: "encounter".to_string(),
            effect_value: 0,
        },
        "retreat" => CaptainAction {
            captain_id: cid,
            captain_name: name,
            verb: "retreat".to_string(),
            message: format!(
                "The dockhand mentions that {captain_name}'s ship was spotted heading out of the region in a hurry."
            ),
            effect_type: "message".to_string(),
            effect_value: 0,
        },
        "extort" => {
            let tribute = 30 + memory.relationship.familiarity;
            CaptainAction {
                captain_id: cid,
                captain_name: name,
                verb: "extort".to_string(),
                message: format!(
                    "{captain_name} sends a runner: \"Tribute. {tribute} silver. Or we do this the other way.\""
                ),
                effect_type: "message".to_string(),
                effect_value: tribute,
            }
        }
        "gift" => {
            let gift = 15 + memory.times_spared * 10;
            CaptainAction {
                captain_id: cid,
                captain_name: name,
                verb: "gift".to_string(),
                message: format!(
                    "A crate arrives at your berth, marked with {captain_name}'s seal. Inside: {gift} silver and a note — \"The sea remembers its friends.\""
                ),
                effect_type: "silver".to_string(),
                effect_value: gift,
            }
        }
        "rumor" => {
            let breakpoint = derive_breakpoint(&memory.relationship);
            let opinion = match breakpoint {
                "ally" => "speaks highly of",
                "rival" => "respects but watches closely",
                "nemesis" => "has a grudge against",
                "prey" => "avoids any mention of",
                "neutral" => "has crossed paths with",
                _ => "knows of",
            };
            CaptainAction {
                captain_id: cid,
                captain_name: name,
                verb: "rumor".to_string(),
                message: format!(
                    "Tavern talk: \"{captain_name} {opinion} a captain called {{player_name}}.\""
                ),
                effect_type: "message".to_string(),
                effect_value: 0,
            }
        }
        _ => return None,
    };
    Some(action)
}

fn md5_prefix(input: &str) -> u32 {
    let digest = Md5::digest(input.as_bytes());
    u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]])
}

/// At most one autonomous action. `rng` is unused, matching Python.
pub fn tick_captain_agency(
    memories: &[CaptainMemory],
    player_region: &str,
    player_silver: i64,
    current_day: i64,
    _rng: &mut PyRandom,
) -> Vec<CaptainAction> {
    let catalog = content::content();
    let captains: Vec<(String, String, String)> = catalog
        .pirate_captains
        .iter()
        .map(|captain| {
            (
                captain.id.clone(),
                captain.name.clone(),
                captain.faction_id.clone(),
            )
        })
        .collect();
    let mut actions = Vec::new();
    for (captain_id, captain_name, faction_id) in captains {
        let Some(memory) = memories
            .iter()
            .find(|memory| memory.captain_id == captain_id)
        else {
            continue;
        };
        if memory.encounters.is_empty() {
            continue;
        }
        let hash_val = md5_prefix(&format!("{captain_id}:{current_day}"));
        let breakpoint = derive_breakpoint(&memory.relationship);
        let modulus = if matches!(breakpoint, "nemesis" | "ally") {
            3
        } else {
            5
        };
        if hash_val % modulus != 0 {
            continue;
        }
        let captain_region = if !memory.last_seen_region.is_empty() {
            memory.last_seen_region.clone()
        } else {
            catalog
                .faction(&faction_id)
                .and_then(|faction| faction.territory_regions.first().cloned())
                .unwrap_or_else(|| "Mediterranean".to_string())
        };
        let goals = derive_goals(
            memory,
            &captain_region,
            player_region,
            player_silver,
            current_day,
        );
        if goals.is_empty() {
            continue;
        }
        let faction_name = catalog
            .faction(&faction_id)
            .map(|faction| faction.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        if let Some(action) = resolve_action(&goals[0], memory, &captain_name, &faction_name) {
            actions.push(action);
        }
        if !actions.is_empty() {
            break;
        }
    }
    actions
}

pub fn get_or_create_memory<'a>(
    memories: &'a mut Vec<CaptainMemory>,
    captain_id: &str,
) -> &'a mut CaptainMemory {
    if let Some(index) = memories
        .iter()
        .position(|memory| memory.captain_id == captain_id)
    {
        return &mut memories[index];
    }
    memories.push(CaptainMemory::new(captain_id));
    memories.last_mut().expect("just pushed")
}

pub fn relationship_summary(memory: &CaptainMemory) -> RelationshipSummary {
    RelationshipSummary {
        breakpoint: derive_breakpoint(&memory.relationship).to_string(),
        respect: memory.relationship.respect,
        fear: memory.relationship.fear,
        grudge: memory.relationship.grudge,
        familiarity: memory.relationship.familiarity,
        encounters: memory.encounters.len(),
        times_spared: memory.times_spared,
        last_seen_day: memory.last_seen_day,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparing_a_win_moves_respect_and_clears_grudge() {
        let mut memory = CaptainMemory::new("scarlet_ana");
        let enc = record_encounter(
            &mut memory,
            4,
            "Mediterranean",
            "player_won",
            true,
            false,
            0,
        );
        assert_eq!(enc.respect_delta, 25);
        assert_eq!(enc.grudge_delta, -10);
        assert_eq!(memory.relationship.respect, 25);
        assert_eq!(memory.times_spared, 1);
        assert_eq!(memory.times_defeated_by_player, 1);
        assert_eq!(relationship_summary(&memory).breakpoint, "neutral");
        record_encounter(&mut memory, 5, "Mediterranean", "player_won", true, true, 6);
        assert_eq!(memory.relationship.respect, 45);
        assert_eq!(memory.relationship.fear, 35);
        assert_eq!(memory.relationship.grudge, 15);
        assert_eq!(memory.relationship.familiarity, 30);
        assert_eq!(derive_breakpoint(&memory.relationship), "neutral");
    }

    #[test]
    fn md5_stagger_matches_cpython_and_neutral_extort_uses_familiarity() {
        assert_eq!(md5_prefix("red_beard:1"), 0xd3cc_2ccd);
        let mut memory = CaptainMemory::new("scarlet_ana");
        record_encounter(
            &mut memory,
            1,
            "Mediterranean",
            "negotiated",
            false,
            false,
            0,
        );
        assert_eq!(memory.relationship.familiarity, 5);
        let goals = derive_goals(&memory, "Mediterranean", "Mediterranean", 400, 2);
        assert_eq!(goals[0].verb, "extort");
        let action = resolve_action(&goals[0], &memory, "Scarlet Ana", "The Crimson Tide").unwrap();
        assert_eq!(action.effect_value, 35);
        assert!(action.message.contains("Tribute. 35 silver"));
    }
}
