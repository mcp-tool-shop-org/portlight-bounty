//! Named-pirate bounties. Port of `engine/bounty.py`.
//!
//! `generate_bounty_board` is line 82, `accept_bounty` line 120,
//! `hunt_bounty` line 134, and `claim_bounty` line 165. Ghost captain ids
//! never list or spawn.

use crate::content::content;
use crate::encounter::{self, EncounterState};
use crate::model::{Captain, CaptainMemory, World};
use crate::pyrand::PyRandom;

/// A pirate captain with a price on their head. Python `BountyTarget`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BountyTarget {
    pub captain_id: String,
    pub captain_name: String,
    pub faction_id: String,
    pub region: String,
    pub reward: i64,
    pub difficulty: String,
    pub description: String,
}

struct BountyRow {
    id: &'static str,
    name: &'static str,
    faction_id: &'static str,
    region: &'static str,
    reward: i64,
    difficulty: &'static str,
    description: &'static str,
}

/// Live `PIRATE_CAPTAINS` only. Ghost ids never appear here.
const PIRATE_BOUNTIES: &[BountyRow] = &[
    BountyRow {
        id: "scarlet_ana",
        name: "Scarlet Ana",
        faction_id: "crimson_tide",
        region: "North Atlantic",
        reward: 150,
        difficulty: "moderate",
        description: "Captain of the Crimson Tide's diplomatic fleet. Deals first, fights well.",
    },
    BountyRow {
        id: "the_butcher",
        name: "The Butcher",
        faction_id: "crimson_tide",
        region: "Mediterranean",
        reward: 220,
        difficulty: "hard",
        description: "Crimson Tide enforcer. No diplomacy. Takes what he wants.",
    },
    BountyRow {
        id: "raj_the_quiet",
        name: "Raj the Quiet",
        faction_id: "monsoon_syndicate",
        region: "East Indies",
        reward: 120,
        difficulty: "easy",
        description: "Syndicate spymaster. Knows your hold before you open it.",
    },
    BountyRow {
        id: "typhoon_mei",
        name: "Typhoon Mei",
        faction_id: "monsoon_syndicate",
        region: "East Indies",
        reward: 200,
        difficulty: "hard",
        description: "The monsoon in human form. Controls the eastern sea lanes with chaos.",
    },
    BountyRow {
        id: "old_coral",
        name: "Old Coral",
        faction_id: "deep_reef",
        region: "South Seas",
        reward: 160,
        difficulty: "moderate",
        description: "Brotherhood elder. Fifty years on the reef. Respects courage.",
    },
    BountyRow {
        id: "the_diver",
        name: "The Diver",
        faction_id: "deep_reef",
        region: "West Africa",
        reward: 180,
        difficulty: "hard",
        description: "Boards from the waterline and is gone before steel is drawn.",
    },
    BountyRow {
        id: "sergeant_kruze",
        name: "Sergeant Kruze",
        faction_id: "iron_wolves",
        region: "North Atlantic",
        reward: 200,
        difficulty: "hard",
        description: "Former garrison sergeant. Runs piracy like a military operation.",
    },
    BountyRow {
        id: "gnaw",
        name: "Gnaw",
        faction_id: "iron_wolves",
        region: "North Atlantic",
        reward: 200,
        difficulty: "hard",
        description: "Most feared pirate in the North Atlantic. Destroys what he cannot take.",
    },
];

fn is_live_captain(target_id: &str) -> bool {
    content().pirate(target_id).is_some()
}

fn times_defeated(memories: &[CaptainMemory], captain_id: &str) -> i64 {
    memories
        .iter()
        .find(|memory| memory.captain_id == captain_id)
        .map(|memory| memory.times_defeated_by_player)
        .unwrap_or(0)
}

fn row_to_target(row: &BountyRow) -> BountyTarget {
    BountyTarget {
        captain_id: row.id.to_string(),
        captain_name: row.name.to_string(),
        faction_id: row.faction_id.to_string(),
        region: row.region.to_string(),
        reward: row.reward,
        difficulty: row.difficulty.to_string(),
        description: row.description.to_string(),
    }
}

/// `generate_bounty_board` (line 82).
///
/// Drops captains the player has already defeated. When more than
/// `max_targets` remain, one `Random.sample` draws that many. A shorter
/// list returns in table order and does not draw.
pub fn generate_bounty_board(
    memories: &[CaptainMemory],
    rng: &mut PyRandom,
    max_targets: usize,
) -> Vec<BountyTarget> {
    let available: Vec<BountyTarget> = PIRATE_BOUNTIES
        .iter()
        .filter(|row| times_defeated(memories, row.id) <= 0 && is_live_captain(row.id))
        .map(row_to_target)
        .collect();
    if available.len() <= max_targets {
        return available;
    }
    rng.sample_indices(available.len(), max_targets)
        .into_iter()
        .map(|index| available[index].clone())
        .collect()
}

/// `accept_bounty` (line 120). `Ok(())` is Python's `None`.
pub fn accept_bounty(captain: &mut Captain, target_id: &str) -> Result<(), String> {
    if !is_live_captain(target_id) {
        return Err("Unknown captain".into());
    }
    if captain.active_bounties.iter().any(|id| id == target_id) {
        return Err("Already hunting this target".into());
    }
    if captain.claimed_bounties.iter().any(|id| id == target_id) {
        return Err("Bounty already claimed".into());
    }
    if captain.active_bounties.len() >= 3 {
        return Err("Maximum 3 active bounties".into());
    }
    captain.active_bounties.push(target_id.to_string());
    Ok(())
}

/// `hunt_bounty` (line 134).
///
/// Locks `create_encounter` to an accepted, living target. The caller stores
/// the pending duel the way `GameSession.hunt_bounty_cmd` does.
pub fn hunt_bounty(
    world: &World,
    target_id: &str,
    rng: &mut PyRandom,
) -> Result<EncounterState, String> {
    if !is_live_captain(target_id) {
        return Err("Unknown captain".into());
    }
    if world
        .captain
        .claimed_bounties
        .iter()
        .any(|id| id == target_id)
    {
        return Err("Bounty already claimed".into());
    }
    if !world
        .captain
        .active_bounties
        .iter()
        .any(|id| id == target_id)
    {
        return Err("No active bounty for this target".into());
    }
    encounter::create_encounter(world, rng, Some(target_id)).ok_or_else(|| "Unknown captain".into())
}

/// `claim_bounty` (line 165). Silver on success, or the Python error sentence.
pub fn claim_bounty(
    captain: &mut Captain,
    memories: &[CaptainMemory],
    target_id: &str,
) -> Result<i64, String> {
    if !captain.active_bounties.iter().any(|id| id == target_id) {
        return Err("No active bounty for this target".into());
    }
    if captain.claimed_bounties.iter().any(|id| id == target_id) {
        return Err("Bounty already claimed".into());
    }
    if times_defeated(memories, target_id) <= 0 {
        return Err("Target not yet defeated. Find and defeat them at sea.".into());
    }
    let reward = PIRATE_BOUNTIES
        .iter()
        .find(|row| row.id == target_id)
        .map(|row| row.reward)
        .unwrap_or(0);
    if reward == 0 {
        return Err("Unknown bounty target".into());
    }
    captain.silver += reward;
    if let Some(index) = captain
        .active_bounties
        .iter()
        .position(|id| id == target_id)
    {
        captain.active_bounties.remove(index);
    }
    if !captain.claimed_bounties.iter().any(|id| id == target_id) {
        captain.claimed_bounties.push(target_id.to_string());
    }
    Ok(reward)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory;
    use crate::world::new_game;

    #[test]
    fn board_sample_matches_cpython() {
        let mut rng = PyRandom::from_seed(1);
        let board = generate_bounty_board(&[], &mut rng, 3);
        let ids: Vec<_> = board.iter().map(|row| row.captain_id.as_str()).collect();
        assert_eq!(ids, vec!["raj_the_quiet", "old_coral", "scarlet_ana"]);

        let mut rng = PyRandom::from_seed(42);
        let board = generate_bounty_board(&[], &mut rng, 3);
        let ids: Vec<_> = board.iter().map(|row| row.captain_id.as_str()).collect();
        assert_eq!(ids, vec!["the_butcher", "scarlet_ana", "the_diver"]);
    }

    #[test]
    fn defeated_captains_leave_the_board_without_a_draw() {
        let mut memories = Vec::new();
        for id in [
            "scarlet_ana",
            "the_butcher",
            "raj_the_quiet",
            "typhoon_mei",
            "old_coral",
            "the_diver",
        ] {
            let memory = memory::get_or_create_memory(&mut memories, id);
            memory.times_defeated_by_player = 1;
        }
        let mut rng = PyRandom::from_seed(1);
        let mut before = rng.clone();
        let board = generate_bounty_board(&memories, &mut rng, 3);
        let ids: Vec<_> = board.iter().map(|row| row.captain_id.as_str()).collect();
        assert_eq!(ids, vec!["sergeant_kruze", "gnaw"]);
        assert_eq!(rng.random(), before.random());
    }

    #[test]
    fn accept_hunt_and_claim_follow_python_refusals() {
        let mut world = new_game("Ada", "merchant", 1, None).unwrap();
        assert_eq!(
            accept_bounty(&mut world.captain, "shadow_vex").unwrap_err(),
            "Unknown captain"
        );
        accept_bounty(&mut world.captain, "raj_the_quiet").unwrap();
        assert_eq!(
            accept_bounty(&mut world.captain, "raj_the_quiet").unwrap_err(),
            "Already hunting this target"
        );
        assert_eq!(
            claim_bounty(&mut world.captain, &world.captain_memories, "raj_the_quiet").unwrap_err(),
            "Target not yet defeated. Find and defeat them at sea."
        );
        let memory = memory::get_or_create_memory(&mut world.captain_memories, "raj_the_quiet");
        memory.times_defeated_by_player = 1;
        let silver = world.captain.silver;
        assert_eq!(
            claim_bounty(&mut world.captain, &world.captain_memories, "raj_the_quiet").unwrap(),
            120
        );
        assert_eq!(world.captain.silver, silver + 120);
        assert!(world.captain.active_bounties.is_empty());
        assert_eq!(world.captain.claimed_bounties, vec!["raj_the_quiet"]);
        assert_eq!(
            accept_bounty(&mut world.captain, "raj_the_quiet").unwrap_err(),
            "Bounty already claimed"
        );
    }
}
