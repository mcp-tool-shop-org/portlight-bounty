//! Custom captain builder. Python `engine/custom_captain.py`.
//!
//! [`validate_spec`] is `validate_spec` (line 96). [`build_custom_template`] is
//! `build_custom_template` (line 153). [`CustomCaptainSpec`] is the dataclass
//! at line 76. The CLI registers the template on `CAPTAIN_TEMPLATES` and then
//! calls `GameSession.new` (`app/cli.py` line 292). [`crate::session::Session::new_custom`]
//! does that for one world: the template is what [`crate::world::new_game_with_def`]
//! consumes, and the world keeps it so later voyage, inspection, and arrival
//! lookups find it. Pricing uses [`captain_template`], which falls back to the
//! merchant archetype when the custom template is missing.

use std::collections::BTreeMap;

use crate::content::{self, CaptainDef, InspectionDef, PricingDef, ReputationDef, VoyageModsDef};
use crate::error::SimError;
use crate::model::World;
use crate::util::py_round_places;

/// `TOTAL_SKILL_POINTS` (`custom_captain.py` line 71).
pub const TOTAL_SKILL_POINTS: i64 = 10;
/// `MAX_POINTS_PER_CATEGORY` (`custom_captain.py` line 72).
pub const MAX_POINTS_PER_CATEGORY: i64 = 7;

const TRADE_BUY: f64 = -0.01;
const TRADE_SELL: f64 = 0.01;
const TRADE_LUXURY: f64 = 0.03;
const TRADE_FEE: f64 = -0.03;
const SAIL_BURN: f64 = -0.03;
const SAIL_SPEED: f64 = 0.3;
const SAIL_STORM: f64 = 0.015;
const SAIL_CARGO: f64 = -0.03;
const SHADOW_INSPECTION: f64 = -0.06;
const SHADOW_UNDERWORLD: i64 = 5;
const REPUTATION_TRUST: i64 = 2;
const REPUTATION_STANDING: i64 = 2;

/// Script command shared with `tools/parity/oracle.py`.
pub const CUSTOM_USAGE: &str = "Usage: custom <name> <seed> [trade=N] [sailing=N] [shadow=N] [reputation=N] [home=PORT] [region=REGION] [title=TITLE] [bloc=BLOC] [faction=FACTION] [mentor=NPC] [backstory=TEXT] [port=PORT]";

/// Player choices. Python `CustomCaptainSpec` (line 76).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomCaptainSpec {
    pub name: String,
    pub title: String,
    pub home_port_id: String,
    pub home_region: String,
    pub trade_points: i64,
    pub sailing_points: i64,
    pub shadow_points: i64,
    pub reputation_points: i64,
    pub bloc_alignment: String,
    pub faction_alignment: String,
    pub mentor_npc_id: String,
    pub backstory: String,
}

impl Default for CustomCaptainSpec {
    fn default() -> Self {
        Self {
            name: "Captain".to_string(),
            title: "Freelance Captain".to_string(),
            home_port_id: "porto_novo".to_string(),
            home_region: "Mediterranean".to_string(),
            trade_points: 0,
            sailing_points: 0,
            shadow_points: 0,
            reputation_points: 0,
            bloc_alignment: String::new(),
            faction_alignment: String::new(),
            mentor_npc_id: String::new(),
            backstory: String::new(),
        }
    }
}

/// Python `CaptainTemplate` built for `CaptainType.CUSTOM`.
#[derive(Debug, Clone)]
pub struct CustomCaptainTemplate {
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub home_region: String,
    pub home_port_id: String,
    pub starting_silver: i64,
    pub starting_ship_id: String,
    pub starting_provisions: i64,
    pub pricing: PricingDef,
    pub voyage: VoyageModsDef,
    pub inspection: InspectionDef,
    pub reputation: ReputationDef,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub backstory: String,
    pub mentor_npc_id: String,
    pub bloc_alignment: String,
    pub faction_alignment: BTreeMap<String, i64>,
}

impl CustomCaptainTemplate {
    /// The slice [`crate::world::new_game_with_def`] and later modifier lookups use.
    pub fn to_captain_def(&self) -> CaptainDef {
        CaptainDef {
            id: self.id.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            home_region: self.home_region.clone(),
            home_port_id: self.home_port_id.clone(),
            mentor_npc_id: self.mentor_npc_id.clone(),
            starting_silver: self.starting_silver,
            starting_ship_id: self.starting_ship_id.clone(),
            starting_provisions: self.starting_provisions,
            pricing: self.pricing.clone(),
            voyage: self.voyage.clone(),
            inspection: self.inspection.clone(),
            reputation: self.reputation.clone(),
        }
    }
}

/// Parsed `custom` script command. `starting_port` is `new_game`'s override.
#[derive(Debug, Clone)]
pub struct CustomStart {
    pub spec: CustomCaptainSpec,
    pub seed: i128,
    pub starting_port: Option<String>,
}

/// `validate_spec` (line 96). Empty means valid. Messages and order match Python.
pub fn validate_spec(spec: &CustomCaptainSpec) -> Vec<String> {
    let mut errors = Vec::new();
    let total = i128::from(spec.trade_points)
        + i128::from(spec.sailing_points)
        + i128::from(spec.shadow_points)
        + i128::from(spec.reputation_points);
    if total != i128::from(TOTAL_SKILL_POINTS) {
        errors.push(format!(
            "Must allocate exactly {TOTAL_SKILL_POINTS} points (allocated {total})"
        ));
    }
    for (cat_name, pts) in [
        ("Trade", spec.trade_points),
        ("Sailing", spec.sailing_points),
        ("Shadow", spec.shadow_points),
        ("Reputation", spec.reputation_points),
    ] {
        if pts < 0 {
            errors.push(format!("{cat_name} points cannot be negative"));
        }
        if pts > MAX_POINTS_PER_CATEGORY {
            errors.push(format!(
                "{cat_name} points cannot exceed {MAX_POINTS_PER_CATEGORY} (got {pts})"
            ));
        }
    }

    let catalog = content::content();
    if let Some(port) = catalog.port(&spec.home_port_id) {
        if port.region != spec.home_region {
            errors.push(format!(
                "Port '{}' is in {}, not {}",
                spec.home_port_id, port.region, spec.home_region
            ));
        }
    } else {
        errors.push(format!("Unknown home port: '{}'", spec.home_port_id));
    }

    const REGIONS: [&str; 5] = [
        "Mediterranean",
        "North Atlantic",
        "West Africa",
        "East Indies",
        "South Seas",
    ];
    if !REGIONS.contains(&spec.home_region.as_str()) {
        errors.push(format!("Unknown region: '{}'", spec.home_region));
    }

    if !spec.bloc_alignment.is_empty() && catalog.trade_bloc(&spec.bloc_alignment).is_none() {
        errors.push(format!("Unknown trade bloc: '{}'", spec.bloc_alignment));
    }
    if !spec.faction_alignment.is_empty() && catalog.faction(&spec.faction_alignment).is_none() {
        errors.push(format!(
            "Unknown pirate faction: '{}'",
            spec.faction_alignment
        ));
    }
    if !spec.mentor_npc_id.is_empty() {
        match catalog.npc(&spec.mentor_npc_id) {
            None => errors.push(format!("Unknown mentor NPC: '{}'", spec.mentor_npc_id)),
            Some(mentor) if mentor.port_id != spec.home_port_id => errors.push(format!(
                "Mentor '{}' is at {}, not at your home port {}",
                spec.mentor_npc_id, mentor.port_id, spec.home_port_id
            )),
            Some(_) => {}
        }
    }
    errors
}

/// `build_custom_template` (line 153). Assumes a spec `validate_spec` accepted.
pub fn build_custom_template(spec: &CustomCaptainSpec) -> CustomCaptainTemplate {
    let pricing = PricingDef {
        buy_price_mult: py_round_places(1.0 + TRADE_BUY * spec.trade_points as f64, 3),
        sell_price_mult: py_round_places(1.0 + TRADE_SELL * spec.trade_points as f64, 3),
        luxury_sell_bonus: py_round_places(TRADE_LUXURY * spec.trade_points as f64, 3),
        port_fee_mult: py_round_places((1.0 + TRADE_FEE * spec.trade_points as f64).max(0.5), 3),
    };
    let voyage = VoyageModsDef {
        provision_burn: py_round_places((1.0 + SAIL_BURN * spec.sailing_points as f64).max(0.5), 3),
        speed_bonus: py_round_places(SAIL_SPEED * spec.sailing_points as f64, 2),
        storm_resist_bonus: py_round_places(SAIL_STORM * spec.sailing_points as f64, 3),
        cargo_damage_mult: py_round_places(
            (1.0 + SAIL_CARGO * spec.sailing_points as f64).max(0.5),
            3,
        ),
    };
    let inspection = InspectionDef {
        inspection_chance_mult: py_round_places(
            (1.0 + SHADOW_INSPECTION * spec.shadow_points as f64).max(0.3),
            3,
        ),
        seizure_risk: 0.0,
        fine_mult: py_round_places((1.0 - 0.06 * spec.shadow_points as f64).max(0.4), 3),
    };

    let trust = REPUTATION_TRUST * spec.reputation_points;
    let standing = REPUTATION_STANDING * spec.reputation_points;
    let mut underworld = BTreeMap::new();
    if spec.shadow_points > 0 && !spec.faction_alignment.is_empty() {
        underworld.insert(
            spec.faction_alignment.clone(),
            SHADOW_UNDERWORLD * spec.shadow_points,
        );
    }
    let region_value = |region: &str| {
        if spec.home_region == region {
            standing
        } else {
            0
        }
    };
    let reputation = ReputationDef {
        commercial_trust: trust,
        customs_heat: 0,
        mediterranean: region_value("Mediterranean"),
        north_atlantic: region_value("North Atlantic"),
        west_africa: region_value("West Africa"),
        east_indies: region_value("East Indies"),
        south_seas: region_value("South Seas"),
        underworld: underworld.clone(),
    };

    let mut strengths = Vec::new();
    let mut weaknesses = Vec::new();
    if spec.trade_points >= 4 {
        strengths.push(format!(
            "Strong trader ({} points in Trade)",
            spec.trade_points
        ));
    } else if spec.trade_points == 0 {
        weaknesses.push("No trade advantages".to_string());
    }
    if spec.sailing_points >= 4 {
        strengths.push(format!(
            "Expert sailor ({} points in Sailing)",
            spec.sailing_points
        ));
    } else if spec.sailing_points == 0 {
        weaknesses.push("No sailing advantages".to_string());
    }
    if spec.shadow_points >= 4 {
        strengths.push(format!(
            "Shadow operative ({} points in Shadow)",
            spec.shadow_points
        ));
    } else if spec.shadow_points == 0 {
        weaknesses.push("No underworld connections".to_string());
    }
    if spec.reputation_points >= 4 {
        strengths.push(format!(
            "Well-connected ({} points in Reputation)",
            spec.reputation_points
        ));
    } else if spec.reputation_points == 0 {
        weaknesses.push("No starting reputation".to_string());
    }
    if strengths.is_empty() {
        strengths.push("Balanced \u{2014} jack of all trades".to_string());
    }
    if weaknesses.is_empty() {
        weaknesses.push("No clear specialty \u{2014} master of none".to_string());
    }

    let backstory = if spec.backstory.is_empty() {
        format!(
            "A captain from {} who chose their own path. The world doesn't know your name yet. That's about to change.",
            spec.home_region
        )
    } else {
        spec.backstory.clone()
    };
    let faction_alignment = if !spec.faction_alignment.is_empty() && !underworld.is_empty() {
        underworld.clone()
    } else {
        BTreeMap::new()
    };

    CustomCaptainTemplate {
        id: "custom".to_string(),
        name: spec.name.clone(),
        title: spec.title.clone(),
        description: backstory.clone(),
        home_region: spec.home_region.clone(),
        home_port_id: spec.home_port_id.clone(),
        starting_silver: region_starting_silver(&spec.home_region),
        starting_ship_id: "coastal_sloop".to_string(),
        starting_provisions: 30,
        pricing,
        voyage,
        inspection,
        reputation,
        strengths,
        weaknesses,
        backstory,
        mentor_npc_id: spec.mentor_npc_id.clone(),
        bloc_alignment: spec.bloc_alignment.clone(),
        faction_alignment,
    }
}

/// Catalog captain, or the custom template stored on the world.
///
/// Python `voyage._get_captain_mods` returns `None` when `custom` was never
/// registered. A missing template does the same. It does not fall back to
/// the merchant archetype. Buy, sell, and reprice use [`captain_template`].
pub fn active_captain(world: &World) -> Option<&CaptainDef> {
    if world.captain.captain_type == "custom" {
        return world.custom_captain.as_ref();
    }
    content::content().captain(&world.captain.captain_type)
}

/// `GameSession.captain_template` (`app/session.py` lines 488–496).
///
/// The registered custom template when `world.custom_captain` is set. A
/// `custom` captain whose template was not restored (a fresh load: saves do
/// not store it, so `CAPTAIN_TEMPLATES` raises `KeyError`) and any unknown
/// type both return the merchant archetype.
pub fn captain_template(world: &World) -> &CaptainDef {
    if world.captain.captain_type == "custom" {
        if let Some(custom) = world.custom_captain.as_ref() {
            return custom;
        }
    } else if let Some(catalogued) = content::content().captain(&world.captain.captain_type) {
        return catalogued;
    }
    content::content()
        .captain("merchant")
        .expect("merchant archetype")
}

/// Parse a `custom` script line. `tokens[0]` is the command word.
pub fn parse_script_spec(tokens: &[String]) -> Result<CustomStart, SimError> {
    if tokens.len() < 3 {
        return Err(SimError::Sentence(CUSTOM_USAGE.to_string()));
    }
    let seed: i128 = tokens[2]
        .parse()
        .map_err(|_| SimError::InvalidNumber(tokens[2].clone()))?;
    let mut spec = CustomCaptainSpec {
        name: tokens[1].clone(),
        ..CustomCaptainSpec::default()
    };
    let mut starting_port = None;
    for token in &tokens[3..] {
        let Some((key, value)) = token.split_once('=') else {
            return Err(SimError::Sentence(format!("Unknown custom field: {token}")));
        };
        match key {
            "trade" => spec.trade_points = parse_points(value)?,
            "sailing" => spec.sailing_points = parse_points(value)?,
            "shadow" => spec.shadow_points = parse_points(value)?,
            "reputation" => spec.reputation_points = parse_points(value)?,
            "home" => spec.home_port_id = value.to_string(),
            "region" => spec.home_region = value.to_string(),
            "title" => spec.title = value.to_string(),
            "bloc" => spec.bloc_alignment = value.to_string(),
            "faction" => spec.faction_alignment = value.to_string(),
            "mentor" => spec.mentor_npc_id = value.to_string(),
            "backstory" => spec.backstory = value.to_string(),
            "port" => {
                starting_port = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                };
            }
            _ => {
                return Err(SimError::Sentence(format!("Unknown custom field: {key}")));
            }
        }
    }
    Ok(CustomStart {
        spec,
        seed,
        starting_port,
    })
}

fn parse_points(value: &str) -> Result<i64, SimError> {
    value
        .parse()
        .map_err(|_| SimError::InvalidNumber(value.to_string()))
}

fn region_starting_silver(region: &str) -> i64 {
    match region {
        "Mediterranean" => 500,
        "North Atlantic" | "West Africa" => 450,
        "East Indies" => 400,
        "South Seas" => 350,
        _ => 400,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn points(trade: i64, sailing: i64, shadow: i64, reputation: i64) -> CustomCaptainSpec {
        CustomCaptainSpec {
            home_port_id: "porto_novo".to_string(),
            home_region: "Mediterranean".to_string(),
            trade_points: trade,
            sailing_points: sailing,
            shadow_points: shadow,
            reputation_points: reputation,
            ..CustomCaptainSpec::default()
        }
    }

    #[test]
    fn valid_spec_passes() {
        assert!(validate_spec(&points(3, 3, 2, 2)).is_empty());
    }

    #[test]
    fn validation_messages_match_python_order() {
        assert_eq!(
            validate_spec(&points(5, 5, 5, 5)),
            vec!["Must allocate exactly 10 points (allocated 20)".to_string()]
        );
        assert_eq!(
            validate_spec(&points(-1, 5, 3, 3)),
            vec!["Trade points cannot be negative".to_string()]
        );
        assert_eq!(
            validate_spec(&points(8, 1, 1, 0)),
            vec!["Trade points cannot exceed 7 (got 8)".to_string()]
        );
        let mut spec = points(3, 3, 2, 2);
        spec.home_port_id = "atlantis".to_string();
        assert_eq!(
            validate_spec(&spec),
            vec!["Unknown home port: 'atlantis'".to_string()]
        );
        spec = points(3, 3, 2, 2);
        spec.home_region = "East Indies".to_string();
        assert_eq!(
            validate_spec(&spec),
            vec!["Port 'porto_novo' is in Mediterranean, not East Indies".to_string()]
        );
        spec.home_region = "Narnia".to_string();
        assert_eq!(
            validate_spec(&spec),
            vec![
                "Port 'porto_novo' is in Mediterranean, not Narnia".to_string(),
                "Unknown region: 'Narnia'".to_string(),
            ]
        );
        spec = points(3, 3, 2, 2);
        spec.bloc_alignment = "fake_bloc".to_string();
        assert_eq!(
            validate_spec(&spec),
            vec!["Unknown trade bloc: 'fake_bloc'".to_string()]
        );
        spec.bloc_alignment.clear();
        spec.faction_alignment = "fake_faction".to_string();
        assert_eq!(
            validate_spec(&spec),
            vec!["Unknown pirate faction: 'fake_faction'".to_string()]
        );
        spec.faction_alignment.clear();
        spec.mentor_npc_id = "am_yasmin".to_string();
        assert_eq!(
            validate_spec(&spec),
            vec!["Mentor 'am_yasmin' is at al_manar, not at your home port porto_novo".to_string()]
        );
        spec.mentor_npc_id = "no_such_npc".to_string();
        assert_eq!(
            validate_spec(&spec),
            vec!["Unknown mentor NPC: 'no_such_npc'".to_string()]
        );
        spec.mentor_npc_id = "pn_marta".to_string();
        spec.bloc_alignment = "exchange_alliance".to_string();
        assert!(validate_spec(&spec).is_empty());
        spec.mentor_npc_id.clear();
        spec.bloc_alignment.clear();
        spec.faction_alignment = "crimson_tide".to_string();
        spec.home_port_id = "corsairs_rest".to_string();
        assert!(validate_spec(&spec).is_empty());
    }

    #[test]
    fn each_category_refusal_names_that_category() {
        assert_eq!(
            validate_spec(&points(4, -1, 4, 3)),
            vec!["Sailing points cannot be negative".to_string()]
        );
        assert_eq!(
            validate_spec(&points(4, 4, -1, 3)),
            vec!["Shadow points cannot be negative".to_string()]
        );
        assert_eq!(
            validate_spec(&points(4, 4, 3, -1)),
            vec!["Reputation points cannot be negative".to_string()]
        );
        assert_eq!(
            validate_spec(&points(1, 8, 1, 0)),
            vec!["Sailing points cannot exceed 7 (got 8)".to_string()]
        );
        assert_eq!(
            validate_spec(&points(1, 1, 8, 0)),
            vec!["Shadow points cannot exceed 7 (got 8)".to_string()]
        );
        assert_eq!(
            validate_spec(&points(1, 1, 0, 8)),
            vec!["Reputation points cannot exceed 7 (got 8)".to_string()]
        );
    }

    #[test]
    fn template_matches_python_examples() {
        let play = CustomCaptainSpec {
            name: "Mara Voss".to_string(),
            title: "Freelance Captain".to_string(),
            home_port_id: "porto_novo".to_string(),
            home_region: "Mediterranean".to_string(),
            trade_points: 4,
            sailing_points: 3,
            shadow_points: 2,
            reputation_points: 1,
            bloc_alignment: "exchange_alliance".to_string(),
            faction_alignment: "crimson_tide".to_string(),
            mentor_npc_id: "pn_marta".to_string(),
            backstory: "Born on the quay.".to_string(),
        };
        let template = build_custom_template(&play);
        assert_eq!(template.id, "custom");
        assert_eq!(template.starting_silver, 500);
        assert_eq!(template.starting_ship_id, "coastal_sloop");
        assert_eq!(template.starting_provisions, 30);
        assert_eq!(template.pricing.buy_price_mult, 0.96);
        assert_eq!(template.pricing.sell_price_mult, 1.04);
        assert_eq!(template.pricing.luxury_sell_bonus, 0.12);
        assert_eq!(template.pricing.port_fee_mult, 0.88);
        assert_eq!(template.voyage.provision_burn, 0.91);
        assert_eq!(template.voyage.speed_bonus, 0.9);
        assert_eq!(template.voyage.storm_resist_bonus, 0.045);
        assert_eq!(template.voyage.cargo_damage_mult, 0.91);
        assert_eq!(template.inspection.inspection_chance_mult, 0.88);
        assert_eq!(template.inspection.seizure_risk, 0.0);
        assert_eq!(template.inspection.fine_mult, 0.88);
        assert_eq!(template.reputation.commercial_trust, 2);
        assert_eq!(template.reputation.mediterranean, 2);
        assert_eq!(
            template.reputation.underworld.get("crimson_tide").copied(),
            Some(10)
        );
        assert_eq!(
            template.strengths,
            vec!["Strong trader (4 points in Trade)".to_string()]
        );
        assert_eq!(
            template.weaknesses,
            vec!["No clear specialty \u{2014} master of none".to_string()]
        );
        assert_eq!(template.backstory, "Born on the quay.");
        assert_eq!(template.bloc_alignment, "exchange_alliance");
        assert_eq!(template.mentor_npc_id, "pn_marta");

        let east = CustomCaptainSpec {
            home_port_id: "jade_port".to_string(),
            home_region: "East Indies".to_string(),
            trade_points: 1,
            sailing_points: 1,
            shadow_points: 1,
            reputation_points: 7,
            ..CustomCaptainSpec::default()
        };
        let east = build_custom_template(&east);
        assert_eq!(east.starting_silver, 400);
        assert_eq!(east.reputation.commercial_trust, 14);
        assert_eq!(east.reputation.east_indies, 14);
        assert_eq!(east.reputation.mediterranean, 0);
        assert!(east.backstory.contains("East Indies"));
        assert_eq!(
            east.strengths,
            vec!["Well-connected (7 points in Reputation)".to_string()]
        );

        let shadow = CustomCaptainSpec {
            home_port_id: "corsairs_rest".to_string(),
            home_region: "Mediterranean".to_string(),
            trade_points: 1,
            sailing_points: 1,
            shadow_points: 7,
            reputation_points: 1,
            faction_alignment: "crimson_tide".to_string(),
            ..CustomCaptainSpec::default()
        };
        let shadow = build_custom_template(&shadow);
        assert_eq!(shadow.inspection.inspection_chance_mult, 0.58);
        assert_eq!(shadow.inspection.fine_mult, 0.58);
        assert_eq!(shadow.voyage.speed_bonus, 0.3);
        assert_eq!(
            shadow.faction_alignment.get("crimson_tide").copied(),
            Some(35)
        );
        let sailing = CustomCaptainSpec {
            home_port_id: "silva_bay".to_string(),
            home_region: "Mediterranean".to_string(),
            trade_points: 1,
            sailing_points: 7,
            shadow_points: 1,
            reputation_points: 1,
            ..CustomCaptainSpec::default()
        };
        let sailing = build_custom_template(&sailing);
        assert_eq!(sailing.voyage.speed_bonus, 2.1);
        assert_eq!(sailing.voyage.provision_burn, 0.79);
        assert_eq!(sailing.voyage.storm_resist_bonus, 0.105);

        let no_faction = points(3, 0, 7, 0);
        let no_faction = build_custom_template(&no_faction);
        assert!(no_faction.reputation.underworld.is_empty());
        assert!(no_faction.faction_alignment.is_empty());
        assert_eq!(
            no_faction.weaknesses,
            vec![
                "No sailing advantages".to_string(),
                "No starting reputation".to_string(),
            ]
        );

        let balanced = points(2, 3, 2, 3);
        let balanced = build_custom_template(&balanced);
        assert_eq!(
            balanced.strengths,
            vec!["Balanced \u{2014} jack of all trades".to_string()]
        );
        assert_eq!(
            balanced.weaknesses,
            vec!["No clear specialty \u{2014} master of none".to_string()]
        );
    }

    #[test]
    fn new_custom_installs_the_template_on_the_session() {
        let spec = CustomCaptainSpec {
            name: "Mara Voss".to_string(),
            title: "Freelance Captain".to_string(),
            home_port_id: "porto_novo".to_string(),
            home_region: "Mediterranean".to_string(),
            trade_points: 4,
            sailing_points: 3,
            shadow_points: 2,
            reputation_points: 1,
            bloc_alignment: "exchange_alliance".to_string(),
            faction_alignment: "crimson_tide".to_string(),
            mentor_npc_id: "pn_marta".to_string(),
            backstory: "Born on the quay.".to_string(),
        };
        let session = crate::session::Session::new_custom(&spec, 42, None).expect("game");
        let world = session.world();
        assert_eq!(world.captain.captain_type, "custom");
        assert_eq!(world.captain.name, "Mara Voss");
        assert_eq!(world.captain.silver, 500);
        assert_eq!(world.captain.provisions, 30);
        assert_eq!(world.voyage.destination_id, "porto_novo");
        assert_eq!(world.captain.standing.commercial_trust, 2);
        assert_eq!(world.captain.standing.regional_of("Mediterranean"), 2);
        assert_eq!(
            world
                .captain
                .standing
                .underworld
                .iter()
                .find(|(id, _)| id == "crimson_tide")
                .map(|(_, value)| *value),
            Some(10)
        );
        assert!(world.custom_captain.is_some());
        let err = crate::session::Session::new_custom(&points(-1, 5, 3, 3), 1, None)
            .expect_err("refused");
        assert_eq!(err.to_string(), "Trade points cannot be negative");
        let mut narnia = points(3, 3, 2, 2);
        narnia.home_region = "Narnia".to_string();
        let err = crate::session::Session::new_custom(&narnia, 1, None).expect_err("region");
        assert_eq!(
            err.to_string(),
            "Port 'porto_novo' is in Mediterranean, not Narnia\nUnknown region: 'Narnia'"
        );
    }

    #[test]
    fn south_seas_silver_is_lower_than_the_mediterranean() {
        let med = build_custom_template(&points(3, 3, 2, 2));
        let mut south = points(3, 3, 2, 2);
        south.home_port_id = "coral_throne".to_string();
        south.home_region = "South Seas".to_string();
        let south = build_custom_template(&south);
        assert!(med.starting_silver > south.starting_silver);
        assert_eq!(south.starting_silver, 350);
        assert_eq!(south.reputation.south_seas, 4);
    }
}
