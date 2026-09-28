//! Port culture: festivals, sacred goods, and bloc price affinity.
//!
//! Ported from `engine/culture_engine.py` and the price rates in
//! `content/port_politics.py`. Sacred and forbidden goods change standing and
//! heat. An active festival multiplies that good's demand. A bloc's loyalty
//! bonus and disloyalty penalty scale a price. Python's session never calls
//! these adjustments; [`apply_trade`] and [`price_multiplier`] are the same
//! formulas, available to a caller that wants them.

use crate::content::{self, FestivalDef};
use crate::model::{ActiveFestival, CulturalState, Standing};
use crate::pyrand::PyRandom;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrivalFlavor {
    pub dock_scene: String,
    pub greeting: String,
    pub landmark: String,
    pub cultural_note: String,
    pub active_festival: Option<String>,
    pub tavern_rumor: String,
}

pub fn generate_arrival_flavor(
    port_id: &str,
    region: &str,
    captain_standing: i64,
    day: i64,
    cultural_state: &CulturalState,
) -> Option<ArrivalFlavor> {
    let catalog = content::content();
    let pc = catalog.port_culture(port_id)?;
    let rc = catalog.region_culture(region);
    let greeting = if let Some(rc) = rc {
        if captain_standing >= 20 {
            format!("The harbor erupts in welcome. {}", rc.greeting)
        } else if captain_standing >= 10 {
            rc.greeting.clone()
        } else if captain_standing >= 0 {
            format!("A dock clerk nods curtly. {}", rc.greeting)
        } else {
            "The dockworkers eye you with suspicion. No greeting is offered.".to_string()
        }
    } else {
        String::new()
    };
    let mut active_festival = None;
    for fest in &cultural_state.active_festivals {
        if fest.port_id == port_id && fest.start_day <= day && day <= fest.end_day {
            if let Some(rc) = rc {
                active_festival = rc
                    .festivals
                    .iter()
                    .find(|def| def.id == fest.festival_id)
                    .map(|def| def.id.clone());
            }
            break;
        }
    }
    Some(ArrivalFlavor {
        dock_scene: pc.dock_scene.clone(),
        greeting,
        landmark: pc.landmark.clone(),
        cultural_note: pc.local_custom.clone(),
        active_festival,
        tavern_rumor: pc.tavern_rumor.clone(),
    })
}

/// Festivals that should start. Each pair is `(festival id, port id)`.
pub fn check_festival_trigger(
    region: &str,
    rng: &mut PyRandom,
    cultural_state: &CulturalState,
) -> Vec<(String, String)> {
    let Some(rc) = content::content().region_culture(region) else {
        return Vec::new();
    };
    let mut triggered = Vec::new();
    let region_ports: Vec<&str> = content::content()
        .ports
        .iter()
        .filter(|port| port.region == region)
        .map(|port| port.id.as_str())
        .collect();
    for festival in &rc.festivals {
        let already = cultural_state
            .active_festivals
            .iter()
            .any(|active| active.festival_id == festival.id);
        if already {
            continue;
        }
        if festival.frequency_days <= 0 {
            continue;
        }
        if rng.random() < (1.0 / festival.frequency_days as f64) && !region_ports.is_empty() {
            let port = region_ports[rng.choice_index(region_ports.len())];
            triggered.push((festival.id.clone(), port.to_string()));
        }
    }
    triggered
}

pub fn activate_festival(
    festival_id: &str,
    port_id: &str,
    day: i64,
    cultural_state: &mut CulturalState,
) -> Option<ActiveFestival> {
    let festival = find_festival(festival_id)?;
    let active = ActiveFestival {
        festival_id: festival.id.clone(),
        port_id: port_id.to_string(),
        start_day: day,
        end_day: day + festival.duration_days,
    };
    cultural_state.active_festivals.push(active.clone());
    Some(active)
}

pub fn expire_festivals(day: i64, cultural_state: &mut CulturalState) -> Vec<ActiveFestival> {
    let expired = cultural_state
        .active_festivals
        .iter()
        .filter(|fest| day > fest.end_day)
        .cloned()
        .collect();
    cultural_state
        .active_festivals
        .retain(|fest| day <= fest.end_day);
    expired
}

pub fn record_port_visit(port_id: &str, region: &str, cultural_state: &mut CulturalState) {
    cultural_state.add_visit(port_id);
    if !cultural_state
        .regions_entered
        .iter()
        .any(|entered| entered == region)
    {
        cultural_state.regions_entered.push(region.to_string());
    }
}

pub fn record_cultural_encounter(cultural_state: &mut CulturalState) {
    cultural_state.cultural_encounters += 1;
}

pub fn sacred_standing_bonus(good_id: &str, region: &str) -> i64 {
    match content::content().region_culture(region) {
        Some(rc) if rc.sacred_goods.iter().any(|id| id == good_id) => 2,
        _ => 0,
    }
}

pub fn forbidden_heat_penalty(good_id: &str, region: &str) -> i64 {
    match content::content().region_culture(region) {
        Some(rc) if rc.forbidden_goods.iter().any(|id| id == good_id) => 3,
        _ => 0,
    }
}

pub fn cultural_good_note(good_id: &str, region: &str) -> Option<&'static str> {
    let rc = content::content().region_culture(region)?;
    if rc.sacred_goods.iter().any(|id| id == good_id) {
        return Some(match good_id {
            "grain" => "sacred - never let a city starve",
            "medicines" => "sacred - the north remembers its plagues",
            "pearls" => "sacred - gift from the sea",
            "porcelain" => "sacred - master craftsmen are revered",
            "silk" => "sacred - a thousand years of weaving",
            _ => "sacred",
        });
    }
    if rc.forbidden_goods.iter().any(|id| id == good_id) {
        return Some(match good_id {
            "weapons" => "forbidden - banned by decree",
            _ => "forbidden",
        });
    }
    if rc.prized_goods.iter().any(|id| id == good_id) {
        return Some("prized");
    }
    None
}

/// Demand multiplier from a festival active at `port_id` on `day`.
///
/// `1.0` when no festival lists `good_id`. Python stores `market_effects` and
/// does not apply them inside `tick_markets`.
pub fn festival_demand(
    cultural_state: &CulturalState,
    region: &str,
    port_id: &str,
    good_id: &str,
    day: i64,
) -> f64 {
    let Some(rc) = content::content().region_culture(region) else {
        return 1.0;
    };
    for active in &cultural_state.active_festivals {
        if active.port_id != port_id || day < active.start_day || day > active.end_day {
            continue;
        }
        if let Some(fest) = rc
            .festivals
            .iter()
            .find(|fest| fest.id == active.festival_id)
        {
            if let Some(mult) = fest.market_effects.get(good_id) {
                return *mult;
            }
        }
    }
    1.0
}

/// Bloc loyalty and hostility applied to a displayed price.
///
/// High standing in the port's own region grants `loyalty_bonus` (a discount:
/// multiplier `1 - bonus`). Standing of 15 or more with a hostile bloc in any
/// of that bloc's regions adds `disloyalty_penalty`. The catalog describes
/// these rates; `GameSession` does not call them.
pub fn price_multiplier(port_id: &str, standing: &Standing) -> f64 {
    let catalog = content::content();
    let Some(profile) = catalog.port_politics(port_id) else {
        return 1.0;
    };
    let Some(bloc) = catalog.trade_bloc(&profile.bloc_id) else {
        return 1.0;
    };
    let Some(port) = catalog.port(port_id) else {
        return 1.0;
    };
    let mut mult = 1.0;
    if standing.regional_of(&port.region) >= 15 {
        mult -= bloc.loyalty_bonus;
    }
    let hostile = bloc_is_hostile(standing, bloc.hostile_to.as_slice());
    if hostile {
        mult += bloc.disloyalty_penalty;
    }
    mult
}

fn bloc_is_hostile(standing: &Standing, hostile_blocs: &[String]) -> bool {
    let catalog = content::content();
    for bloc_id in hostile_blocs {
        let Some(bloc) = catalog.trade_bloc(bloc_id) else {
            continue;
        };
        for port_id in &bloc.port_ids {
            let Some(port) = catalog.port(port_id) else {
                continue;
            };
            if standing.regional_of(&port.region) >= 15 {
                return true;
            }
        }
    }
    false
}

/// Standing and heat a culturally marked sale would add.
///
/// Returns `(standing_delta, heat_delta)`. Python defines the checks and never
/// calls them from `execute_sell`.
pub fn trade_reputation(good_id: &str, region: &str) -> (i64, i64) {
    (
        sacred_standing_bonus(good_id, region),
        forbidden_heat_penalty(good_id, region),
    )
}

/// Write [`trade_reputation`] onto `standing`.
pub fn apply_trade(standing: &mut Standing, good_id: &str, region: &str) {
    let (standing_delta, heat_delta) = trade_reputation(good_id, region);
    if standing_delta != 0 {
        let current = standing.regional_of(region);
        standing.set_regional(region, (-20).max(current + standing_delta));
    }
    if heat_delta != 0 {
        let current = standing.heat_of(region);
        standing.set_heat(region, (0.max(current + heat_delta)).min(100));
    }
}

fn find_festival(id: &str) -> Option<&'static FestivalDef> {
    content::content()
        .culture
        .regions
        .iter()
        .flat_map(|region| region.festivals.iter())
        .find(|fest| fest.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pyrand::PyRandom;

    #[test]
    fn mediterranean_grain_is_sacred_and_weapons_are_not_the_only_note() {
        assert_eq!(sacred_standing_bonus("grain", "Mediterranean"), 2);
        assert_eq!(sacred_standing_bonus("silk", "Mediterranean"), 0);
        assert_eq!(
            cultural_good_note("grain", "Mediterranean"),
            Some("sacred - never let a city starve")
        );
        assert_eq!(forbidden_heat_penalty("weapons", "East Indies"), 3);
        assert_eq!(
            cultural_good_note("weapons", "East Indies"),
            Some("forbidden - banned by decree")
        );
    }

    #[test]
    fn festival_demand_uses_the_active_port() {
        let mut culture = CulturalState::default();
        activate_festival("harvest_of_plenty", "porto_novo", 10, &mut culture).unwrap();
        assert!(
            (festival_demand(&culture, "Mediterranean", "porto_novo", "grain", 11) - 1.8).abs()
                < 1e-9
        );
        assert!(
            (festival_demand(&culture, "Mediterranean", "porto_novo", "silk", 11) - 1.0).abs()
                < 1e-9
        );
        assert!(
            (festival_demand(&culture, "Mediterranean", "silva_bay", "grain", 11) - 1.0).abs()
                < 1e-9
        );
    }

    #[test]
    fn visit_records_a_region_once() {
        let mut culture = CulturalState::default();
        record_port_visit("porto_novo", "Mediterranean", &mut culture);
        record_port_visit("porto_novo", "Mediterranean", &mut culture);
        record_port_visit("sun_harbor", "West Africa", &mut culture);
        assert_eq!(culture.visits("porto_novo"), 2);
        assert_eq!(
            culture.regions_entered,
            vec!["Mediterranean".to_string(), "West Africa".to_string()]
        );
    }

    #[test]
    fn loyalty_discounts_a_friendly_region() {
        let mut standing = Standing::from_captain(content::content().captain("merchant").unwrap());
        let base = price_multiplier("porto_novo", &standing);
        // Corsairs Rest is a Shadow Port in the Mediterranean, and that bloc is
        // hostile to the Exchange Alliance. Standing 15 here is both loyalty
        // (0.08) and disloyalty (0.10).
        standing.set_regional("Mediterranean", 15);
        let loyal = price_multiplier("porto_novo", &standing);
        assert!((base - 1.0).abs() < 1e-9);
        assert!((loyal - 1.02).abs() < 1e-9);
        standing.set_regional("Mediterranean", 10);
        standing.set_regional("North Atlantic", 15);
        let iron = price_multiplier("ironhaven", &standing);
        assert!((1.0 - iron - 0.06).abs() < 1e-9);
    }

    #[test]
    fn festival_trigger_draws_the_session_rng() {
        let mut rng = PyRandom::from_seed(1);
        let culture = CulturalState::default();
        let first = check_festival_trigger("Mediterranean", &mut rng, &culture);
        let mut again = PyRandom::from_seed(1);
        let second = check_festival_trigger("Mediterranean", &mut again, &culture);
        assert_eq!(first, second);
    }
}
