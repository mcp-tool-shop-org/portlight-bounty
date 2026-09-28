//! Voyage state machine: depart, one day at sea, arrival.
//!
//! Event text and RNG call order follow `portlight.engine.voyage`. A pending
//! pirate duel blocks further days until `Session::duel` or
//! `Session::resolve_pending_duel` clears it. `advance` does not auto-resolve.

use crate::content::{self, class_rank, ship_class_rank, CaptainDef};
use crate::economy::{cargo_quantity, consume_cargo_fifo};
use crate::error::SimError;
use crate::fleet;
use crate::model::{PendingDuel, Voyage, VoyageStatus, World};
use crate::pyrand::PyRandom;
use crate::reputation::inspection_modifier;
use crate::ship::{
    apply_crew_delta, has_special, morale_speed_modifier, navigator_speed_bonus, resolve_speed,
    resolve_storm_resist, template_crew_min, tick_morale_at_port, tick_morale_at_sea, wage_bill,
};
use crate::util::{py_round, py_trunc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Storm,
    Pirates,
    Inspection,
    CalmSeas,
    FavorableWind,
    ProvisionsSpoiled,
    CargoDamaged,
    MerchantEncounter,
    Flotsam,
    Nothing,
    ForeignVessel,
    CulturalWaters,
    SeaCeremony,
    WhaleSighting,
    Lighthouse,
    MusicianAboard,
    DriftingOffering,
    StarNavigation,
}

impl EventType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Storm => "storm",
            Self::Pirates => "pirates",
            Self::Inspection => "inspection",
            Self::CalmSeas => "calm_seas",
            Self::FavorableWind => "favorable_wind",
            Self::ProvisionsSpoiled => "provisions_spoiled",
            Self::CargoDamaged => "cargo_damaged",
            Self::MerchantEncounter => "merchant_encounter",
            Self::Flotsam => "flotsam",
            Self::Nothing => "nothing",
            Self::ForeignVessel => "foreign_vessel",
            Self::CulturalWaters => "cultural_waters",
            Self::SeaCeremony => "sea_ceremony",
            Self::WhaleSighting => "whale_sighting",
            Self::Lighthouse => "lighthouse",
            Self::MusicianAboard => "musician_aboard",
            Self::DriftingOffering => "drifting_offering",
            Self::StarNavigation => "star_navigation",
        }
    }
}

#[derive(Debug, Clone)]
pub struct VoyageEvent {
    pub event_type: EventType,
    pub message: String,
    pub hull_delta: i64,
    pub provision_delta: i64,
    pub silver_delta: i64,
    pub crew_delta: i64,
    pub speed_modifier: f64,
    pub cargo_lost: Vec<(String, i64)>,
    pub flavor: String,
    pub pending_duel: Option<PendingDuel>,
}

impl VoyageEvent {
    pub(crate) fn annotated(
        event_type: EventType,
        message: impl Into<String>,
        flavor: impl Into<String>,
    ) -> Self {
        let mut event = Self::new(event_type, message);
        event.flavor = flavor.into();
        event
    }

    fn new(event_type: EventType, message: impl Into<String>) -> Self {
        Self {
            event_type,
            message: message.into(),
            hull_delta: 0,
            provision_delta: 0,
            silver_delta: 0,
            crew_delta: 0,
            speed_modifier: 1.0,
            cargo_lost: Vec::new(),
            flavor: String::new(),
            pending_duel: None,
        }
    }
}

const EVENT_WEIGHTS: [(EventType, f64); 18] = [
    (EventType::Nothing, 0.23),
    (EventType::CalmSeas, 0.10),
    (EventType::FavorableWind, 0.11),
    (EventType::Storm, 0.10),
    (EventType::Pirates, 0.12),
    (EventType::Inspection, 0.05),
    (EventType::ProvisionsSpoiled, 0.04),
    (EventType::CargoDamaged, 0.03),
    (EventType::MerchantEncounter, 0.06),
    (EventType::Flotsam, 0.05),
    (EventType::ForeignVessel, 0.015),
    (EventType::CulturalWaters, 0.015),
    (EventType::SeaCeremony, 0.01),
    (EventType::WhaleSighting, 0.015),
    (EventType::Lighthouse, 0.015),
    (EventType::MusicianAboard, 0.015),
    (EventType::DriftingOffering, 0.01),
    (EventType::StarNavigation, 0.015),
];

fn captain_mods(world: &World) -> Option<&CaptainDef> {
    crate::custom_captain::active_captain(world)
}

fn pick_event(
    danger: f64,
    rng: &mut PyRandom,
    inspection_mult: f64,
    recent: &[String],
) -> EventType {
    let mut weights = Vec::with_capacity(EVENT_WEIGHTS.len());
    for (etype, base) in EVENT_WEIGHTS {
        let mut w = base;
        if matches!(
            etype,
            EventType::Storm | EventType::Pirates | EventType::CargoDamaged
        ) {
            w *= 1.0 + danger * 2.0;
        } else if matches!(etype, EventType::MerchantEncounter | EventType::Flotsam) {
            w *= (1.0 - danger).max(0.5);
        } else if etype == EventType::Inspection {
            w *= inspection_mult;
        }
        if recent.iter().any(|r| r == etype.as_str()) {
            w *= 0.2;
        }
        weights.push(w);
    }
    EVENT_WEIGHTS[rng.choices_weighted(&weights)].0
}

fn resolve_event(
    event_type: EventType,
    rng: &mut PyRandom,
    world: &World,
) -> Result<VoyageEvent, SimError> {
    let Some(ship) = world.captain.ship.as_ref() else {
        return Err(SimError::NoShip);
    };
    let mut storm_resist = resolve_storm_resist(ship);
    let mods = captain_mods(world);
    let (cargo_dmg_mult, fine_mult, seizure_risk) = if let Some(mods) = mods {
        storm_resist = 0.9_f64.min(storm_resist + mods.voyage.storm_resist_bonus);
        (
            mods.voyage.cargo_damage_mult,
            mods.inspection.fine_mult,
            mods.inspection.seizure_risk,
        )
    } else {
        (1.0, 1.0, 0.0)
    };

    Ok(match event_type {
        EventType::Storm => {
            let raw = rng.randint(5, 18);
            let dmg = 1.max(py_trunc(raw as f64 * (1.0 - storm_resist)));
            let msg = if storm_resist > 0.3 {
                format!(
                    "A storm batters the ship. Your hull absorbs the worst of it. (-{dmg} hull)"
                )
            } else {
                format!("A violent storm batters the ship! (-{dmg} hull)")
            };
            let mut ev = VoyageEvent::new(EventType::Storm, msg);
            ev.hull_delta = -dmg;
            ev.speed_modifier = 0.5;
            ev
        }
        EventType::Pirates => resolve_pirates(rng, world, storm_resist),
        EventType::Inspection => resolve_inspection(rng, world, fine_mult, seizure_risk),
        EventType::FavorableWind => VoyageEvent::new(
            EventType::FavorableWind,
            "Strong tailwinds speed your journey!",
        )
        .with_speed(1.5),
        EventType::ProvisionsSpoiled => {
            let spoil = rng.randint(2, 6);
            let mut ev = VoyageEvent::new(
                EventType::ProvisionsSpoiled,
                format!("Some provisions have spoiled. (-{spoil} days)"),
            );
            ev.provision_delta = -spoil;
            ev
        }
        EventType::CalmSeas => VoyageEvent::new(
            EventType::CalmSeas,
            "Calm seas. Good for rest, bad for progress.",
        )
        .with_speed(0.6),
        EventType::CargoDamaged => {
            if world.captain.cargo.is_empty() {
                return Ok(VoyageEvent::new(
                    EventType::Nothing,
                    "An uneventful day at sea.",
                ));
            }
            let idx = rng.choice_index(world.captain.cargo.len());
            let good_id = world.captain.cargo[idx].good_id.clone();
            let raw_lost = rng.randint(1, 3);
            let mut lost = 1.max(py_trunc(raw_lost as f64 * cargo_dmg_mult));
            lost = cargo_quantity(&world.captain.cargo, &good_id).min(lost);
            let mut ev = VoyageEvent::new(
                EventType::CargoDamaged,
                format!("Rough seas damaged {lost} units of {good_id} in the hold."),
            );
            ev.cargo_lost.push((good_id, lost));
            ev
        }
        EventType::MerchantEncounter => {
            let gain = rng.randint(5, 20);
            let mut ev = VoyageEvent::new(
                EventType::MerchantEncounter,
                format!("A passing merchant offers information and a small gift. (+{gain} silver)"),
            );
            ev.silver_delta = gain;
            ev
        }
        EventType::Flotsam => {
            let prov = rng.randint(1, 4);
            let mut ev = VoyageEvent::new(
                EventType::Flotsam,
                format!("Floating wreckage yields salvageable supplies. (+{prov} provisions)"),
            );
            ev.provision_delta = prov;
            ev
        }
        EventType::ForeignVessel => resolve_foreign_vessel(rng),
        EventType::CulturalWaters => resolve_cultural_waters(rng),
        EventType::SeaCeremony => {
            let mut ev = VoyageEvent::new(
                EventType::SeaCeremony,
                "The crew gathers at dusk. The bosun pours rum into the sea — an old offering for safe passage.",
            );
            ev.provision_delta = -1;
            ev.flavor = "Some rituals are older than the ships that carry them.".to_string();
            ev
        }
        EventType::WhaleSighting => {
            let mut ev = VoyageEvent::new(
                EventType::WhaleSighting,
                "A pod of whales surfaces alongside the ship. The crew watches in silence.",
            );
            ev.flavor = "Some things are bigger than commerce.".to_string();
            ev
        }
        EventType::Lighthouse => resolve_lighthouse(rng),
        EventType::MusicianAboard => resolve_musician(rng),
        EventType::DriftingOffering => resolve_offering(rng),
        EventType::StarNavigation => {
            let is_navigator = world.captain.captain_type == "navigator";
            let (speed, msg) = if is_navigator {
                (
                    1.2,
                    "You read the stars yourself and correct course. The old constellations guide you true — no one reads them better.",
                )
            } else {
                (
                    1.05,
                    "The navigator reads the stars and adjusts course. Ancient constellations confirm your heading.",
                )
            };
            let mut ev = VoyageEvent::new(EventType::StarNavigation, msg).with_speed(speed);
            ev.flavor = "The sky is the oldest chart.".to_string();
            ev
        }
        EventType::Nothing => VoyageEvent::new(EventType::Nothing, "An uneventful day at sea."),
    })
}

impl VoyageEvent {
    fn with_speed(mut self, speed: f64) -> Self {
        self.speed_modifier = speed;
        self
    }
}

fn resolve_pirates(rng: &mut PyRandom, world: &World, storm_resist: f64) -> VoyageEvent {
    let duel_roll = rng.random();
    if duel_roll < 0.40 {
        let region = world
            .port(&world.voyage.destination_id)
            .map(|p| p.region.as_str())
            .unwrap_or("Mediterranean");
        let catalog = content::content();
        let live_ids: Vec<&str> = world
            .captain
            .active_bounties
            .iter()
            .filter(|id| catalog.pirate(id).is_some())
            .map(|s| s.as_str())
            .collect();
        let regional_ids: Vec<&str> = live_ids
            .iter()
            .copied()
            .filter(|cid| {
                let Some(pc) = catalog.pirate(cid) else {
                    return false;
                };
                catalog
                    .factions
                    .iter()
                    .find(|f| f.id == pc.faction_id)
                    .is_some_and(|f| f.territory_regions.iter().any(|r| r == region))
            })
            .collect();
        let preferred: Vec<&str> = if regional_ids.is_empty() {
            live_ids.clone()
        } else {
            regional_ids
        };
        let picked = if !preferred.is_empty() {
            let id = preferred[rng.choice_index(preferred.len())];
            catalog.pirate(id)
        } else {
            None
        };
        let (pirate_id, faction_id, faction_name, personality, strength, pirate_name) =
            if let Some(pc) = picked {
                let faction = catalog.factions.iter().find(|f| f.id == pc.faction_id);
                (
                    pc.id.clone(),
                    pc.faction_id.clone(),
                    faction
                        .map(|f| f.name.clone())
                        .unwrap_or_else(|| pc.faction_id.clone()),
                    pc.personality.clone(),
                    pc.strength,
                    pc.name.clone(),
                )
            } else {
                let mut active = catalog.factions_in(region);
                if active.is_empty() {
                    active = catalog.factions.iter().take(1).collect();
                }
                let faction = active[rng.choice_index(active.len())];
                let captains = catalog.captains_in_faction(&faction.id);
                if captains.is_empty() {
                    (
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        0,
                        String::new(),
                    )
                } else {
                    let pc = captains[rng.choice_index(captains.len())];
                    (
                        pc.id.clone(),
                        faction.id.clone(),
                        faction.name.clone(),
                        pc.personality.clone(),
                        pc.strength,
                        pc.name.clone(),
                    )
                }
            };
        if !pirate_id.is_empty() {
            let mut ev = VoyageEvent::new(
                EventType::Pirates,
                format!(
                    "{pirate_name} of the {faction_name} blocks your path and demands a duel! Use [bold]portlight duel <stance>[/bold] to fight (thrust/slash/parry, 5 rounds)."
                ),
            );
            ev.flavor = format!("Strength {strength}, {personality} fighter.");
            ev.pending_duel = Some(PendingDuel {
                captain_id: pirate_id,
                captain_name: pirate_name,
                faction_id,
                personality,
                strength,
                region: region.to_string(),
            });
            return ev;
        }
    }
    let cargo_value: i64 = world.captain.cargo.iter().map(|c| c.quantity * 10).sum();
    let base_loss = rng.randint(10, 40);
    let silver_loss = (base_loss + cargo_value / 10).min(world.captain.silver);
    let mut dmg = rng.randint(3, 12);
    dmg = 1.max(py_trunc(dmg as f64 * (1.0 - storm_resist * 0.5)));
    let mut ev = VoyageEvent::new(
        EventType::Pirates,
        format!("Pirates attack! You fight them off but lose {silver_loss} silver. (-{dmg} hull)"),
    );
    ev.hull_delta = -dmg;
    ev.silver_delta = -silver_loss;
    ev
}

fn resolve_inspection(
    rng: &mut PyRandom,
    world: &World,
    mut fine_mult: f64,
    seizure_risk: f64,
) -> VoyageEvent {
    let mut fee = rng.randint(5, 25);
    let max_heat = world
        .captain
        .standing
        .heat
        .iter()
        .copied()
        .max()
        .unwrap_or(0);
    if max_heat >= 30 {
        fine_mult *= 1.5;
    } else if max_heat >= 15 {
        fine_mult *= 1.2;
    }
    fee = 1.max(py_trunc(fee as f64 * fine_mult));

    let contraband: Vec<(String, i64)> = {
        let mut acc: Vec<(String, i64)> = Vec::new();
        for item in &world.captain.cargo {
            if crate::economy::is_contraband(&item.good_id) {
                if let Some(slot) = acc.iter_mut().find(|(id, _)| id == &item.good_id) {
                    slot.1 += item.quantity;
                } else {
                    acc.push((item.good_id.clone(), item.quantity));
                }
            }
        }
        acc
    };
    if !contraband.is_empty() {
        let mut detect = 0.40 + (max_heat - 15).max(0) as f64 * 0.10;
        if world.captain.captain_type == "smuggler" {
            detect -= 0.15;
        }
        detect = detect.clamp(0.1, 0.9);
        if rng.random() < detect {
            let mut contraband_value = 0;
            for (_, qty) in &contraband {
                contraband_value += qty * 30;
            }
            let fine = contraband_value * 3;
            fee += fine;
            let mut ev = VoyageEvent::new(
                EventType::Inspection,
                format!(
                    "A patrol boards your ship. CONTRABAND FOUND! The inspectors seize everything illegal in your hold. Fine: {fine} silver. Your reputation takes a devastating hit."
                ),
            );
            ev.silver_delta = -fee;
            ev.cargo_lost = contraband;
            ev.flavor =
                "The silence after a contraband seizure is the loudest sound at sea.".to_string();
            return ev;
        }
    }

    let mut seized: Vec<(String, i64)> = Vec::new();
    let mut seizure_msg = String::new();
    if seizure_risk > 0.0 && !world.captain.cargo.is_empty() && rng.random() < seizure_risk {
        let legal: Vec<usize> = world
            .captain
            .cargo
            .iter()
            .enumerate()
            .filter(|(_, c)| !crate::economy::is_contraband(&c.good_id))
            .map(|(i, _)| i)
            .collect();
        if !legal.is_empty() {
            let target = &world.captain.cargo[legal[rng.choice_index(legal.len())]];
            let good_id = target.good_id.clone();
            let seized_qty = cargo_quantity(&world.captain.cargo, &good_id).min(rng.randint(1, 3));
            seizure_msg = format!(" They confiscate {seized_qty} units of {good_id}!");
            seized.push((good_id, seized_qty));
        }
    }
    let actual_fee = fee.min(world.captain.silver);
    let msg = if actual_fee < fee {
        format!(
            "A patrol inspects your cargo and levies a {fee} silver fee (only {actual_fee} collected).{seizure_msg}"
        )
    } else {
        format!("A patrol inspects your cargo and levies a {fee} silver fee.{seizure_msg}")
    };
    let mut ev = VoyageEvent::new(EventType::Inspection, msg);
    ev.silver_delta = -actual_fee;
    ev.cargo_lost = seized;
    ev
}

fn resolve_foreign_vessel(rng: &mut PyRandom) -> VoyageEvent {
    const FLAVOR: &[(&str, &[&str])] = &[
        (
            "Mediterranean",
            &[
                "A merchant galley with striped sails crosses your bow, its deck stacked with amphoras.",
                "A felucca glides past, its triangular sail catching the coastal wind. The crew waves.",
            ],
        ),
        (
            "North Atlantic",
            &[
                "A heavy iron-hulled freighter steams past, smoke trailing from its stack. Northern build.",
                "A grey warship cuts through the swell, pennants snapping. The North Atlantic patrol.",
            ],
        ),
        (
            "West Africa",
            &[
                "A carved fishing boat with outriggers crosses your wake. The crew sings as they work.",
                "A cotton trader's vessel passes, bales stacked so high the deck is barely visible.",
            ],
        ),
        (
            "East Indies",
            &[
                "A junk with crimson sails and incense burners at the prow glides past in silence.",
                "A fleet of sampans appears from behind an island, loaded with silk-wrapped cargo.",
            ],
        ),
        (
            "South Seas",
            &[
                "A war canoe with painted warriors paddles past. They watch you but do not stop.",
                "An outrigger with pearl divers skims across the reef. They move like the water itself.",
            ],
        ),
    ];
    let region = FLAVOR[rng.choice_index(FLAVOR.len())];
    let msg = region.1[rng.choice_index(region.1.len())];
    let mut ev = VoyageEvent::new(EventType::ForeignVessel, msg);
    ev.flavor = "The sea is shared.".to_string();
    ev
}

fn resolve_cultural_waters(rng: &mut PyRandom) -> VoyageEvent {
    const LINES: &[&str] = &[
        "The water changes here — warmer, bluer. The Middle Sea welcomes you.",
        "Grey waves and cold spray. You feel the weight of the Iron Coast ahead.",
        "The current warms. Palm-fringed shores appear on the horizon. The Gold Coast.",
        "Jade-green water and the distant scent of spice. The Silk Waters begin here.",
        "Turquoise shallows and coral beneath the hull. The Reef Kingdoms lie ahead.",
    ];
    let mut ev = VoyageEvent::new(
        EventType::CulturalWaters,
        LINES[rng.choice_index(LINES.len())],
    );
    ev.flavor = "Every border on the sea is drawn by culture, not by stone.".to_string();
    ev
}

fn resolve_lighthouse(rng: &mut PyRandom) -> VoyageEvent {
    const BEACONS: &[&str] = &[
        "The beacon of Porto Novo breaks through the haze. You're on course.",
        "Ironhaven's Great Foundry chimney glows red on the horizon — a landmark for miles.",
        "The Wind Temple pagoda catches the sunset. Monsoon Reach is near.",
        "Ember Peak's volcanic glow marks the horizon. The South Seas await.",
        "The Whale Arch of Thornport stands white against the grey sky.",
    ];
    let mut ev = VoyageEvent::new(
        EventType::Lighthouse,
        BEACONS[rng.choice_index(BEACONS.len())],
    )
    .with_speed(1.1);
    ev.flavor = "Known waters. The charts don't lie.".to_string();
    ev
}

fn resolve_musician(rng: &mut PyRandom) -> VoyageEvent {
    const LINES: &[&str] = &[
        "A sailor plays a reed flute — an old Mediterranean melody about grain ships and fair winds.",
        "A northern ballad, deep and slow, about iron and ice and the lights in winter skies.",
        "Drums and singing from the crew — rhythms of the Gold Coast that make the work feel lighter.",
        "A string instrument hums from below deck — eastern scales that the crew learned in Jade Port.",
        "Shell horns and chanting — songs the crew picked up at Coral Throne. Haunting and beautiful.",
    ];
    let mut ev = VoyageEvent::new(
        EventType::MusicianAboard,
        LINES[rng.choice_index(LINES.len())],
    );
    ev.flavor = "For a moment, the sea feels smaller.".to_string();
    ev
}

fn resolve_offering(rng: &mut PyRandom) -> VoyageEvent {
    const LINES: &[&str] = &[
        "Floating flowers and a small wooden shrine drift past — an offering for safe passage.",
        "A garland of marigolds on a leaf boat. Someone prayed for a ship that never came home.",
        "A sealed clay jar bobs in the waves, marked with symbols of good fortune.",
        "Driftwood carved with old prayers. The crew leaves it undisturbed.",
    ];
    let mut ev = VoyageEvent::new(
        EventType::DriftingOffering,
        LINES[rng.choice_index(LINES.len())],
    );
    ev.flavor = "The sea remembers everyone who sails it.".to_string();
    ev
}

fn bounty_hunter_event(captain_silver_debt: i64, rng: &mut PyRandom) -> VoyageEvent {
    const HUNTERS: &[(&str, &str, &str, &str, i64)] = &[
        (
            "marshal_kael",
            "Marshal Kael",
            "northern_pact",
            "balanced",
            8,
        ),
        (
            "iron_hound",
            "The Iron Hound",
            "northern_pact",
            "aggressive",
            9,
        ),
        (
            "silent_mora",
            "Silent Mora",
            "northern_pact",
            "defensive",
            7,
        ),
    ];
    let hunter = HUNTERS[rng.choice_index(HUNTERS.len())];
    let mut ev = VoyageEvent::new(
        EventType::Pirates,
        format!(
            "A fast ship flying Pact colors cuts across your bow. {} stands at the rail, commission papers in hand. \"You owe debts, Captain. {captain_silver_debt} silver, or we settle this with steel.\"\n\nUse [bold]portlight encounter <negotiate|flee|fight>[/bold] to respond.",
            hunter.1
        ),
    );
    ev.flavor = format!(
        "Bounty hunter — strength {}, {} fighter. Demands {captain_silver_debt} silver.",
        hunter.4, hunter.3
    );
    ev.pending_duel = Some(PendingDuel {
        captain_id: hunter.0.to_string(),
        captain_name: hunter.1.to_string(),
        faction_id: hunter.2.to_string(),
        personality: hunter.3.to_string(),
        strength: hunter.4,
        region: String::new(),
    });
    ev
}

/// How a ship sits against a lane's `min_ship_class`.
///
/// A warning still sails (`depart` only rejects notes that start with
/// `BLOCKED`). The sail picker lists both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneSuitability {
    Ok,
    Warning,
    Blocked,
}

/// One neighbor the sail picker offers from the docked port.
///
/// `estimated_days` is `max(1, round(distance / raw_ship_speed))`. It ignores
/// crew, morale, season, and events. `depart` still applies those when the
/// day actually advances. Blocked lanes stay in this list.
#[derive(Debug, Clone, PartialEq)]
pub struct SailLane {
    pub destination_id: String,
    pub destination_name: String,
    pub region: String,
    pub distance: i64,
    pub danger: f64,
    pub min_ship_class: String,
    pub suitability: LaneSuitability,
    /// Python `check_route_suitability` text, when the fit is not clean.
    pub suitability_note: Option<String>,
    pub estimated_days: i64,
    /// `routes_view` provision column: estimated days plus a two-day buffer.
    pub provisions_needed: i64,
}

/// TUI `execute_sail_flow` speed when the captain has no ship.
///
/// `routes_view` uses 4 and `_estimate_sail_days` uses 6 in that same case.
/// `new_game` always fits a ship, so the picker path uses `ship.speed`.
const SAIL_PICKER_FALLBACK_SPEED: f64 = 5.0;

/// Estimated sail days from raw speed, matching the Python picker.
pub fn estimate_sail_days(distance: i64, raw_speed: f64) -> i64 {
    1.max(py_round(distance as f64 / raw_speed))
}

/// Lanes leaving the docked port, in the TUI sail picker's order.
///
/// Catalog order, then a stable sort by distance. Empty while `AtSea`,
/// matching `execute_sail_flow`. Every direct route is included, including
/// ones `depart` will refuse.
pub fn sail_lanes(world: &World) -> Vec<SailLane> {
    if world.voyage.status == VoyageStatus::AtSea {
        return Vec::new();
    }
    let port_id = world.voyage.destination_id.as_str();
    let speed = world
        .captain
        .ship
        .as_ref()
        .map(|ship| ship.speed)
        .unwrap_or(SAIL_PICKER_FALLBACK_SPEED);
    let mut lanes = Vec::new();
    for route in &world.routes {
        let dest_id = if route.port_a == port_id {
            route.port_b.as_str()
        } else if route.port_b == port_id {
            route.port_a.as_str()
        } else {
            continue;
        };
        let Some(dest) = world.port(dest_id) else {
            continue;
        };
        let (suitability, suitability_note) = match world.captain.ship.as_ref() {
            Some(ship) => {
                match check_route_suitability(&route.min_ship_class, &ship.template_id, &ship.name)
                {
                    Some(note) if note.starts_with("BLOCKED") => {
                        (LaneSuitability::Blocked, Some(note))
                    }
                    Some(note) => (LaneSuitability::Warning, Some(note)),
                    None => (LaneSuitability::Ok, None),
                }
            }
            None => (LaneSuitability::Ok, None),
        };
        let estimated_days = estimate_sail_days(route.distance, speed);
        lanes.push(SailLane {
            destination_id: dest.id.clone(),
            destination_name: dest.name.clone(),
            region: dest.region.clone(),
            distance: route.distance,
            danger: route.danger,
            min_ship_class: route.min_ship_class.clone(),
            suitability,
            suitability_note,
            estimated_days,
            provisions_needed: estimated_days + 2,
        });
    }
    lanes.sort_by_key(|lane| lane.distance);
    lanes
}

pub fn check_route_suitability(
    min_ship_class: &str,
    template_id: &str,
    ship_name: &str,
) -> Option<String> {
    let route_rank = class_rank(min_ship_class);
    let ship_rank = ship_class_rank(template_id);
    if ship_rank < route_rank {
        if route_rank - ship_rank >= 2 {
            return Some(format!(
                "BLOCKED: This route requires at least a {min_ship_class}. Your {ship_name} cannot attempt it."
            ));
        }
        return Some(format!(
            "WARNING: This route recommends a {min_ship_class}. Your {ship_name} will face increased danger."
        ));
    }
    None
}

pub fn depart(world: &mut World, destination_id: &str, defer_fee: bool) -> Result<(), SimError> {
    if world.captain.ship.is_none() {
        return Err(SimError::NoShip);
    }
    if world.voyage.status == VoyageStatus::AtSea {
        return Err(SimError::AlreadyAtSea);
    }
    let current = world.voyage.destination_id.clone();
    if current == destination_id {
        return Err(SimError::AlreadyAtThisPort);
    }
    let Some(route) = world.find_route(&current, destination_id) else {
        return Err(SimError::NoRoute {
            from: current,
            to: destination_id.to_string(),
        });
    };
    let distance = route.distance;
    let min_class = route.min_ship_class.clone();
    let Some(ship) = world.captain.ship.as_ref() else {
        return Err(SimError::NoShip);
    };
    if let Some(note) = check_route_suitability(&min_class, &ship.template_id, &ship.name) {
        if note.starts_with("BLOCKED") {
            return Err(SimError::RouteBlocked(note));
        }
    }
    let crew_min = template_crew_min(ship);
    if ship.crew < crew_min {
        return Err(SimError::CrewMinimum {
            need: crew_min,
            have: ship.crew,
        });
    }
    if let Some(port) = world.port(&current) {
        let fee_mult = captain_mods(world)
            .map(|m| m.pricing.port_fee_mult)
            .unwrap_or(1.0);
        let fee = 1.max(py_trunc(port.port_fee as f64 * fee_mult));
        if fee > world.captain.silver {
            if defer_fee {
                world.captain.deferred_fees.push(crate::model::DeferredFee {
                    fee_type: "port_fee".to_string(),
                    amount: fee * 2,
                    day: world.captain.day,
                });
            } else {
                return Err(SimError::NeedPortFee {
                    fee,
                    have: world.captain.silver,
                });
            }
        } else {
            world.captain.silver -= fee;
        }
    }
    fleet::form_convoy(&mut world.captain, &current);
    world.voyage = Voyage {
        origin_id: current,
        destination_id: destination_id.to_string(),
        distance,
        progress: 0,
        days_elapsed: 0,
        status: VoyageStatus::AtSea,
        recent_events: Vec::new(),
    };
    Ok(())
}

pub fn advance_day(
    world: &mut World,
    rng: &mut PyRandom,
    breach_count: i64,
) -> Result<Vec<VoyageEvent>, SimError> {
    if world.voyage.status != VoyageStatus::AtSea || world.captain.ship.is_none() {
        return Ok(Vec::new());
    }
    if world.pending_duel.is_some() {
        return Ok(Vec::new());
    }
    let mods = captain_mods(world);
    let provision_burn = mods.map(|m| m.voyage.provision_burn).unwrap_or(1.0);
    let speed_bonus = mods.map(|m| m.voyage.speed_bonus).unwrap_or(0.0);
    let mut inspection_mult = mods
        .map(|m| m.inspection.inspection_chance_mult)
        .unwrap_or(1.0);
    let dest_region = world
        .port(&world.voyage.destination_id)
        .map(|p| p.region.clone())
        .unwrap_or_else(|| "Mediterranean".to_string());
    inspection_mult *= inspection_modifier(&world.captain.standing, &dest_region);
    if world.captain.wanted_level >= 2 {
        inspection_mult *= 1.5;
    } else if world.captain.wanted_level >= 1 {
        inspection_mult *= 1.2;
    }

    let mut events = Vec::new();
    if provision_burn >= 1.0 || rng.random() < provision_burn {
        world.captain.provisions -= 1;
    }
    if world.captain.provisions < 0 {
        world.captain.provisions = 0;
        let mut ev = VoyageEvent::new(EventType::Nothing, "No provisions! The crew suffers.");
        ev.crew_delta = -1;
        events.push(ev);
    }

    let wage_cost = world.captain.ship.as_ref().map(wage_bill).unwrap_or(0)
        + fleet::fleet_daily_wages(&world.captain);
    let wages_paid = if wage_cost > 0 && world.captain.silver >= wage_cost {
        world.captain.silver -= wage_cost;
        true
    } else if wage_cost > 0 {
        let mut ev = VoyageEvent::new(EventType::Nothing, "Can't pay crew wages! Morale drops.");
        ev.crew_delta = -1;
        events.push(ev);
        false
    } else {
        true
    };
    let provisions_ok = world.captain.provisions > 0;
    let days_elapsed = world.voyage.days_elapsed;
    if let Some(ship) = world.captain.ship.as_mut() {
        ship.morale = tick_morale_at_sea(ship, wages_paid, provisions_ok, days_elapsed);
    }

    let route_danger = world
        .find_route(&world.voyage.origin_id, &world.voyage.destination_id)
        .map(|r| (r.danger, r.min_ship_class.clone()));
    let mut danger = route_danger.as_ref().map(|r| r.0).unwrap_or(0.1);
    if let Some((_, min_class)) = &route_danger {
        let template_id = world
            .captain
            .ship
            .as_ref()
            .map(|s| s.template_id.as_str())
            .unwrap_or("");
        if ship_class_rank(template_id) < class_rank(min_class) {
            danger *= 1.5;
        }
    }
    if let Some(profile) = content::content().season_profile(&dest_region, world.day) {
        danger *= profile.danger_mult;
    }
    let recent = world.voyage.recent_events.clone();
    let event_type = pick_event(danger, rng, inspection_mult, &recent);
    let event = resolve_event(event_type, rng, world)?;
    let picked = event_type.as_str().to_string();

    if world.captain.wanted_level >= 3 && event.pending_duel.is_none() && rng.random() < 0.15 {
        let loans = world
            .captain
            .deferred_fees
            .iter()
            .filter(|f| f.fee_type == "emergency_loan")
            .map(|f| f.amount)
            .sum::<i64>();
        // `_bounty_hunter_event`: emergency-loan principal plus 50 silver per breach.
        let demand = loans + breach_count * 50;
        let bh = bounty_hunter_event(demand, rng);
        if event.pending_duel.is_none() {
            if let Some(duel) = bh.pending_duel.clone() {
                world.pending_duel = Some(duel);
            }
        }
        events.push(event);
        events.push(bh);
    } else {
        if event.pending_duel.is_some() {
            world.pending_duel = event.pending_duel.clone();
        }
        events.push(event);
    }
    // `event` was moved. Re-bind the route event as the one that is not a later bounty append.
    // The route event is the last one only when no bounty was appended; with bounty it is the
    // second to last. Recover by index.
    let route_index = if events.len() >= 2
        && events
            .last()
            .is_some_and(|e| e.flavor.starts_with("Bounty hunter"))
    {
        events.len() - 2
    } else {
        events.len() - 1
    };

    world.voyage.recent_events.push(picked);
    if world.voyage.recent_events.len() > 5 {
        let extra = world.voyage.recent_events.len() - 5;
        world.voyage.recent_events.drain(0..extra);
    }

    let route_event = events[route_index].clone();
    if let Some(ship) = world.captain.ship.as_mut() {
        ship.hull = 0.max(ship.hull + route_event.hull_delta);
    }
    fleet::damage_convoy(&mut world.captain, route_event.hull_delta);
    world.captain.provisions = 0.max(world.captain.provisions + route_event.provision_delta);
    world.captain.silver = 0.max(world.captain.silver + route_event.silver_delta);

    let crew_deltas: Vec<(i64, String)> = events
        .iter()
        .filter(|ev| ev.crew_delta != 0)
        .map(|ev| {
            let ctx = if ev.event_type == EventType::Storm {
                "storm"
            } else {
                "voyage"
            };
            (ev.crew_delta, ctx.to_string())
        })
        .collect();
    if let Some(ship) = world.captain.ship.as_mut() {
        for (delta, ctx) in crew_deltas {
            apply_crew_delta(ship, delta, &ctx, rng);
        }
    }
    for ev in &events {
        for (good_id, lost) in &ev.cargo_lost {
            if *lost > 0 {
                consume_cargo_fifo(&mut world.captain.cargo, good_id, *lost);
            }
        }
    }

    let season_speed = content::content()
        .season_profile(&dest_region, world.day)
        .map(|p| p.speed_mult)
        .unwrap_or(1.0);
    let day_progress = {
        let Some(ship) = world.captain.ship.as_ref() else {
            return Err(SimError::NoShip);
        };
        let mut base_speed = resolve_speed(ship) + speed_bonus + navigator_speed_bonus(ship);
        let crew_min = template_crew_min(ship);
        if ship.crew < crew_min {
            base_speed *= 0.5;
        } else if ship.crew < ship.crew_max {
            let crew_ratio = ship.crew as f64 / ship.crew_max as f64;
            base_speed *= 0.7 + 0.3 * crew_ratio;
        }
        base_speed = fleet::convoy_speed(&world.captain, base_speed);
        base_speed *= morale_speed_modifier(ship.morale);
        base_speed *= season_speed;
        py_trunc(base_speed * route_event.speed_modifier)
    };
    world.voyage.progress += day_progress;
    world.voyage.days_elapsed += 1;
    world.day += 1;
    world.captain.day += 1;

    if world.voyage.days_elapsed > 0 && world.voyage.days_elapsed % 20 == 0 {
        if let Some(ship) = world.captain.ship.as_mut() {
            if ship.hull_max > 20 {
                ship.hull_max -= 1;
                ship.hull = ship.hull.min(ship.hull_max);
            }
        }
        fleet::wear_convoy(&mut world.captain);
    }
    if world.voyage.progress >= world.voyage.distance {
        world.voyage.status = VoyageStatus::Arrived;
    }
    Ok(events)
}

pub fn arrive(world: &mut World) -> Result<(), SimError> {
    if world.voyage.status != VoyageStatus::Arrived {
        return Err(SimError::NotArrivedYet);
    }
    world.voyage.status = VoyageStatus::InPort;
    if let Some(ship) = world.captain.ship.as_mut() {
        let cabin = has_special(ship, "morale_bonus");
        ship.morale = tick_morale_at_port(ship, cabin);
    }
    let destination = world.voyage.destination_id.clone();
    fleet::dock_convoy(&mut world.captain, &destination);
    if !world.captain.deferred_fees.is_empty() {
        let mut remaining = Vec::new();
        for fee in world.captain.deferred_fees.drain(..) {
            if world.captain.silver >= fee.amount {
                world.captain.silver -= fee.amount;
            } else {
                remaining.push(fee);
            }
        }
        world.captain.deferred_fees = remaining;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::new_game;

    fn lane<'a>(lanes: &'a [SailLane], id: &str) -> &'a SailLane {
        lanes
            .iter()
            .find(|lane| lane.destination_id == id)
            .unwrap_or_else(|| panic!("missing lane {id}"))
    }

    #[test]
    fn picker_uses_raw_speed_and_lists_warning_lanes() {
        let world = new_game("Ada", "merchant", 42, None).expect("game");
        assert_eq!(world.captain.ship.as_ref().map(|s| s.speed), Some(8.0));
        let lanes = sail_lanes(&world);
        let ids: Vec<_> = lanes
            .iter()
            .map(|lane| lane.destination_id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec![
                "silva_bay",
                "corsairs_rest",
                "al_manar",
                "ironhaven",
                "sun_harbor",
            ]
        );

        let silva = lane(&lanes, "silva_bay");
        assert_eq!(silva.distance, 16);
        assert_eq!(silva.min_ship_class, "sloop");
        assert_eq!(silva.suitability, LaneSuitability::Ok);
        assert_eq!(silva.estimated_days, 2);
        assert_eq!(silva.provisions_needed, 4);

        // 36 / 8 = 4.5. Python round half-to-even yields 4. The sea day applies
        // crew, morale, season, and the event modifier, so the sailed progress
        // is a different number. The picker must stay on 4.
        let ironhaven = lane(&lanes, "ironhaven");
        assert_eq!(ironhaven.min_ship_class, "cutter");
        assert_eq!(ironhaven.suitability, LaneSuitability::Warning);
        assert_eq!(ironhaven.estimated_days, estimate_sail_days(36, 8.0));
        assert_eq!(ironhaven.estimated_days, 4);
        assert_eq!(
            ironhaven.suitability_note.as_deref(),
            Some(
                "WARNING: This route recommends a cutter. Your Coastal Sloop will face increased danger."
            )
        );

        let sun = lane(&lanes, "sun_harbor");
        assert_eq!(sun.suitability, LaneSuitability::Warning);
        assert_eq!(sun.estimated_days, 5);
    }

    #[test]
    fn blocked_lane_stays_listed_and_depart_refuses_it() {
        let mut world = new_game("Ada", "merchant", 42, Some("sun_harbor")).expect("game");
        let lanes = sail_lanes(&world);
        let crosswind = lane(&lanes, "crosswind_isle");
        assert_eq!(crosswind.min_ship_class, "galleon");
        assert_eq!(crosswind.suitability, LaneSuitability::Blocked);
        assert_eq!(crosswind.estimated_days, 8);
        assert_eq!(
            crosswind.suitability_note.as_deref(),
            Some(
                "BLOCKED: This route requires at least a galleon. Your Coastal Sloop cannot attempt it."
            )
        );

        let al_manar = lane(&lanes, "al_manar");
        assert_eq!(al_manar.min_ship_class, "brigantine");
        assert_eq!(al_manar.suitability, LaneSuitability::Blocked);

        let err = depart(&mut world, "crosswind_isle", false).expect_err("blocked");
        assert!(err.to_string().starts_with("BLOCKED"), "{err}");
        assert_eq!(world.voyage.status, VoyageStatus::InPort);

        let warned = lane(&lanes, "porto_novo");
        assert_eq!(warned.suitability, LaneSuitability::Warning);
        depart(&mut world, "porto_novo", false).expect("warning still sails");
        assert_eq!(world.voyage.status, VoyageStatus::AtSea);
        assert!(sail_lanes(&world).is_empty());
    }

    #[test]
    fn short_crew_slows_the_day_and_day_20_wears_hull() {
        use crate::model::Voyage;
        use crate::pyrand::PyRandom;

        fn one_day(crew: i64) -> i64 {
            let mut world = new_game("Ada", "merchant", 99, None).expect("game");
            let ship = world.captain.ship.as_mut().expect("ship");
            ship.crew = crew;
            ship.sailors = crew;
            ship.gunners = 0;
            ship.navigators = 0;
            ship.surgeons = 0;
            ship.marines = 0;
            ship.quartermasters = 0;
            world.captain.provisions = 40;
            world.captain.silver = 500;
            world.voyage = Voyage {
                origin_id: "porto_novo".to_string(),
                destination_id: "silva_bay".to_string(),
                distance: 10_000,
                progress: 0,
                days_elapsed: 0,
                status: VoyageStatus::AtSea,
                recent_events: Vec::new(),
            };
            let mut rng = PyRandom::from_seed(99);
            advance_day(&mut world, &mut rng, 0).expect("day");
            world.voyage.progress
        }

        let full = one_day(8);
        let partial = one_day(3);
        let short = one_day(1);
        assert!(full > partial, "full {full} partial {partial}");
        assert!(partial > short, "partial {partial} short {short}");

        let mut world = new_game("Ada", "merchant", 99, None).expect("game");
        let hull_max = world.captain.ship.as_ref().expect("ship").hull_max;
        world.voyage = Voyage {
            origin_id: "porto_novo".to_string(),
            destination_id: "silva_bay".to_string(),
            distance: 10_000,
            progress: 0,
            days_elapsed: 0,
            status: VoyageStatus::AtSea,
            recent_events: Vec::new(),
        };
        world.captain.provisions = 80;
        world.captain.silver = 5_000;
        let mut rng = PyRandom::from_seed(99);
        let mut wore = false;
        for _ in 0..40 {
            if world.pending_duel.is_some() {
                world.pending_duel = None;
            }
            let before_days = world.voyage.days_elapsed;
            let before_hull = world.captain.ship.as_ref().expect("ship").hull_max;
            advance_day(&mut world, &mut rng, 0).expect("day");
            let after_days = world.voyage.days_elapsed;
            let after_hull = world.captain.ship.as_ref().expect("ship").hull_max;
            if after_days == 20 {
                assert_eq!(after_hull, before_hull - 1);
                wore = true;
                break;
            } else if after_days != before_days {
                assert_eq!(after_hull, before_hull);
            }
        }
        assert!(wore, "hull wear did not fire");
        assert!(hull_max > 20);
    }
}
