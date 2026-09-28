//! Pirate approach. Port of `engine/encounter.py`.
//!
//! The choice is negotiate, flee, or fight. `resolve_flee` calls
//! [`crate::naval::attempt_flee`]. Success leaves. Failure applies the
//! broadside and opens naval combat. This module never calls `duel.rs`.

use crate::combat::{self, CombatRound, CombatantState};
use crate::content::content;
use crate::model::{Captain, PirateEncounterRecord, Ship, Standing, World};
use crate::naval::{self, EnemyShip, NavalRound};
use crate::pyrand::PyRandom;

#[derive(Debug, Clone)]
pub struct EncounterState {
    pub enemy_captain_id: String,
    pub enemy_captain_name: String,
    pub enemy_faction_id: String,
    pub enemy_personality: String,
    pub enemy_strength: i64,
    pub enemy_region: String,
    pub enemy_ship_hull: i64,
    pub enemy_ship_hull_max: i64,
    pub enemy_ship_cannons: i64,
    pub enemy_ship_maneuver: f64,
    pub enemy_ship_speed: f64,
    pub enemy_ship_crew: i64,
    pub enemy_ship_crew_max: i64,
    pub phase: String,
    pub boarding_progress: i64,
    pub boarding_threshold: i64,
    pub naval_turns: i64,
    pub duel_turns: i64,
}

pub fn voyage_region(world: &World) -> String {
    world
        .port(&world.voyage.destination_id)
        .map(|port| port.region.clone())
        .unwrap_or_else(|| "Mediterranean".to_string())
}

fn hostility(standing: &Standing, faction_id: &str, captain_type: &str) -> &'static str {
    let mut value = standing
        .underworld
        .iter()
        .find(|(id, _)| id == faction_id)
        .map(|(_, value)| *value)
        .unwrap_or(0);
    if captain_type == "smuggler" {
        if let Some(faction) = content().faction(faction_id) {
            if matches!(
                faction.smuggler_attitude.as_str(),
                "friendly" | "cooperative" | "respectful"
            ) {
                value += 5;
            }
        }
    }
    if value >= 50 {
        "allied"
    } else if value >= 25 {
        "trade"
    } else if value >= 10 {
        "neutral"
    } else {
        "attack"
    }
}

/// Lock `target_captain_id` when set. `None` rolls a faction captain in the
/// destination region. Returns `None` when that captain or faction is missing.
pub fn create_encounter(
    world: &World,
    rng: &mut PyRandom,
    target_captain_id: Option<&str>,
) -> Option<EncounterState> {
    let region = voyage_region(world);
    let catalog = content();
    let (pirate_id, pirate_name, faction_id, personality, strength) =
        if let Some(target) = target_captain_id {
            let pirate = catalog.pirate(target)?;
            let faction = catalog.faction(&pirate.faction_id)?;
            (
                pirate.id.clone(),
                pirate.name.clone(),
                faction.id.clone(),
                pirate.personality.clone(),
                pirate.strength,
            )
        } else {
            let mut factions = catalog.factions_in(&region);
            if factions.is_empty() {
                factions = catalog.factions.first().into_iter().collect();
            }
            if factions.is_empty() {
                return None;
            }
            let faction = factions[rng.choice_index(factions.len())];
            let captains = catalog.captains_in_faction(&faction.id);
            if captains.is_empty() {
                return None;
            }
            let pirate = captains[rng.choice_index(captains.len())];
            (
                pirate.id.clone(),
                pirate.name.clone(),
                faction.id.clone(),
                pirate.personality.clone(),
                pirate.strength,
            )
        };
    Some(from_captain(
        &pirate_id,
        &pirate_name,
        &faction_id,
        &personality,
        strength,
        &region,
        rng,
    ))
}

/// A ship-band probe. No catalog captain sits in strength 1–3, so flee
/// goldens build the hull with [`naval::generate_enemy_ship`] directly.
pub fn create_band_encounter(world: &World, strength: i64, rng: &mut PyRandom) -> EncounterState {
    let region = voyage_region(world);
    from_captain(
        &format!("band-{strength}"),
        &format!("Band {strength}"),
        "",
        "balanced",
        strength,
        &region,
        rng,
    )
}

fn from_captain(
    id: &str,
    name: &str,
    faction_id: &str,
    personality: &str,
    strength: i64,
    region: &str,
    rng: &mut PyRandom,
) -> EncounterState {
    let enemy = naval::generate_enemy_ship(name, strength, rng);
    EncounterState {
        enemy_captain_id: id.to_string(),
        enemy_captain_name: name.to_string(),
        enemy_faction_id: faction_id.to_string(),
        enemy_personality: personality.to_string(),
        enemy_strength: strength,
        enemy_region: region.to_string(),
        enemy_ship_hull: enemy.hull,
        enemy_ship_hull_max: enemy.hull_max,
        enemy_ship_cannons: enemy.cannons,
        enemy_ship_maneuver: enemy.maneuver,
        enemy_ship_speed: enemy.speed,
        enemy_ship_crew: enemy.crew,
        enemy_ship_crew_max: enemy.crew_max,
        phase: "approach".to_string(),
        boarding_progress: 0,
        boarding_threshold: 3,
        naval_turns: 0,
        duel_turns: 0,
    }
}

pub fn enemy_ship(encounter: &EncounterState) -> EnemyShip {
    EnemyShip {
        name: format!("{}'s Ship", encounter.enemy_captain_name),
        hull: encounter.enemy_ship_hull,
        hull_max: encounter.enemy_ship_hull_max,
        cannons: encounter.enemy_ship_cannons,
        maneuver: encounter.enemy_ship_maneuver,
        speed: encounter.enemy_ship_speed,
        crew: encounter.enemy_ship_crew,
        crew_max: encounter.enemy_ship_crew_max,
    }
}

pub fn resolve_negotiate(
    encounter: &mut EncounterState,
    standing: &Standing,
    captain_type: &str,
    rng: &mut PyRandom,
) -> (bool, String) {
    let attitude = hostility(standing, &encounter.enemy_faction_id, captain_type);
    let name = &encounter.enemy_captain_name;
    match attitude {
        "allied" => {
            encounter.phase = "resolved".to_string();
            (
                true,
                format!("{name} recognizes you as an ally. Safe passage granted."),
            )
        }
        "trade" => {
            encounter.phase = "resolved".to_string();
            (
                true,
                format!("{name} agrees to let you pass. Professional courtesy."),
            )
        }
        "neutral" => {
            if rng.random() < 0.50 {
                encounter.phase = "resolved".to_string();
                (
                    true,
                    format!("{name} considers, then waves you through. This time."),
                )
            } else {
                (
                    false,
                    format!("{name} isn't interested in talking. Prepare to fight!"),
                )
            }
        }
        _ => (
            false,
            format!("{name} sees prey, not a diplomat. Steel it is."),
        ),
    }
}

pub fn resolve_flee(
    encounter: &mut EncounterState,
    player: &Ship,
    rng: &mut PyRandom,
) -> (bool, i64, String) {
    let enemy = enemy_ship(encounter);
    let (escaped, damage) = naval::attempt_flee(player, &enemy, rng);
    if escaped {
        encounter.phase = "resolved".to_string();
        let mut msg = "You break away!".to_string();
        if damage > 0 {
            msg.push_str(&format!(
                " A parting shot catches your hull for {damage} damage."
            ));
        }
        (true, damage, msg)
    } else {
        let msg = format!(
            "Can't outrun them! {}'s ship closes in. Their broadside rakes you for {damage} hull damage. Battle is joined!",
            encounter.enemy_captain_name
        );
        (false, damage, msg)
    }
}

pub fn begin_fight(encounter: &mut EncounterState, player: &Ship) -> String {
    encounter.phase = "naval".to_string();
    encounter.boarding_threshold = naval::boarding_threshold(player.maneuver);
    format!(
        "Battle stations! {} engages! Boarding threshold: {} close actions.",
        encounter.enemy_captain_name, encounter.boarding_threshold
    )
}

pub fn resolve_naval_turn(
    encounter: &mut EncounterState,
    player_action: &str,
    player: &Ship,
    rng: &mut PyRandom,
) -> NavalRound {
    let enemy = enemy_ship(encounter);
    let enemy_action = naval::pick_enemy_naval_action(
        &encounter.enemy_personality,
        &enemy,
        player,
        encounter.boarding_progress,
        encounter.boarding_threshold,
        rng,
    );
    let mut result = naval::resolve_naval_round(
        player_action,
        &enemy_action,
        player,
        &enemy,
        encounter.boarding_progress,
        rng,
    );
    encounter.enemy_ship_hull = 0.max(encounter.enemy_ship_hull + result.enemy_hull_delta);
    encounter.enemy_ship_crew = 0.max(encounter.enemy_ship_crew + result.enemy_crew_delta);
    encounter.boarding_progress = result.boarding_progress;
    encounter.naval_turns += 1;
    result.turn = encounter.naval_turns;
    let enemy_sunk = encounter.enemy_ship_hull <= 0;
    let boarding_triggered = encounter.boarding_progress >= encounter.boarding_threshold;
    if enemy_sunk {
        encounter.phase = "resolved".to_string();
    } else if boarding_triggered {
        encounter.phase = "boarding".to_string();
    }
    result
}

pub struct BoardingOutcome {
    pub player_crew_lost: i64,
    pub enemy_crew_lost: i64,
    pub player_advantage: bool,
    pub flavor: String,
}

pub fn resolve_boarding_phase(
    encounter: &mut EncounterState,
    player_crew: i64,
    rng: &mut PyRandom,
) -> BoardingOutcome {
    let (p_lost, e_lost, player_advantage) =
        naval::resolve_boarding(player_crew, encounter.enemy_ship_crew, rng);
    encounter.enemy_ship_crew = 0.max(encounter.enemy_ship_crew - e_lost);
    encounter.phase = "duel".to_string();
    let advantage = if player_advantage {
        "You have the advantage!"
    } else {
        "They outnumber you on deck!"
    };
    BoardingOutcome {
        player_crew_lost: p_lost,
        enemy_crew_lost: e_lost,
        player_advantage,
        flavor: format!(
            "Grappling hooks fly! Your crew boards the enemy vessel. You lose {p_lost} crew, they lose {e_lost}. {advantage} Now face {} blade to blade.",
            encounter.enemy_captain_name
        ),
    }
}

pub fn create_duel_combatants(
    encounter: &EncounterState,
    player: &Captain,
) -> (CombatantState, CombatantState) {
    let crew = player.ship.as_ref().map(|ship| ship.crew).unwrap_or(5);
    let throwing_ids: Vec<String> = player
        .throwing
        .iter()
        .flat_map(|weapon| vec![weapon.id.clone(); weapon.ammo.max(0) as usize])
        .collect();
    let throwing_count: i64 = player
        .throwing
        .iter()
        .map(|weapon| weapon.ammo.max(0))
        .sum();
    let p = combat::create_player_combatant(
        crew,
        player.active_style.as_deref(),
        &[],
        player.firearm.as_ref().map(|w| w.id.as_str()),
        player.firearm.as_ref().map(|w| w.ammo).unwrap_or(0),
        throwing_count,
        &throwing_ids,
        player.mechanical.as_ref().map(|w| w.id.as_str()),
        player.mechanical.as_ref().map(|w| w.ammo).unwrap_or(0),
        player.armor.as_ref().map(|a| a.id.as_str()),
        player.melee.as_ref().map(|w| w.id.as_str()),
        player
            .melee
            .as_ref()
            .map(|w| w.quality.as_str())
            .unwrap_or("standard"),
        player
            .firearm
            .as_ref()
            .map(|w| w.quality.as_str())
            .unwrap_or("standard"),
    );
    let opp_ammo = 2.min(encounter.enemy_strength / 4);
    let opp_throwing = 3.min(encounter.enemy_strength / 3);
    let o = combat::create_opponent_combatant(
        encounter.enemy_strength,
        opp_ammo,
        opp_throwing,
        None,
        None,
    );
    (p, o)
}

pub fn resolve_duel_turn(
    encounter: &mut EncounterState,
    player_action: &str,
    player: &mut CombatantState,
    opponent: &mut CombatantState,
    rng: &mut PyRandom,
) -> CombatRound {
    encounter.duel_turns += 1;
    let mut result = combat::resolve_combat_round(
        player_action,
        player,
        opponent,
        &encounter.enemy_personality,
        rng,
    );
    result.turn = encounter.duel_turns;
    combat::apply_round_to_states(&result, player, opponent);
    if player.hp <= 0 || opponent.hp <= 0 {
        encounter.phase = "resolved".to_string();
    }
    result
}

pub fn remember(captain: &mut Captain, encounter: &EncounterState, day: i64, outcome: &str) {
    captain.encounters.push(PirateEncounterRecord {
        captain_id: encounter.enemy_captain_id.clone(),
        faction_id: encounter.enemy_faction_id.clone(),
        day,
        outcome: outcome.to_string(),
        region: encounter.enemy_region.clone(),
    });
}

pub fn negotiate_outcome(attitude_success: bool, allied: bool) -> &'static str {
    if allied {
        "alliance"
    } else if attitude_success {
        "trade"
    } else {
        "attack"
    }
}
