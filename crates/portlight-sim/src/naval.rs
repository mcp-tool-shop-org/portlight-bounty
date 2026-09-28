//! Ship-to-ship combat. Port of `engine/naval.py`.
//!
//! Broadside, close, evade, and rake. A ship with no cannons can only close,
//! evade, or flee. Boarding opens once the close counter reaches the threshold.
//! Hull reduction uses Python `int()` truncation ([`crate::util::py_trunc`]).

use crate::content;
use crate::model::{Captain, OwnedShip, Ship};
use crate::pyrand::PyRandom;
use crate::util::py_trunc;

pub const NAVAL_ACTIONS: [&str; 4] = ["broadside", "close", "evade", "rake"];
const BOARDING_BASE_THRESHOLD: i64 = 3;

#[derive(Debug, Clone)]
pub struct EnemyShip {
    pub name: String,
    pub hull: i64,
    pub hull_max: i64,
    pub cannons: i64,
    pub maneuver: f64,
    pub speed: f64,
    pub crew: i64,
    pub crew_max: i64,
}

#[derive(Debug, Clone)]
pub struct NavalRound {
    pub turn: i64,
    pub player_action: String,
    pub enemy_action: String,
    pub player_hull_delta: i64,
    pub enemy_hull_delta: i64,
    pub player_crew_delta: i64,
    pub enemy_crew_delta: i64,
    pub boarding_progress: i64,
    pub flavor: String,
}

/// Pirate hull from captain strength.
///
/// 1–3 sloop, 4–6 cutter, 7–8 brigantine, 9–10 galleon. One `random()` call
/// sets speed in every band.
pub fn generate_enemy_ship(captain_name: &str, strength: i64, rng: &mut PyRandom) -> EnemyShip {
    let (hull, cannons, maneuver, speed, crew, crew_max) = if strength <= 3 {
        let hull = 40 + strength * 8;
        let crew = 4 + strength;
        (hull, 0, 0.85, 7.0 + rng.random(), crew, crew + 3)
    } else if strength <= 6 {
        let hull = 60 + strength * 10;
        let cannons = 2 + (strength - 3) * 2;
        let maneuver = 0.6 + (6 - strength) as f64 * 0.05;
        let crew = 8 + (strength - 3) * 4;
        (
            hull,
            cannons,
            maneuver,
            6.0 + rng.random() * 0.5,
            crew,
            crew + 5,
        )
    } else if strength <= 8 {
        let hull = 90 + strength * 10;
        let cannons = 6 + (strength - 6) * 3;
        let maneuver = 0.4 + (8 - strength) as f64 * 0.05;
        let crew = 18 + (strength - 6) * 5;
        (
            hull,
            cannons,
            maneuver,
            5.0 + rng.random() * 0.5,
            crew,
            crew + 8,
        )
    } else {
        let hull = 130 + strength * 8;
        let cannons = 12 + (strength - 8) * 4;
        let crew = 30 + (strength - 8) * 8;
        (
            hull,
            cannons,
            0.25,
            4.0 + rng.random() * 0.5,
            crew,
            crew + 10,
        )
    };
    EnemyShip {
        name: format!("{captain_name}'s Ship"),
        hull,
        hull_max: hull,
        cannons,
        maneuver,
        speed,
        crew,
        crew_max,
    }
}

/// `(escaped, hull_damage)`. Failure is a free broadside, not a stance duel.
pub fn attempt_flee(player: &Ship, enemy: &EnemyShip, rng: &mut PyRandom) -> (bool, i64) {
    let mut escape_chance = 0.3 + (player.speed - enemy.speed) * 0.1 + player.maneuver * 0.2;
    escape_chance = escape_chance.clamp(0.1, 0.95);
    if rng.random() < escape_chance {
        let graze = if enemy.cannons > 0 {
            rng.randint(1, 3)
        } else {
            0
        };
        return (true, graze);
    }
    if enemy.cannons > 0 {
        let mut damage = enemy.cannons * rng.randint(1, 2);
        damage = 1.max(py_trunc(damage as f64 * (1.0 - player.maneuver * 0.3)));
        (false, damage)
    } else {
        (false, rng.randint(2, 5))
    }
}

pub fn valid_actions(cannons: i64) -> &'static [&'static str] {
    if cannons <= 0 {
        &["close", "evade", "flee"]
    } else {
        &["broadside", "close", "evade", "rake", "flee"]
    }
}

pub fn boarding_threshold(player_maneuver: f64) -> i64 {
    let threshold = BOARDING_BASE_THRESHOLD - (player_maneuver * 2.0).floor() as i64;
    threshold.max(1)
}

fn naval_weights(personality: &str) -> [(&'static str, f64); 4] {
    match personality {
        "aggressive" => [
            ("broadside", 0.45),
            ("close", 0.35),
            ("evade", 0.05),
            ("rake", 0.15),
        ],
        "defensive" => [
            ("broadside", 0.20),
            ("close", 0.10),
            ("evade", 0.40),
            ("rake", 0.30),
        ],
        "wild" => [
            ("broadside", 0.30),
            ("close", 0.25),
            ("evade", 0.20),
            ("rake", 0.25),
        ],
        _ => [
            ("broadside", 0.35),
            ("close", 0.25),
            ("evade", 0.20),
            ("rake", 0.20),
        ],
    }
}

pub fn pick_enemy_naval_action(
    personality: &str,
    enemy: &EnemyShip,
    player: &Ship,
    boarding_progress: i64,
    boarding_threshold: i64,
    rng: &mut PyRandom,
) -> String {
    let mut weights = naval_weights(personality);
    if enemy.cannons <= 0 {
        for (action, weight) in &mut weights {
            match *action {
                "broadside" | "rake" => *weight = 0.0,
                "close" => *weight += 0.5,
                "evade" => *weight += 0.2,
                _ => {}
            }
        }
    }
    if personality == "balanced" {
        let hull_ratio = enemy.hull as f64 / enemy.hull_max.max(1) as f64;
        let player_hull_ratio = player.hull as f64 / player.hull_max.max(1) as f64;
        if hull_ratio > player_hull_ratio + 0.2 {
            bump(&mut weights, "broadside", 0.15);
            bump(&mut weights, "close", 0.10);
        } else if hull_ratio < player_hull_ratio - 0.2 {
            bump(&mut weights, "evade", 0.15);
            bump(&mut weights, "rake", 0.10);
        }
    }
    if boarding_progress >= boarding_threshold - 1 {
        bump(&mut weights, "close", 0.30);
    }
    let values: Vec<f64> = weights.iter().map(|(_, w)| w.max(0.0)).collect();
    let total: f64 = values.iter().sum();
    if total <= 0.0 {
        return "evade".to_string();
    }
    weights[rng.choices_weighted(&values)].0.to_string()
}

fn bump(weights: &mut [(&str, f64)], action: &str, extra: f64) {
    if let Some(slot) = weights.iter_mut().find(|(name, _)| *name == action) {
        slot.1 += extra;
    }
}

fn naval_flavor(player: &str, enemy: &str) -> &'static str {
    match (player, enemy) {
        ("broadside", "broadside") => {
            "Cannons roar from both ships — smoke and splinters fill the air."
        }
        ("broadside", "close") => {
            "Your broadside catches them closing. Wood splinters fly as they press forward."
        }
        ("broadside", "evade") => "They weave hard but your guns still find timber.",
        ("broadside", "rake") => {
            "Both ships fire — your broadside hammers their hull while their high shot sweeps your deck."
        }
        ("close", "broadside") => {
            "You press forward through their broadside. Iron and wood fly past your bow."
        }
        ("close", "close") => "Both ships close, hulls nearly touching. Grappling hooks ready.",
        ("close", "evade") => "You lunge forward but they slip aside. The gap holds.",
        ("close", "rake") => "You close through a hail of grapeshot aimed at your crew.",
        ("evade", "broadside") => "You turn hard. Their broadside catches only wake and spray.",
        ("evade", "close") => "They try to close but your evasion keeps them at arm's length.",
        ("evade", "evade") => "Both ships circle warily. Neither commits.",
        ("evade", "rake") => "You evade but their high shot still peppers the rigging.",
        ("rake", "broadside") => {
            "Your grapeshot rakes their deck as their broadside finds your hull."
        }
        ("rake", "close") => "Your crew-killing shot catches them as they close.",
        ("rake", "evade") => "They evade but your grapeshot still finds a few sailors.",
        ("rake", "rake") => "Both ships fire high — crew on both sides fall to grapeshot.",
        _ => "The ships maneuver across the waves.",
    }
}

pub fn resolve_naval_round(
    player_action: &str,
    enemy_action: &str,
    player: &Ship,
    enemy: &EnemyShip,
    boarding_progress: i64,
    rng: &mut PyRandom,
) -> NavalRound {
    let mut p_hull_delta = 0;
    let mut e_hull_delta = 0;
    let mut p_crew_delta = 0;
    let mut e_crew_delta = 0;
    let mut new_boarding = boarding_progress;

    if player_action == "broadside" && player.cannons > 0 {
        let base = player.cannons * rng.randint(1, 3);
        let mut reduced = 1.max(py_trunc(base as f64 * (1.0 - enemy.maneuver * 0.3)));
        if enemy_action == "evade" {
            reduced = 0.max(reduced / 2);
        }
        e_hull_delta -= reduced;
    } else if player_action == "rake" && player.cannons > 0 {
        let mut hull_dmg = player.cannons * rng.randint(0, 1);
        if enemy_action == "evade" {
            hull_dmg = 0.max(hull_dmg / 2);
        }
        e_hull_delta -= hull_dmg;
        let mut crew_kill = rng.randint(1, 1.max(player.cannons / 3 + 1));
        if enemy_action == "evade" {
            crew_kill = 0.max(crew_kill / 2);
        }
        e_crew_delta -= crew_kill;
    } else if player_action == "close" && enemy_action != "evade" {
        new_boarding += 1;
    }

    if enemy_action == "broadside" && enemy.cannons > 0 {
        let base = enemy.cannons * rng.randint(1, 3);
        let mut reduced = 1.max(py_trunc(base as f64 * (1.0 - player.maneuver * 0.3)));
        if player_action == "evade" {
            reduced = 0.max(reduced / 2);
        }
        p_hull_delta -= reduced;
    } else if enemy_action == "rake" && enemy.cannons > 0 {
        let mut hull_dmg = enemy.cannons * rng.randint(0, 1);
        if player_action == "evade" {
            hull_dmg = 0.max(hull_dmg / 2);
        }
        p_hull_delta -= hull_dmg;
        let mut crew_kill = rng.randint(1, 1.max(enemy.cannons / 3 + 1));
        if player_action == "evade" {
            crew_kill = 0.max(crew_kill / 2);
        }
        p_crew_delta -= crew_kill;
    } else if enemy_action == "close" && player_action != "evade" {
        new_boarding += 1;
    }

    NavalRound {
        turn: 0,
        player_action: player_action.to_string(),
        enemy_action: enemy_action.to_string(),
        player_hull_delta: p_hull_delta,
        enemy_hull_delta: e_hull_delta,
        player_crew_delta: p_crew_delta,
        enemy_crew_delta: e_crew_delta,
        boarding_progress: new_boarding,
        flavor: naval_flavor(player_action, enemy_action).to_string(),
    }
}

/// `(player_crew_lost, enemy_crew_lost, player_advantage)`.
pub fn resolve_boarding(player_crew: i64, enemy_crew: i64, rng: &mut PyRandom) -> (i64, i64, bool) {
    let ratio = player_crew as f64 / enemy_crew.max(1) as f64;
    let (mut p_lost, mut e_lost) = if ratio > 2.0 {
        (rng.randint(0, 1), rng.randint(2, 4))
    } else if ratio > 1.5 {
        (rng.randint(1, 3), rng.randint(2, 4))
    } else if ratio > 0.67 {
        (rng.randint(2, 5), rng.randint(2, 5))
    } else {
        (rng.randint(3, 6), rng.randint(1, 3))
    };
    p_lost = p_lost.min(0.max(player_crew - 1));
    e_lost = e_lost.min(enemy_crew);
    let player_advantage = player_crew - p_lost > enemy_crew - e_lost;
    (p_lost, e_lost, player_advantage)
}

/// Sailors first, then specialists, matching `apply_crew_casualties`.
pub fn apply_crew_loss(ship: &mut Ship, lost: i64) -> i64 {
    if lost <= 0 {
        return 0;
    }
    let current = ship.roster_total();
    if current <= 0 {
        let applied = lost.min(0.max(ship.crew));
        ship.crew = 0.max(ship.crew - lost);
        return applied;
    }
    let lost = lost.min(current);
    let mut remaining = lost;
    let take = ship.sailors.min(remaining);
    ship.sailors -= take;
    remaining -= take;
    let slots: [(&mut i64, &str); 5] = [
        (&mut ship.gunners, "gunner"),
        (&mut ship.navigators, "navigator"),
        (&mut ship.surgeons, "surgeon"),
        (&mut ship.marines, "marine"),
        (&mut ship.quartermasters, "quartermaster"),
    ];
    for (count, role) in slots {
        if remaining <= 0 {
            break;
        }
        let take = (*count).min(remaining);
        if take <= 0 {
            continue;
        }
        *count -= take;
        remaining -= take;
        let mut to_drop = take;
        let mut kept = Vec::new();
        for officer in ship.officers.iter().rev() {
            if officer.role == role && to_drop > 0 {
                to_drop -= 1;
            } else {
                kept.push(officer.clone());
            }
        }
        kept.reverse();
        ship.officers = kept;
    }
    ship.sync_crew();
    lost - remaining
}

pub fn prize_template_id(strength: i64) -> &'static str {
    match strength {
        1..=3 => "coastal_sloop",
        4..=5 => "swift_cutter",
        6..=7 => "trade_brigantine",
        8..=10 => "merchant_galleon",
        _ => "coastal_sloop",
    }
}

pub fn max_fleet_size(commercial_trust: i64) -> i64 {
    if commercial_trust >= 26 {
        5
    } else if commercial_trust >= 11 {
        3
    } else {
        2
    }
}

pub fn can_capture_prize(captain: &Captain, enemy_strength: i64) -> (bool, String) {
    let fleet_count = captain.fleet.len() as i64 + 1;
    if fleet_count >= max_fleet_size(captain.standing.commercial_trust) {
        return (false, "Fleet is full".to_string());
    }
    let catalog = content::content();
    let current_min = captain
        .ship
        .as_ref()
        .and_then(|ship| catalog.ship(&ship.template_id))
        .map(|ship| ship.crew_min)
        .unwrap_or(3);
    let prize_min = catalog
        .ship(prize_template_id(enemy_strength))
        .map(|ship| ship.crew_min)
        .unwrap_or(3);
    let have = captain.ship.as_ref().map(|ship| ship.crew).unwrap_or(0);
    if have < current_min + prize_min {
        return (
            false,
            format!(
                "Need at least {} crew to man both ships (have {have})",
                current_min + prize_min
            ),
        );
    }
    (true, String::new())
}

fn pick_casualty(ship: &Ship, rng: &mut PyRandom) -> Option<usize> {
    let mut weights = Vec::new();
    if ship.sailors > 0 {
        weights.push(3.0);
    }
    if ship.gunners > 0 {
        weights.push(1.5);
    }
    if ship.navigators > 0 {
        weights.push(0.5);
    }
    if ship.surgeons > 0 {
        weights.push(0.5);
    }
    if ship.marines > 0 {
        weights.push(2.0);
    }
    if ship.quartermasters > 0 {
        weights.push(0.5);
    }
    if weights.is_empty() {
        return None;
    }
    let mut roles = Vec::new();
    if ship.sailors > 0 {
        roles.push(0);
    }
    if ship.gunners > 0 {
        roles.push(1);
    }
    if ship.navigators > 0 {
        roles.push(2);
    }
    if ship.surgeons > 0 {
        roles.push(3);
    }
    if ship.marines > 0 {
        roles.push(4);
    }
    if ship.quartermasters > 0 {
        roles.push(5);
    }
    Some(roles[rng.choices_weighted(&weights)])
}

fn add_role(ship: &mut Ship, role: usize) {
    match role {
        0 => ship.sailors += 1,
        1 => ship.gunners += 1,
        2 => ship.navigators += 1,
        3 => ship.surgeons += 1,
        4 => ship.marines += 1,
        _ => ship.quartermasters += 1,
    }
}

fn remove_role(ship: &mut Ship, role: usize) {
    match role {
        0 => ship.sailors -= 1,
        1 => ship.gunners -= 1,
        2 => ship.navigators -= 1,
        3 => ship.surgeons -= 1,
        4 => ship.marines -= 1,
        _ => ship.quartermasters -= 1,
    }
    let role_name = match role {
        1 => "gunner",
        2 => "navigator",
        3 => "surgeon",
        4 => "marine",
        5 => "quartermaster",
        _ => "",
    };
    if role_name.is_empty() {
        return;
    }
    if let Some(idx) = ship
        .officers
        .iter()
        .rposition(|officer| officer.role == role_name)
    {
        ship.officers.remove(idx);
    }
}

/// Move `count` hands onto a prize hull. Returns the new ship.
pub fn capture_prize(
    captain: &mut Captain,
    enemy_name: &str,
    enemy_strength: i64,
    enemy_hull: i64,
    crew_to_prize: i64,
    docked_port_id: &str,
    rng: &mut PyRandom,
) -> OwnedShip {
    let catalog = content::content();
    let template_id = prize_template_id(enemy_strength);
    let template = catalog.ship(template_id);
    let hull_max = template.map(|t| t.hull_max).unwrap_or(enemy_hull.max(1));
    let hull_remaining = 1.max(enemy_hull).min(hull_max);
    let mut prize = Ship {
        template_id: template_id.to_string(),
        name: format!("{enemy_name}'s Prize"),
        hull: hull_remaining,
        hull_max,
        cargo_capacity: template.map(|t| t.cargo_capacity).unwrap_or(30),
        speed: template.map(|t| t.speed).unwrap_or(6.0),
        crew: 0,
        crew_max: template.map(|t| t.crew_max).unwrap_or(8),
        cannons: template.map(|t| t.cannons).unwrap_or(0),
        maneuver: template.map(|t| t.maneuver).unwrap_or(0.5),
        upgrade_slots: template
            .map(|t| content::upgrade_slots(&t.ship_class))
            .unwrap_or(2),
        morale: 50,
        sailors: 0,
        gunners: 0,
        navigators: 0,
        surgeons: 0,
        marines: 0,
        quartermasters: 0,
        officers: Vec::new(),
    };
    if let Some(flagship) = captain.ship.as_mut() {
        for _ in 0..crew_to_prize.max(0) {
            let Some(role) = pick_casualty(flagship, rng) else {
                break;
            };
            remove_role(flagship, role);
            add_role(&mut prize, role);
        }
        flagship.sync_crew();
        prize.sync_crew();
    }
    OwnedShip {
        ship: prize,
        docked_port_id: docked_port_id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Ship;

    fn cutter() -> Ship {
        Ship {
            template_id: "swift_cutter".into(),
            name: "S".into(),
            hull: 70,
            hull_max: 70,
            cargo_capacity: 50,
            speed: 9.0,
            crew: 5,
            crew_max: 12,
            cannons: 2,
            maneuver: 0.8,
            upgrade_slots: 3,
            morale: 50,
            sailors: 5,
            gunners: 0,
            navigators: 0,
            surgeons: 0,
            marines: 0,
            quartermasters: 0,
            officers: Vec::new(),
        }
    }

    #[test]
    fn strength_bands_and_flee_match_cpython_seed_42() {
        // generate_enemy_ship + attempt_flee against a cutter, seed 42.
        // Checked with CPython 3 against engine/naval.py.
        let cases = [
            (
                2,
                56,
                0,
                0.85,
                7.639_426_798_457_884,
                6,
                9,
                true,
                0,
                true,
                0,
                true,
                0,
            ),
            (
                5,
                110,
                6,
                0.65,
                6.319_713_399_228_942,
                16,
                21,
                true,
                2,
                true,
                1,
                false,
                4,
            ),
            (
                8,
                170,
                12,
                0.4,
                5.319_713_399_228_942,
                28,
                36,
                true,
                2,
                true,
                1,
                true,
                3,
            ),
            (
                9,
                202,
                16,
                0.25,
                4.319_713_399_228_942,
                38,
                48,
                true,
                2,
                true,
                1,
                true,
                3,
            ),
        ];
        for (strength, hull, cannons, maneuver, speed, crew, crew_max, e1, d1, e2, d2, e3, d3) in
            cases
        {
            let mut rng = PyRandom::from_seed(42);
            let enemy = generate_enemy_ship("Band", strength, &mut rng);
            assert_eq!(enemy.hull, hull, "hull {strength}");
            assert_eq!(enemy.cannons, cannons, "guns {strength}");
            assert!(
                (enemy.maneuver - maneuver).abs() < 1e-12,
                "maneuver {strength}"
            );
            assert!(
                (enemy.speed - speed).abs() < 1e-12,
                "speed {strength} {}",
                enemy.speed
            );
            assert_eq!((enemy.crew, enemy.crew_max), (crew, crew_max));
            let player = cutter();
            assert_eq!(attempt_flee(&player, &enemy, &mut rng), (e1, d1));
            assert_eq!(attempt_flee(&player, &enemy, &mut rng), (e2, d2));
            assert_eq!(attempt_flee(&player, &enemy, &mut rng), (e3, d3));
        }
    }

    #[test]
    fn prize_template_follows_strength() {
        assert_eq!(prize_template_id(2), "coastal_sloop");
        assert_eq!(prize_template_id(5), "swift_cutter");
        assert_eq!(prize_template_id(7), "trade_brigantine");
        assert_eq!(prize_template_id(9), "merchant_galleon");
        assert_eq!(prize_template_id(10), "merchant_galleon");
    }

    #[test]
    fn boarding_threshold_uses_floor_of_maneuver() {
        assert_eq!(boarding_threshold(0.9), 2);
        assert_eq!(boarding_threshold(0.8), 2);
        assert_eq!(boarding_threshold(0.3), 3);
        assert_eq!(boarding_threshold(0.0), 3);
    }
}
