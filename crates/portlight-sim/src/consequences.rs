//! History-gated encounters.
//!
//! Ported from `engine/consequences.py`. Sea checks run from
//! [`crate::sea_culture::enrich_voyage_day`]. Port checks run when a voyage
//! arrives. Effects are applied immediately. Python does not keep a delayed
//! consequence queue, and v12 does not store one.
//!
//! Contract outcomes still carry `trust_delta`, `standing_delta`, and
//! `heat_delta`. `GameSession.advance` uses those only to size a
//! `contract_failure` claim. It does not write them onto standing. This module
//! writes the deltas on its own [`Consequence`] values.

use crate::content;
use crate::cross_port_networks;
use crate::economy::TradeReceipt;
use crate::model::{Consequence, ContractBoard, ContractOutcome, Standing, World};
use crate::pyrand::PyRandom;
use crate::util::py_set_order;

const CONTRABAND: [&str; 3] = ["opium", "black_powder", "stolen_cargo"];

pub fn check_sea_consequences(
    world: &World,
    receipts: &[TradeReceipt],
    total_sells: i64,
    board: &ContractBoard,
    rng: &mut PyRandom,
) -> Vec<Consequence> {
    if rng.random() > 0.15 {
        return Vec::new();
    }
    let captain = &world.captain;
    let dest_region = world
        .port(&world.voyage.destination_id)
        .map(|port| port.region.clone())
        .unwrap_or_else(|| "Mediterranean".to_string());

    if total_sells >= 10 && captain.standing.commercial_trust >= 10 && rng.random() < 0.3 {
        let silver_gift = rng.randint(10, 30);
        return vec![Consequence {
            id: "fair_trader_gift".to_string(),
            category: "sea".to_string(),
            trigger: format!(
                "Completed {total_sells} honest trades with trust {}",
                captain.standing.commercial_trust
            ),
            text: format!(
                "A merchant vessel approaches flying a friendly flag. The captain shouts: 'You traded fairly with us at port — my employer sends {silver_gift} silver as a thank-you and an invitation to do business again.' A crate of silver is swung across on a rope."
            ),
            effect_type: "reward".to_string(),
            silver_delta: silver_gift,
            standing_delta: 0,
            heat_delta: 0,
            trust_delta: 0,
            region: String::new(),
        }];
    }

    for (faction_id, standing) in &captain.standing.underworld {
        if *standing < 5 && rng.random() < 0.4 {
            if let Some(faction) = content::content().faction(faction_id) {
                if faction
                    .territory_regions
                    .iter()
                    .any(|region| region == &dest_region)
                {
                    return vec![Consequence {
                        id: "faction_warning".to_string(),
                        category: "sea".to_string(),
                        trigger: format!(
                            "Low standing ({standing}) with {} in their territory",
                            faction.name
                        ),
                        text: format!(
                            "A ship flying the {}'s colors crosses your bow — close, deliberate, threatening. A signal flag spells out: 'UNWELCOME.' They don't attack. Not today. But the message is clear: you are being watched, and the next encounter might not end with a warning.",
                            faction.name
                        ),
                        effect_type: "threat".to_string(),
                        silver_delta: 0,
                        standing_delta: 0,
                        heat_delta: 2,
                        trust_delta: 0,
                        region: dest_region,
                    }];
                }
            }
        }
    }

    if world.captain.duels_won >= 1 {
        for encounter in &world.captain.encounters {
            if encounter.outcome == "duel_win" && rng.random() < 0.2 {
                if let Some(pirate) = content::content().pirate(&encounter.captain_id) {
                    return vec![Consequence {
                        id: "spared_pirate_gratitude".to_string(),
                        category: "sea".to_string(),
                        trigger: format!(
                            "Won a duel against {} and let them live",
                            pirate.name
                        ),
                        text: format!(
                            "A familiar sail on the horizon — {}'s ship. Your crew tenses. But instead of approaching, the ship changes course — away from you, revealing a navy patrol that was behind them. They spotted the patrol and warned you by drawing its attention. A debt repaid.",
                            pirate.name
                        ),
                        effect_type: "reward".to_string(),
                        silver_delta: 0,
                        standing_delta: 0,
                        heat_delta: 0,
                        trust_delta: 0,
                        region: String::new(),
                    }];
                }
            }
        }
    }

    if let Some(nemesis_id) = &world.nemesis_id {
        if rng.random() < 0.3 {
            if let Some(nemesis) = content::content().pirate(nemesis_id) {
                return vec![Consequence {
                    id: "nemesis_shadow".to_string(),
                    category: "sea".to_string(),
                    trigger: format!("Nemesis {} is tracking you", nemesis.name),
                    text: format!(
                        "Your lookout spots a ship matching your course — same heading, same speed, keeping distance. {}'s vessel. They're not attacking. They're FOLLOWING. Learning your route, your schedule, your patterns. The next time they appear, they'll be ready.",
                        nemesis.name
                    ),
                    effect_type: "threat".to_string(),
                    silver_delta: 0,
                    standing_delta: 0,
                    heat_delta: 0,
                    trust_delta: 0,
                    region: String::new(),
                }];
            }
        }
    }

    if receipts.len() >= 20 && rng.random() < 0.25 {
        let traded = py_set_order(receipts.iter().map(|receipt| receipt.port_id.as_str()));
        if !traded.is_empty() {
            let tip_port = &traded[rng.choice_index(traded.len())];
            if let Some(port) = world.port(tip_port) {
                if !port.market.is_empty() {
                    let slot = &port.market[rng.choice_index(port.market.len())];
                    // Python compares `stock_current < stock_target * 0.5`.
                    // The product is a float, so the int is promoted. `int()`
                    // truncation is not applied.
                    if (slot.stock_current as f64) < (slot.stock_target as f64) * 0.5 {
                        return vec![Consequence {
                                id: "trade_intelligence".to_string(),
                                category: "sea".to_string(),
                                trigger: format!(
                                    "Traded at {} ports, earned trust",
                                    traded.len()
                                ),
                                text: format!(
                                    "A passing merchant hails you: 'Captain! We've traded before — at {}, yes? I have news: {} is running scarce there. Prices will climb. If you have any, now's the time.' He tips his hat and sails on. Information freely given — because you earned it with honest trade.",
                                    port.name, slot.good_id
                                ),
                                effect_type: "information".to_string(),
                                silver_delta: 0,
                                standing_delta: 0,
                                heat_delta: 0,
                                trust_delta: 0,
                                region: String::new(),
                            }];
                    }
                }
            }
        }
    }

    let failed = failed_contracts(&board.completed);
    if !failed.is_empty() && rng.random() < 0.3 {
        let failed = &failed[rng.choice_index(failed.len())];
        return vec![Consequence {
            id: "broken_contract_haunts".to_string(),
            category: "sea".to_string(),
            trigger: format!("Failed contract: {}", failed.summary),
            text: "A merchant vessel pulls alongside. The captain's face is stone. 'You were supposed to deliver. You didn't. My employer lost silver. I lost my commission. Word travels, Captain. The exchanges are talking about you — and what they're saying isn't good.' He turns away without waiting for a response.".to_string(),
            effect_type: "threat".to_string(),
            silver_delta: 0,
            standing_delta: 0,
            heat_delta: 0,
            trust_delta: -1,
            region: String::new(),
        }];
    }

    let regional_standing = captain.standing.regional_of(&dest_region);
    if regional_standing >= 15 && rng.random() < 0.2 {
        return vec![Consequence {
            id: "regional_escort".to_string(),
            category: "sea".to_string(),
            trigger: format!("High standing ({regional_standing}) in {dest_region}"),
            text: format!(
                "A patrol vessel from {dest_region} approaches — not to inspect, but to escort. 'Captain, your reputation precedes you. We'll see you safely to port.' They fall in alongside. For the next stretch, the danger drops to zero. This is what standing buys: not immunity, but protection."
            ),
            effect_type: "reward".to_string(),
            silver_delta: 0,
            standing_delta: 0,
            heat_delta: 0,
            trust_delta: 0,
            region: String::new(),
        }];
    }

    let contraband_trades = receipts
        .iter()
        .filter(|receipt| CONTRABAND.contains(&receipt.good_id.as_str()))
        .count();
    let heat = captain.standing.heat_of(&dest_region);
    if contraband_trades >= 3 && heat >= 10 && rng.random() < 0.3 {
        return vec![Consequence {
            id: "contraband_reputation".to_string(),
            category: "sea".to_string(),
            trigger: format!("Traded contraband {contraband_trades} times with heat {heat}"),
            text: "A customs cutter appears from behind an island — fast, purposeful, heading straight for you. They don't inspect. They circle your ship slowly, noting your hull number, your heading, your cargo profile. Then they leave. But now they know your route. Next time, the inspection will be thorough.".to_string(),
            effect_type: "threat".to_string(),
            silver_delta: 0,
            standing_delta: 0,
            heat_delta: 3,
            trust_delta: 0,
            region: dest_region,
        }];
    }

    if captain.day > 100 && rng.random() < 0.15 {
        return vec![Consequence {
            id: "old_crew_memory".to_string(),
            category: "sea".to_string(),
            trigger: format!("Veteran captain (day {})", captain.day),
            text: "A fishing boat hails you. At the tiller, a face you half-recognize — a sailor who served under you months ago. He waves. 'Captain! Still sailing! I tell everyone I served with you. Some are impressed. Some aren't.' He laughs and heads on. The sea is full of people who remember you.".to_string(),
            effect_type: "neutral".to_string(),
            silver_delta: 0,
            standing_delta: 0,
            heat_delta: 0,
            trust_delta: 0,
            region: String::new(),
        }];
    }
    Vec::new()
}

pub fn check_port_consequences(
    world: &World,
    port_id: &str,
    receipts: &[TradeReceipt],
    _total_sells: i64,
    board: &ContractBoard,
    rng: &mut PyRandom,
) -> Vec<Consequence> {
    if rng.random() > 0.25 {
        return Vec::new();
    }
    let Some(port) = world.port(port_id) else {
        return Vec::new();
    };
    let port_name = port.name.clone();
    let region = port.region.clone();
    let captain = &world.captain;

    let port_trades: Vec<&TradeReceipt> = receipts
        .iter()
        .filter(|receipt| receipt.port_id == port_id)
        .collect();
    if port_trades.len() >= 5 {
        let profitable = port_trades
            .iter()
            .filter(|receipt| receipt.action == "sell")
            .count();
        if profitable >= 3 && rng.random() < 0.4 {
            let discount = rng.randint(5, 15);
            return vec![Consequence {
                id: "loyal_customer".to_string(),
                category: "port".to_string(),
                trigger: format!("Traded {} times at {port_name}", port_trades.len()),
                text: format!(
                    "A dockworker recognizes your ship. 'Captain! You're back!' Word spreads quickly. By the time you've tied up, the harbor master has reduced your docking fee by {discount} silver. 'Loyal customers get loyal treatment,' he says. The port remembers who keeps coming back."
                ),
                effect_type: "reward".to_string(),
                silver_delta: discount,
                standing_delta: 0,
                heat_delta: 0,
                trust_delta: 0,
                region: String::new(),
            }];
        }
    }

    let failed = failed_contracts(&board.completed);
    let port_failures =
        !failed.is_empty() && receipts.iter().any(|receipt| receipt.port_id == port_id);
    if port_failures && rng.random() < 0.4 {
        return vec![Consequence {
            id: "contract_shame".to_string(),
            category: "port".to_string(),
            trigger: format!("Failed contract visible at {port_name}"),
            text: "As you dock, you notice the market board. Your name is on it — in the 'defaulted contracts' section. The merchants at the exchange glance at you and look away. The broker's desk has a shorter list of offers today. Word travels. Trust is the hardest thing to rebuild.".to_string(),
            effect_type: "threat".to_string(),
            silver_delta: 0,
            standing_delta: 0,
            heat_delta: 0,
            trust_delta: -1,
            region: String::new(),
        }];
    }

    if let Some(dominant) = content::content().factions_in(&region).into_iter().next() {
        let faction_standing = underworld_of(&captain.standing, &dominant.id);
        if faction_standing >= 25 && rng.random() < 0.35 {
            let silver_offer = rng.randint(20, 50);
            return vec![Consequence {
                id: "faction_favor".to_string(),
                category: "port".to_string(),
                trigger: format!(
                    "Standing {faction_standing} with {} at their port",
                    dominant.name
                ),
                text: format!(
                    "A figure in {} colors approaches as you dock. No introduction -- they know who you are. '{} appreciates your loyalty. A small token.' They press {silver_offer} silver into your hand. 'There's more where that came from. Keep trading with us.' They vanish into the crowd.",
                    dominant.name, dominant.name
                ),
                effect_type: "reward".to_string(),
                silver_delta: silver_offer,
                standing_delta: 0,
                heat_delta: 0,
                trust_delta: 0,
                region: String::new(),
            }];
        }
    }

    let port_heat = captain.standing.heat_of(&region);
    if port_heat >= 20 && rng.random() < 0.4 {
        return vec![Consequence {
            id: "heat_scrutiny".to_string(),
            category: "port".to_string(),
            trigger: format!("Customs heat {port_heat} in {region}"),
            text: "Before your anchor hits bottom, a customs launch is alongside. Two inspectors, not one. They don't wait for you to present a manifest — they board directly. 'Routine inspection,' the lead says, without making eye contact. It's not routine. Your reputation has preceded you.".to_string(),
            effect_type: "threat".to_string(),
            silver_delta: 0,
            standing_delta: 0,
            heat_delta: 2,
            trust_delta: 0,
            region,
        }];
    }

    if captain.standing.commercial_trust >= 8 && rng.random() < 0.3 {
        if let Some(gossip) = cross_port_networks::gossip(port_id, rng) {
            return vec![Consequence {
                id: "network_gossip".to_string(),
                category: "port".to_string(),
                trigger: format!(
                    "Trust {}, {} shares network intelligence",
                    captain.standing.commercial_trust, gossip.local_name
                ),
                text: gossip.text,
                effect_type: "information".to_string(),
                silver_delta: 0,
                standing_delta: 0,
                heat_delta: 0,
                trust_delta: 0,
                region: String::new(),
            }];
        }
    }

    if let Some(template) = crate::custom_captain::active_captain(world) {
        if template.home_port_id == port_id && captain.day > 30 {
            let visit_count = world.culture.visits(port_id);
            if visit_count <= 2 && rng.random() < 0.5 {
                return vec![Consequence {
                    id: "homecoming".to_string(),
                    category: "port".to_string(),
                    trigger: format!(
                        "Returning to home port {port_name} after {} days",
                        captain.day
                    ),
                    text: "You're home. The harbor looks different after weeks at sea — smaller somehow, but warmer. A dockworker you've known since childhood spots your ship and shouts to the others. By the time you tie up, a small crowd has gathered. You left as a captain with a sloop and a dream. You've returned as someone the port talks about. The feeling is... complicated.".to_string(),
                    effect_type: "reward".to_string(),
                    silver_delta: 0,
                    standing_delta: 2,
                    heat_delta: 0,
                    trust_delta: 0,
                    region: region.clone(),
                }];
            }
        }
    }

    let visit_count = world.culture.visits(port_id);
    if visit_count == 0 && rng.random() < 0.5 {
        return vec![Consequence {
            id: "first_visit".to_string(),
            category: "port".to_string(),
            trigger: format!("First visit to {port_name}"),
            text: "Everything is new. The smells, the sounds, the way the dockworkers move — it's all different from what you know. A port you've never visited before. Every stall is a question, every face is a stranger, and every price is a test. This is why you sail: for the moment when the world shows you something you haven't seen.".to_string(),
            effect_type: "neutral".to_string(),
            silver_delta: 0,
            standing_delta: 0,
            heat_delta: 0,
            trust_delta: 0,
            region: String::new(),
        }];
    }
    Vec::new()
}

pub fn apply_consequence(world: &mut World, consequence: &Consequence) {
    let captain = &mut world.captain;
    if consequence.silver_delta != 0 {
        captain.silver = 0.max(captain.silver + consequence.silver_delta);
    }
    if consequence.trust_delta != 0 {
        captain.standing.commercial_trust =
            0.max(captain.standing.commercial_trust + consequence.trust_delta);
    }
    if consequence.standing_delta != 0 && !consequence.region.is_empty() {
        let current = captain.standing.regional_of(&consequence.region);
        captain.standing.set_regional(
            &consequence.region,
            (-20).max(current + consequence.standing_delta),
        );
    }
    if consequence.heat_delta != 0 && !consequence.region.is_empty() {
        let current = captain.standing.heat_of(&consequence.region);
        captain.standing.set_heat(
            &consequence.region,
            0.max(current + consequence.heat_delta).min(100),
        );
    }
}

pub fn port_effect_note(consequence: &Consequence) -> String {
    match consequence.silver_delta {
        delta if delta > 0 => format!(" (+{delta} silver)"),
        delta if delta < 0 => format!(" ({delta} silver)"),
        _ => String::new(),
    }
}

fn failed_contracts(completed: &[ContractOutcome]) -> Vec<&ContractOutcome> {
    completed
        .iter()
        .filter(|outcome| outcome.outcome_type == "expired" || outcome.outcome_type == "abandoned")
        .collect()
}

fn underworld_of(standing: &Standing, faction_id: &str) -> i64 {
    standing
        .underworld
        .iter()
        .find(|(id, _)| id == faction_id)
        .map(|(_, value)| *value)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::new_game;

    #[test]
    fn fresh_port_can_hear_gossip_from_the_session_rng() {
        let world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        let mut rng = PyRandom::from_seed(1);
        let board = ContractBoard::default();
        let found = check_port_consequences(&world, "porto_novo", &[], 0, &board, &mut rng);
        let mut again = PyRandom::from_seed(1);
        let second = check_port_consequences(&world, "porto_novo", &[], 0, &board, &mut again);
        assert_eq!(
            found.iter().map(|row| row.id.clone()).collect::<Vec<_>>(),
            second.iter().map(|row| row.id.clone()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn trust_delta_lowers_commercial_trust() {
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        let before = world.captain.standing.commercial_trust;
        apply_consequence(
            &mut world,
            &Consequence {
                id: "broken_contract_haunts".to_string(),
                category: "sea".to_string(),
                trigger: String::new(),
                text: String::new(),
                effect_type: "threat".to_string(),
                silver_delta: 0,
                standing_delta: 0,
                heat_delta: 0,
                trust_delta: -1,
                region: String::new(),
            },
        );
        assert_eq!(world.captain.standing.commercial_trust, before - 1);
    }

    fn sample_receipt(port_id: &str, good_id: &str, action: &'static str) -> TradeReceipt {
        TradeReceipt {
            receipt_id: "r1".to_string(),
            captain_name: "Ada".to_string(),
            port_id: port_id.to_string(),
            good_id: good_id.to_string(),
            action,
            quantity: 5,
            unit_price: 10,
            total_price: 50,
            day: 1,
            stock_before: 20,
            stock_after: 15,
        }
    }

    fn sample_failed_outcome(outcome_type: &str, summary: &str) -> ContractOutcome {
        ContractOutcome {
            contract_id: "c1".to_string(),
            outcome_type: outcome_type.to_string(),
            silver_delta: 0,
            trust_delta: -2,
            standing_delta: -1,
            heat_delta: 0,
            completion_day: 5,
            summary: summary.to_string(),
            family: "procurement".to_string(),
            good_id: "grain".to_string(),
            required_quantity: 10,
            delivered_quantity: 0,
            destination_port_id: "porto_novo".to_string(),
            deadline_day: 10,
            reward_silver: 100,
        }
    }

    #[test]
    fn sea_consequences_cover_reward_threat_information_and_veteran_branches() {
        let board = ContractBoard::default();

        // 1. fair_trader_gift
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        world.captain.standing.commercial_trust = 15;
        let mut hit_fair = false;
        for seed in 1..200 {
            let mut rng = PyRandom::from_seed(seed);
            let res = check_sea_consequences(&world, &[], 12, &board, &mut rng);
            if res.iter().any(|c| c.id == "fair_trader_gift") {
                hit_fair = true;
                break;
            }
        }
        assert!(hit_fair);

        // 2. faction_warning
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        world
            .captain
            .standing
            .underworld
            .push(("crimson_tide".to_string(), 0));
        let mut hit_warn = false;
        for seed in 1..200 {
            let mut rng = PyRandom::from_seed(seed);
            let res = check_sea_consequences(&world, &[], 0, &board, &mut rng);
            if res.iter().any(|c| c.id == "faction_warning") {
                hit_warn = true;
                break;
            }
        }
        assert!(hit_warn);

        // 3. spared_pirate_gratitude
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        world.captain.standing.underworld.clear();
        world.captain.duels_won = 1;
        world
            .captain
            .encounters
            .push(crate::model::PirateEncounterRecord {
                captain_id: "scarlet_ana".to_string(),
                faction_id: "crimson_tide".to_string(),
                day: 2,
                region: "Mediterranean".to_string(),
                outcome: "duel_win".to_string(),
            });
        let mut hit_spared = false;
        for seed in 1..200 {
            let mut rng = PyRandom::from_seed(seed);
            let res = check_sea_consequences(&world, &[], 0, &board, &mut rng);
            if res.iter().any(|c| c.id == "spared_pirate_gratitude") {
                hit_spared = true;
                break;
            }
        }
        assert!(hit_spared);

        // 4. nemesis_shadow
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        world.captain.standing.underworld.clear();
        world.nemesis_id = Some("the_butcher".to_string());
        let mut hit_nemesis = false;
        for seed in 1..200 {
            let mut rng = PyRandom::from_seed(seed);
            let res = check_sea_consequences(&world, &[], 0, &board, &mut rng);
            if res.iter().any(|c| c.id == "nemesis_shadow") {
                hit_nemesis = true;
                break;
            }
        }
        assert!(hit_nemesis);

        // 5. trade_intelligence
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        world.captain.standing.underworld.clear();
        if let Some(port) = world.port_mut("porto_novo") {
            for slot in &mut port.market {
                slot.stock_current = 0;
                slot.stock_target = 50;
            }
        }
        let receipts: Vec<TradeReceipt> = (0..22)
            .map(|_| sample_receipt("porto_novo", "grain", "buy"))
            .collect();
        let mut hit_intel = false;
        for seed in 1..200 {
            let mut rng = PyRandom::from_seed(seed);
            let res = check_sea_consequences(&world, &receipts, 0, &board, &mut rng);
            if res.iter().any(|c| c.id == "trade_intelligence") {
                hit_intel = true;
                break;
            }
        }
        assert!(hit_intel);

        // 6. broken_contract_haunts
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        world.captain.standing.underworld.clear();
        let mut failed_board = ContractBoard::default();
        failed_board
            .completed
            .push(sample_failed_outcome("expired", "Missed grain delivery"));
        let mut hit_broken = false;
        for seed in 1..200 {
            let mut rng = PyRandom::from_seed(seed);
            let res = check_sea_consequences(&world, &[], 0, &failed_board, &mut rng);
            if res.iter().any(|c| c.id == "broken_contract_haunts") {
                hit_broken = true;
                break;
            }
        }
        assert!(hit_broken);

        // 7. regional_escort
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        world.captain.standing.underworld.clear();
        world.captain.standing.set_regional("Mediterranean", 20);
        let mut hit_escort = false;
        for seed in 1..200 {
            let mut rng = PyRandom::from_seed(seed);
            let res = check_sea_consequences(&world, &[], 0, &board, &mut rng);
            if res.iter().any(|c| c.id == "regional_escort") {
                hit_escort = true;
                break;
            }
        }
        assert!(hit_escort);

        // 8. contraband_reputation
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        world.captain.standing.underworld.clear();
        world.captain.standing.set_heat("Mediterranean", 15);
        let contra_receipts = vec![
            sample_receipt("porto_novo", "opium", "sell"),
            sample_receipt("porto_novo", "black_powder", "sell"),
            sample_receipt("porto_novo", "stolen_cargo", "sell"),
        ];
        let mut hit_contra = false;
        for seed in 1..200 {
            let mut rng = PyRandom::from_seed(seed);
            let res = check_sea_consequences(&world, &contra_receipts, 0, &board, &mut rng);
            if res.iter().any(|c| c.id == "contraband_reputation") {
                hit_contra = true;
                break;
            }
        }
        assert!(hit_contra);

        // 9. old_crew_memory
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        world.captain.standing.underworld.clear();
        world.captain.day = 120;
        let mut hit_vet = false;
        for seed in 1..200 {
            let mut rng = PyRandom::from_seed(seed);
            let res = check_sea_consequences(&world, &[], 0, &board, &mut rng);
            if res.iter().any(|c| c.id == "old_crew_memory") {
                hit_vet = true;
                break;
            }
        }
        assert!(hit_vet);
    }

    #[test]
    fn port_consequences_and_apply_helpers_cover_all_branches() {
        let board = ContractBoard::default();

        // 1. loyal_customer
        let world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        let loyal_receipts = vec![
            sample_receipt("porto_novo", "grain", "sell"),
            sample_receipt("porto_novo", "grain", "sell"),
            sample_receipt("porto_novo", "grain", "sell"),
            sample_receipt("porto_novo", "grain", "buy"),
            sample_receipt("porto_novo", "grain", "buy"),
        ];
        let mut hit_loyal = false;
        for seed in 1..200 {
            let mut rng = PyRandom::from_seed(seed);
            let res =
                check_port_consequences(&world, "porto_novo", &loyal_receipts, 3, &board, &mut rng);
            if res.iter().any(|c| c.id == "loyal_customer") {
                hit_loyal = true;
                break;
            }
        }
        assert!(hit_loyal);

        // 2. contract_shame
        let mut failed_board = ContractBoard::default();
        failed_board
            .completed
            .push(sample_failed_outcome("abandoned", "Abandoned contract"));
        let one_receipt = vec![sample_receipt("porto_novo", "grain", "buy")];
        let mut hit_shame = false;
        for seed in 1..200 {
            let mut rng = PyRandom::from_seed(seed);
            let res = check_port_consequences(
                &world,
                "porto_novo",
                &one_receipt,
                0,
                &failed_board,
                &mut rng,
            );
            if res.iter().any(|c| c.id == "contract_shame") {
                hit_shame = true;
                break;
            }
        }
        assert!(hit_shame);

        // 3. faction_favor
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        world.captain.standing.underworld = vec![("crimson_tide".to_string(), 30)];
        let mut hit_favor = false;
        for seed in 1..200 {
            let mut rng = PyRandom::from_seed(seed);
            let res = check_port_consequences(&world, "porto_novo", &[], 0, &board, &mut rng);
            if res.iter().any(|c| c.id == "faction_favor") {
                hit_favor = true;
                break;
            }
        }
        assert!(hit_favor);

        // 4. heat_scrutiny
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        world.captain.standing.underworld.clear();
        world.captain.standing.set_heat("Mediterranean", 25);
        let mut hit_heat = false;
        for seed in 1..200 {
            let mut rng = PyRandom::from_seed(seed);
            let res = check_port_consequences(&world, "porto_novo", &[], 0, &board, &mut rng);
            if res.iter().any(|c| c.id == "heat_scrutiny") {
                hit_heat = true;
                break;
            }
        }
        assert!(hit_heat);

        // 5. homecoming
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        world.captain.standing.commercial_trust = 0;
        world.captain.day = 40;
        let mut hit_home = false;
        for seed in 1..200 {
            let mut rng = PyRandom::from_seed(seed);
            let res = check_port_consequences(&world, "porto_novo", &[], 0, &board, &mut rng);
            if res.iter().any(|c| c.id == "homecoming") {
                hit_home = true;
                break;
            }
        }
        assert!(hit_home);

        // 6. apply_consequence + port_effect_note
        let mut world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        let reward = Consequence {
            id: "test_reward".to_string(),
            category: "port".to_string(),
            trigger: String::new(),
            text: String::new(),
            effect_type: "reward".to_string(),
            silver_delta: 25,
            standing_delta: 3,
            heat_delta: 4,
            trust_delta: 2,
            region: "Mediterranean".to_string(),
        };
        apply_consequence(&mut world, &reward);
        assert_eq!(port_effect_note(&reward), " (+25 silver)");
        let penalty = Consequence {
            silver_delta: -10,
            ..reward.clone()
        };
        assert_eq!(port_effect_note(&penalty), " (-10 silver)");
        let neutral = Consequence {
            silver_delta: 0,
            ..reward
        };
        assert_eq!(port_effect_note(&neutral), "");
    }
}
