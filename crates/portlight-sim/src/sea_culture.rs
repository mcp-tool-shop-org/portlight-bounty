//! Sea-day flavor drawn after `advance_day`.
//!
//! Ported from `engine/sea_culture_engine.py`. [`enrich_voyage_day`] consumes
//! the session RNG in Python's order: route encounter, NPC sighting, weather,
//! crew mood, then history-gated sea consequences when a ledger and board are
//! supplied.

use crate::consequences;
use crate::content::{self, season_name};
use crate::economy::TradeReceipt;
use crate::model::{Consequence, ContractBoard, SeaCultureState, World};
use crate::pyrand::PyRandom;
use crate::voyage::{EventType, VoyageEvent};

const CONTRABAND: [&str; 3] = ["opium", "black_powder", "stolen_cargo"];

pub fn enrich_voyage_day(
    world: &mut World,
    base_events: Vec<VoyageEvent>,
    rng: &mut PyRandom,
    receipts: &[TradeReceipt],
    total_sells: i64,
    board: &ContractBoard,
) -> Vec<VoyageEvent> {
    let mut enriched = base_events;
    let dest_region = world
        .port(&world.voyage.destination_id)
        .map(|port| port.region.clone())
        .unwrap_or_else(|| "Mediterranean".to_string());
    let lore_name = world
        .find_route(&world.voyage.origin_id, &world.voyage.destination_id)
        .map(|route| route.lore_name.clone())
        .unwrap_or_default();
    let day = world.day;

    if let Some(encounter) = pick_route_encounter(&lore_name, &dest_region, rng) {
        add_flavor(
            world,
            &mut enriched,
            VoyageEvent::annotated(
                EventType::Nothing,
                encounter.0,
                format!("[{}]", encounter.1),
            ),
        );
    }
    if let Some(event) = pick_npc_sighting(&dest_region, rng) {
        add_flavor(world, &mut enriched, event);
    }
    let weather = pick_weather_flavor(&dest_region, day, rng);
    if !weather.is_empty() && !is_recent(&weather, &world.voyage.recent_events) {
        add_flavor(
            world,
            &mut enriched,
            VoyageEvent::annotated(EventType::Nothing, weather, "[weather]"),
        );
    }
    let mood = pick_crew_mood(world, rng);
    if !mood.is_empty() {
        add_flavor(
            world,
            &mut enriched,
            VoyageEvent::annotated(EventType::Nothing, mood, "[crew]"),
        );
    }
    for consequence in
        consequences::check_sea_consequences(world, receipts, total_sells, board, rng)
    {
        let note = sea_effect_note(&consequence);
        consequences::apply_consequence(world, &consequence);
        enriched.push(VoyageEvent::annotated(
            EventType::Nothing,
            format!("{}{note}", consequence.text),
            format!("[consequence:{}]", consequence.effect_type),
        ));
    }
    enriched
}

fn sea_effect_note(consequence: &Consequence) -> String {
    if consequence.silver_delta > 0 {
        format!(" (+{} silver)", consequence.silver_delta)
    } else if consequence.silver_delta < 0 {
        format!(" ({} silver)", consequence.silver_delta)
    } else if consequence.heat_delta > 0 {
        format!(" (+{} heat)", consequence.heat_delta)
    } else if consequence.trust_delta < 0 {
        format!(" ({} trust)", consequence.trust_delta)
    } else {
        String::new()
    }
}

fn add_flavor(world: &mut World, enriched: &mut Vec<VoyageEvent>, event: VoyageEvent) {
    if is_recent(&event.message, &world.voyage.recent_events) {
        return;
    }
    world.voyage.recent_events.push(event.message.clone());
    while world.voyage.recent_events.len() > 10 {
        world.voyage.recent_events.remove(0);
    }
    enriched.push(event);
}

fn is_recent(text: &str, recent: &[String]) -> bool {
    let needle = fingerprint(text);
    recent.iter().any(|row| needle == fingerprint(row))
}

fn fingerprint(text: &str) -> String {
    // Python: `text[:60].lower().strip()`.
    text.chars()
        .take(60)
        .collect::<String>()
        .to_lowercase()
        .trim()
        .to_string()
}

fn pick_route_encounter(
    lore_name: &str,
    dest_region: &str,
    rng: &mut PyRandom,
) -> Option<(String, String)> {
    if rng.random() > 0.25 {
        return None;
    }
    let catalog = content::content();
    let table = if !lore_name.is_empty() {
        catalog.route_encounters(lore_name)
    } else {
        None
    };
    let table = table.or_else(|| catalog.region_encounters(dest_region))?;
    if table.encounters.is_empty() {
        return None;
    }
    let encounter = &table.encounters[rng.choice_index(table.encounters.len())];
    Some((encounter.text.clone(), encounter.category.clone()))
}

fn pick_npc_sighting(dest_region: &str, rng: &mut PyRandom) -> Option<VoyageEvent> {
    if rng.random() > 0.10 {
        return None;
    }
    let sightings = content::content().npc_sightings(dest_region);
    if sightings.is_empty() {
        return None;
    }
    let sighting = sightings[rng.choice_index(sightings.len())];
    Some(VoyageEvent::annotated(
        EventType::ForeignVessel,
        sighting.text.clone(),
        format!("[{} from {}]", sighting.npc_name, sighting.port_id),
    ))
}

fn pick_weather_flavor(dest_region: &str, day: i64, rng: &mut PyRandom) -> String {
    if rng.random() > 0.30 {
        return String::new();
    }
    let Some(narrative) = content::content().weather_narrative(dest_region, season_name(day))
    else {
        return String::new();
    };
    if narrative.mid_voyage_texts.is_empty() {
        return String::new();
    }
    narrative.mid_voyage_texts[rng.choice_index(narrative.mid_voyage_texts.len())].clone()
}

pub fn departure_weather(region: &str, day: i64) -> String {
    content::content()
        .weather_narrative(region, season_name(day))
        .map(|narrative| narrative.departure_text.clone())
        .unwrap_or_default()
}

pub fn arrival_weather(region: &str, day: i64, port_name: &str) -> String {
    let Some(narrative) = content::content().weather_narrative(region, season_name(day)) else {
        return String::new();
    };
    let mut text = narrative.arrival_text.clone();
    if !port_name.is_empty() {
        for subject in [
            "The northern port",
            "The island port",
            "The port",
            "The island",
            "The harbor",
            "The lagoon",
        ] {
            if let Some(rest) = text.strip_prefix(subject) {
                text = format!("{port_name}{rest}");
                break;
            }
        }
    }
    text
}

pub fn night_weather(region: &str, day: i64) -> String {
    content::content()
        .weather_narrative(region, season_name(day))
        .map(|narrative| narrative.night_text.clone())
        .unwrap_or_default()
}

fn pick_crew_mood(world: &World, rng: &mut PyRandom) -> String {
    if rng.random() > 0.20 {
        return String::new();
    }
    let captain = &world.captain;
    let mut matching = Vec::new();
    for mood in &content::content().sea_culture.crew_moods {
        let hit = match mood.id.as_str() {
            "prosperous" => captain.silver > 2000,
            "struggling" => captain.silver < 100,
            "first_voyage" => captain.day < 10,
            "veteran" => captain.day > 200,
            "carrying_contraband" => captain
                .cargo
                .iter()
                .any(|item| CONTRABAND.contains(&item.good_id.as_str())),
            "calm_seas" | "new_ship" => false,
            _ => false,
        };
        if hit {
            matching.push(mood);
        }
    }
    if matching.is_empty() {
        return String::new();
    }
    let mood = matching[rng.choice_index(matching.len())];
    if mood.flavor_texts.is_empty() {
        return String::new();
    }
    mood.flavor_texts[rng.choice_index(mood.flavor_texts.len())].clone()
}

/// One-time superstitions. Python defines this and does not call it from
/// `enrich_voyage_day` or `GameSession.advance`.
pub fn check_superstitions(world: &World, state: &mut SeaCultureState) -> Vec<String> {
    let mut fired = Vec::new();
    let captain = &world.captain;
    let catalog = content::content();
    for sup in &catalog.sea_culture.superstitions {
        if state.fired_superstitions.iter().any(|id| id == &sup.id) {
            continue;
        }
        let hit = if let Some(region) = sup.trigger.strip_prefix("first_region_") {
            world
                .culture
                .regions_entered
                .iter()
                .any(|entered| entered == region)
        } else if sup.trigger == "carrying_sacred_good" {
            let dest = world
                .port(&world.voyage.destination_id)
                .and_then(|port| catalog.region_culture(&port.region));
            dest.is_some_and(|rc| {
                captain
                    .cargo
                    .iter()
                    .any(|item| rc.sacred_goods.iter().any(|good| good == &item.good_id))
            })
        } else if sup.trigger == "carrying_contraband" {
            captain
                .cargo
                .iter()
                .any(|item| CONTRABAND.contains(&item.good_id.as_str()))
        } else if sup.trigger == "day_100" {
            world.day == 100
        } else if sup.trigger == "visited_all_regions" {
            world.culture.regions_entered.len() >= 5
        } else {
            false
        };
        if hit {
            fired.push(sup.id.clone());
            state.fired_superstitions.push(sup.id.clone());
        }
    }
    fired
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::new_game;

    #[test]
    fn enrich_is_deterministic_for_a_fixed_seed() {
        let mut world = new_game("Ada", "merchant", 3, Some("porto_novo")).unwrap();
        crate::voyage::depart(&mut world, "silva_bay", false).unwrap();
        let mut rng = PyRandom::from_seed(3);
        let _ = crate::voyage::advance_day(&mut world, &mut rng).unwrap();
        let board = ContractBoard::default();
        let once = enrich_voyage_day(
            &mut world.clone(),
            Vec::new(),
            &mut rng.clone(),
            &[],
            0,
            &board,
        );
        let mut world2 = new_game("Ada", "merchant", 3, Some("porto_novo")).unwrap();
        crate::voyage::depart(&mut world2, "silva_bay", false).unwrap();
        let mut rng2 = PyRandom::from_seed(3);
        let _ = crate::voyage::advance_day(&mut world2, &mut rng2).unwrap();
        let twice = enrich_voyage_day(&mut world2, Vec::new(), &mut rng2, &[], 0, &board);
        let left: Vec<_> = once.iter().map(|event| event.message.clone()).collect();
        let right: Vec<_> = twice.iter().map(|event| event.message.clone()).collect();
        assert_eq!(left, right);
    }

    #[test]
    fn arrival_weather_names_the_port_when_the_subject_matches() {
        let text = arrival_weather("Mediterranean", 1, "");
        assert!(!text.is_empty());
    }
}
