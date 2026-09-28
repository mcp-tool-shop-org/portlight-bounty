//! Forage for provisions, pelts, and silver. Port of `engine/hunting.py`.
//!
//! `hunt` is `engine/hunting.py` line 100. The session applies the yield
//! and copies `captain.day` onto `world.day`.

use crate::model::Captain;
use crate::pyrand::PyRandom;

/// Outcome of one hunting day. Python `HuntResult`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HuntResult {
    pub success: bool,
    pub provisions_gained: i64,
    pub pelts_gained: i64,
    pub silver_gained: i64,
    pub morale_cost: i64,
    pub crew_lost: i64,
    pub hull_damage: i64,
    pub flavor: String,
    pub danger_text: String,
}

const PORT_SUCCESS: &[&str] = &[
    "Your crew hauls in a decent catch from the harbor waters. Fresh fish for everyone.",
    "The nearby woods yield rabbits and wild herbs. The cook is pleased.",
    "Local trappers trade tips with your crew. A productive morning.",
    "Shore birds and shellfish — not glamorous, but the provisions hold is fuller.",
    "A fishmonger buys your surplus catch. A few coins earned.",
    "The crew finds a beached whale carcass — blubber, bone, and oil to sell.",
];

const PORT_FAIL: &[&str] = &[
    "The crew spends the day fishing but catches nothing worth keeping.",
    "Rain drives the hunting party back early. A wasted effort.",
    "The harbor waters are fished out. Other crews got here first.",
];

const SEA_SUCCESS: &[&str] = &[
    "A school of fish passes beneath the hull. The crew drops nets and hauls aboard a good catch.",
    "The lookout spots a seal colony on a rocky outcrop. Your crew returns with pelts and meat.",
    "Drifting kelp beds teem with crabs and small fish. Easy pickings for hungry sailors.",
    "A large tuna breaks the surface. Your best harpooner doesn't miss.",
    "Seabirds circle a bait ball. Where there are birds, there are fish. The nets come up full.",
];

const SEA_FAIL: &[&str] = &[
    "The sea gives nothing today. Your crew stares at empty nets.",
    "A promising fishing spot turns up nothing but jellyfish and seaweed.",
    "The waters are too deep and too cold. Nothing bites.",
    "A sudden squall forces the fishing party back aboard. No catch today.",
];

struct SeaDanger {
    text: &'static str,
    crew_lost: i64,
    hull_damage: i64,
    morale_cost: i64,
}

const SEA_DANGERS: &[SeaDanger] = &[
    SeaDanger {
        text: "A shark tears through the nets, dragging a sailor overboard. He's pulled back but badly cut.",
        crew_lost: 0,
        hull_damage: 0,
        morale_cost: 5,
    },
    SeaDanger {
        text: "The rowboat scrapes against a submerged rock. Minor hull damage, but the catch is lost.",
        crew_lost: 0,
        hull_damage: 3,
        morale_cost: 2,
    },
    SeaDanger {
        text: "A rogue wave swamps the fishing party. One sailor doesn't surface.",
        crew_lost: 1,
        hull_damage: 0,
        morale_cost: 8,
    },
    SeaDanger {
        text: "The nets snag on coral and tear apart. Equipment lost, nothing to show for the effort.",
        crew_lost: 0,
        hull_damage: 0,
        morale_cost: 3,
    },
    SeaDanger {
        text: "A saltwater crocodile lunges from a mangrove island. The crew rows back in terror.",
        crew_lost: 0,
        hull_damage: 0,
        morale_cost: 6,
    },
];

/// `engine/hunting.py` `hunt` (line 100).
///
/// Port is an 80% chance and costs no morale. Sea is 50% and costs 3, then
/// a separate 15% danger roll. Draws stay in Python's order. `captain.day`
/// advances by one. The caller copies that onto `world.day` and applies yield.
pub fn hunt(
    captain: &mut Captain,
    location: &str,
    crew_count: i64,
    rng: &mut PyRandom,
) -> HuntResult {
    let (success_chance, base_morale_cost) = if location == "port" {
        (0.8, 0)
    } else {
        (0.5, 3)
    };
    let success = rng.random() < success_chance;
    let crew_bonus = (crew_count / 3).min(2);
    let mut silver = 0;
    let mut provisions = 0;
    let mut pelts = 0;
    let mut danger_text = String::new();
    let mut crew_lost = 0;
    let mut hull_damage = 0;
    let mut extra_morale = 0;
    let flavor;
    if success {
        if location == "port" {
            provisions = rng.randint(2, 4) + crew_bonus;
            pelts = rng.randint(0, 2);
            silver = if rng.random() < 0.4 {
                rng.randint(1, 3)
            } else {
                0
            };
            flavor = PORT_SUCCESS[rng.choice_index(PORT_SUCCESS.len())].to_string();
        } else {
            provisions = rng.randint(1, 3) + crew_bonus;
            pelts = rng.randint(0, 2);
            silver = if rng.random() < 0.5 {
                rng.randint(2, 6)
            } else {
                0
            };
            flavor = SEA_SUCCESS[rng.choice_index(SEA_SUCCESS.len())].to_string();
        }
    } else {
        flavor = if location == "port" {
            PORT_FAIL[rng.choice_index(PORT_FAIL.len())].to_string()
        } else {
            SEA_FAIL[rng.choice_index(SEA_FAIL.len())].to_string()
        };
    }
    if location == "sea" && rng.random() < 0.15 {
        let danger = &SEA_DANGERS[rng.choice_index(SEA_DANGERS.len())];
        danger_text = danger.text.to_string();
        crew_lost = danger.crew_lost;
        hull_damage = danger.hull_damage;
        extra_morale = danger.morale_cost;
        if !success {
            provisions = 0;
            pelts = 0;
            silver = 0;
        }
    }
    captain.day += 1;
    HuntResult {
        success,
        provisions_gained: provisions,
        pelts_gained: pelts,
        silver_gained: silver,
        morale_cost: base_morale_cost + extra_morale,
        crew_lost,
        hull_damage,
        flavor,
        danger_text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::new_game;

    fn hunt_seed(seed: i128, location: &str) -> HuntResult {
        let mut world = new_game("Ada", "merchant", seed, None).unwrap();
        let mut rng = PyRandom::from_seed(seed);
        let crew = world.captain.ship.as_ref().unwrap().crew;
        hunt(&mut world.captain, location, crew, &mut rng)
    }

    #[test]
    fn port_and_sea_draws_match_cpython() {
        let port = hunt_seed(1, "port");
        assert!(port.success);
        assert_eq!(port.provisions_gained, 3);
        assert_eq!(port.pelts_gained, 1);
        assert_eq!(port.silver_gained, 2);
        assert_eq!(port.morale_cost, 0);
        assert_eq!(
            port.flavor,
            "Shore birds and shellfish — not glamorous, but the provisions hold is fuller."
        );

        let miss = hunt_seed(2, "port");
        assert!(!miss.success);
        assert_eq!(
            miss.flavor,
            "The crew spends the day fishing but catches nothing worth keeping."
        );

        let sea = hunt_seed(1, "sea");
        assert!(sea.success);
        assert_eq!(
            (sea.provisions_gained, sea.pelts_gained, sea.silver_gained),
            (2, 1, 5)
        );
        assert_eq!(sea.morale_cost, 3);
        assert_eq!(
            sea.flavor,
            "A large tuna breaks the surface. Your best harpooner doesn't miss."
        );

        let wave = hunt_seed(2, "sea");
        assert!(!wave.success);
        assert_eq!(wave.morale_cost, 11);
        assert_eq!(wave.crew_lost, 1);
        assert_eq!(
            wave.danger_text,
            "A rogue wave swamps the fishing party. One sailor doesn't surface."
        );
    }
}
