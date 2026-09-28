//! Contract board from `portlight.engine.contracts` and `content/contracts.py`.
//!
//! Offers are generated from the embedded templates. Accepting one creates an
//! obligation. A sale at the destination credits delivery when the cargo's
//! source matches. [`complete_contract`] pays a fulfilled obligation.
//! [`tick_contracts`] expires anything past its deadline and drops offers
//! whose acceptance window has closed.
//!
//! The board draw uses `Random(seed + 7919)` and must not advance the session
//! stream. [`crate::session::Session`] saves that stream, draws, and restores it.

use std::collections::HashSet;

use sha2::{Digest, Sha256};

use crate::content::{self, class_rank, ContractDef};
use crate::error::SimError;
use crate::model::{
    ActiveContract, BreachRecord, Captain, Contract, ContractBoard, ContractOutcome, Port, World,
};
use crate::pyrand::PyRandom;
use crate::util::py_round;

const STATUS_ACCEPTED: &str = "accepted";

/// `generate_offers`. `player_ship_rank` `None` allows the Python multi-hop
/// fallback. The session always passes a rank, including `0` with no ship.
pub fn generate_offers(
    world: &World,
    issuer_port_id: &str,
    captain_type: &str,
    player_ship_rank: Option<i64>,
    max_offers: i64,
    rng: &mut PyRandom,
) -> Vec<Contract> {
    let Some(issuer_index) = world
        .ports
        .iter()
        .position(|port| port.id == issuer_port_id)
    else {
        return Vec::new();
    };
    let templates = &content::content().contracts;
    let trust = trust_rank(trust_tier(world.captain.standing.commercial_trust));
    let region = world.ports[issuer_index].region.clone();
    let region_standing = world.captain.standing.regional_of(&region);
    let region_heat = world.captain.standing.heat_of(&region);
    // No infrastructure is ported, so board effects stay at the Python defaults.
    let quality_mult = 1.0;
    let premium_mult = 1.0;
    let lawful_mult = 1.0;
    let luxury_access = 0.0;

    let mut eligible = Vec::new();
    for (index, template) in templates.iter().enumerate() {
        let required_trust = trust_rank(&template.trust_requirement);
        if trust < required_trust {
            continue;
        }
        if template.standing_requirement > 0 && region_standing < template.standing_requirement {
            continue;
        }
        if template
            .heat_ceiling
            .is_some_and(|ceiling| region_heat > ceiling)
        {
            continue;
        }
        let mut weight = if template
            .captain_bias
            .iter()
            .any(|kind| kind == captain_type)
        {
            2.0
        } else {
            1.0
        };
        if template.family == "shortage" && !has_scarcity(world, &template.goods_pool) {
            weight *= 0.2;
        }
        if template.family == "procurement" || template.family == "reputation_charter" {
            weight *= lawful_mult;
        }
        if required_trust >= 2 || template.standing_requirement >= 10 {
            weight *= quality_mult * premium_mult;
        }
        if template.family == "luxury_discreet" && luxury_access > 0.0 {
            weight *= 1.5;
        }
        eligible.push((index, weight));
    }
    if eligible.is_empty() {
        return Vec::new();
    }

    let mut offers = Vec::new();
    let mut used = HashSet::new();
    let mut attempts = 0i64;
    let limit = max_offers.max(0);
    while (offers.len() as i64) < limit && attempts < limit * 3 {
        attempts += 1;
        let mut choices = Vec::new();
        let mut weights = Vec::new();
        for (index, weight) in &eligible {
            if used.contains(&templates[*index].id) {
                continue;
            }
            choices.push(*index);
            weights.push(*weight);
        }
        if choices.is_empty() {
            break;
        }
        let template = &templates[choices[rng.choices_weighted(&weights)]];
        let mut good_id = template.goods_pool[rng.choice_index(template.goods_pool.len())].clone();
        let mut dest = pick_destination(
            template,
            issuer_port_id,
            world,
            Some(good_id.as_str()),
            rng,
            player_ship_rank,
        );
        if dest.is_none() {
            let mut alt_goods: Vec<String> = template
                .goods_pool
                .iter()
                .filter(|good| *good != &good_id)
                .cloned()
                .collect();
            py_shuffle(&mut alt_goods, rng);
            for alt in &alt_goods {
                dest = pick_destination(
                    template,
                    issuer_port_id,
                    world,
                    Some(alt.as_str()),
                    rng,
                    player_ship_rank,
                );
                if dest.is_some() {
                    good_id = alt.clone();
                    break;
                }
            }
            if dest.is_none() {
                used.insert(template.id.clone());
                continue;
            }
        }
        let dest = dest.expect("destination");
        used.insert(template.id.clone());
        let qty = rng.randint(template.quantity_min, template.quantity_max);
        offers.push(make_offer(
            template,
            &world.ports[issuer_index],
            &world.ports[dest],
            &good_id,
            qty,
            world.day,
            offers.len(),
        ));
    }

    if offers.is_empty() {
        for (index, _) in &eligible {
            let template = &templates[*index];
            if used.contains(&template.id) {
                continue;
            }
            for good in &template.goods_pool {
                let mut dest = pick_destination(
                    template,
                    issuer_port_id,
                    world,
                    Some(good.as_str()),
                    rng,
                    player_ship_rank,
                );
                if dest.is_none() {
                    dest = pick_destination(
                        template,
                        issuer_port_id,
                        world,
                        None,
                        rng,
                        player_ship_rank,
                    );
                }
                if let Some(dest) = dest {
                    let qty = rng.randint(template.quantity_min, template.quantity_max);
                    offers.push(make_offer(
                        template,
                        &world.ports[issuer_index],
                        &world.ports[dest],
                        good,
                        qty,
                        world.day,
                        0,
                    ));
                    break;
                }
            }
            if !offers.is_empty() {
                break;
            }
        }
    }
    offers
}

pub fn accept_offer(
    board: &mut ContractBoard,
    offer_id: &str,
    day: i64,
) -> Result<ActiveContract, SimError> {
    let offer = board
        .offers
        .iter()
        .find(|offer| offer.id == offer_id)
        .cloned()
        .ok_or(SimError::OfferNotFound)?;
    if board.active.len() >= 3 {
        return Err(SimError::TooManyContracts);
    }
    let contract = ActiveContract {
        offer_id: offer.id.clone(),
        template_id: offer.template_id,
        family: offer.family,
        title: offer.title,
        accepted_day: day,
        deadline_day: offer.deadline_day,
        destination_port_id: offer.destination_port_id,
        good_id: offer.good_id,
        required_quantity: offer.quantity,
        delivered_quantity: 0,
        reward_silver: offer.reward_silver,
        bonus_reward: offer.bonus_reward,
        source_region: offer.source_region,
        source_port: offer.source_port,
        inspection_modifier: offer.inspection_modifier,
        status: STATUS_ACCEPTED.to_string(),
    };
    board.active.push(contract.clone());
    board.offers.retain(|offer| offer.id != offer_id);
    Ok(contract)
}

/// Credits a sale against active obligations. Returns `(offer_id, credited qty)`.
pub fn check_delivery(
    board: &mut ContractBoard,
    port_id: &str,
    good_id: &str,
    mut quantity: i64,
    source_port: &str,
    source_region: &str,
) -> Vec<(String, i64)> {
    let mut credited = Vec::new();
    for contract in &mut board.active {
        if contract.status != STATUS_ACCEPTED {
            continue;
        }
        if contract.destination_port_id != port_id || contract.good_id != good_id {
            continue;
        }
        let remaining = contract.required_quantity - contract.delivered_quantity;
        if remaining <= 0 {
            continue;
        }
        if let Some(region) = contract.source_region.as_deref() {
            if !region.is_empty() && source_region != region {
                continue;
            }
        }
        if let Some(port) = contract.source_port.as_deref() {
            if !port.is_empty() && source_port != port {
                continue;
            }
        }
        let credit = quantity.min(remaining);
        contract.delivered_quantity += credit;
        quantity -= credit;
        credited.push((contract.offer_id.clone(), credit));
        if quantity <= 0 {
            break;
        }
    }
    credited
}

pub fn resolve_completed(board: &mut ContractBoard, day: i64) -> Vec<ContractOutcome> {
    let active = std::mem::take(&mut board.active);
    let mut still_active = Vec::new();
    let mut outcomes = Vec::new();
    for contract in active {
        if contract.status != STATUS_ACCEPTED {
            still_active.push(contract);
            continue;
        }
        if contract.delivered_quantity >= contract.required_quantity {
            let outcome = completion_outcome(&contract, day);
            board.completed.push(outcome.clone());
            outcomes.push(outcome);
        } else {
            still_active.push(contract);
        }
    }
    board.active = still_active;
    outcomes
}

/// Pay one fulfilled obligation. The session records it on the house books.
pub fn complete_contract(
    board: &mut ContractBoard,
    offer_id: &str,
    day: i64,
) -> Result<ContractOutcome, SimError> {
    let index = board
        .active
        .iter()
        .position(|contract| contract.offer_id == offer_id)
        .ok_or(SimError::NoActiveContract)?;
    if board.active[index].status != STATUS_ACCEPTED {
        return Err(SimError::NoActiveContract);
    }
    if board.active[index].delivered_quantity < board.active[index].required_quantity {
        return Err(SimError::ContractNotFulfilled);
    }
    let contract = board.active.remove(index);
    let outcome = completion_outcome(&contract, day);
    board.completed.push(outcome.clone());
    Ok(outcome)
}

/// Daily deadline check. `captain` is recorded on a breach when it is passed.
/// `GameSession.advance` omits it; the engine still escalates wanted level
/// when a caller supplies the captain.
pub fn tick_contracts(
    board: &mut ContractBoard,
    day: i64,
    captain: Option<&mut Captain>,
) -> Vec<ContractOutcome> {
    let active = std::mem::take(&mut board.active);
    let mut still_active = Vec::new();
    let mut outcomes = Vec::new();
    let mut breaches = Vec::new();
    for contract in active {
        if contract.status != STATUS_ACCEPTED {
            still_active.push(contract);
            continue;
        }
        if day > contract.deadline_day {
            let outcome = expiry_outcome(&contract, day);
            breaches.push((
                contract.offer_id.clone(),
                contract.destination_port_id.clone(),
                contract.family.clone(),
            ));
            board.completed.push(outcome.clone());
            outcomes.push(outcome);
        } else {
            still_active.push(contract);
        }
    }
    board.active = still_active;
    board
        .offers
        .retain(|offer| day <= offer.created_day + offer.acceptance_window);
    if let Some(captain) = captain {
        for (contract_id, port_id, family) in breaches {
            record_breach(board, captain, &contract_id, day, &port_id, &family);
        }
    }
    outcomes
}

pub fn abandon_contract(
    board: &mut ContractBoard,
    offer_id: &str,
    day: i64,
    captain: Option<&mut Captain>,
) -> Result<ContractOutcome, SimError> {
    let index = board
        .active
        .iter()
        .position(|contract| contract.offer_id == offer_id)
        .ok_or(SimError::NoActiveContract)?;
    let contract = board.active.remove(index);
    let outcome = fill_outcome(
        &contract,
        day,
        OutcomeParts {
            kind: "abandoned",
            silver: 0,
            trust: -2,
            standing: -1,
            heat: 1,
            summary: format!("Abandoned contract: {}", contract.title),
        },
    );
    board.completed.push(outcome.clone());
    if let Some(captain) = captain {
        record_breach(
            board,
            captain,
            &contract.offer_id,
            day,
            &contract.destination_port_id,
            &contract.family,
        );
    }
    Ok(outcome)
}

pub fn breach_count_for_family(board: &ContractBoard, family: &str) -> usize {
    board
        .breaches
        .iter()
        .filter(|breach| breach.family == family)
        .count()
}

fn record_breach(
    board: &mut ContractBoard,
    captain: &mut Captain,
    contract_id: &str,
    day: i64,
    port_id: &str,
    family: &str,
) {
    board.breaches.push(BreachRecord {
        contract_id: contract_id.to_string(),
        day,
        port_id: port_id.to_string(),
        family: family.to_string(),
    });
    let count = board.breaches.len();
    if count >= 5 {
        captain.wanted_level = captain.wanted_level.max(3);
    } else if count >= 3 {
        captain.wanted_level = captain.wanted_level.max(2);
    } else if count >= 2 {
        captain.wanted_level = captain.wanted_level.max(1);
    }
}

fn completion_outcome(contract: &ActiveContract, day: i64) -> ContractOutcome {
    let is_early = day < contract.deadline_day - 3;
    let bonus = if is_early && contract.bonus_reward > 0 {
        contract.bonus_reward
    } else {
        0
    };
    let outcome_type = if bonus > 0 {
        "completed_bonus"
    } else {
        "completed"
    };
    let mut summary = format!(
        "Delivered {} {} to {}",
        contract.delivered_quantity, contract.good_id, contract.destination_port_id
    );
    if bonus > 0 {
        summary.push_str(&format!(" (early bonus: +{bonus} silver)"));
    }
    fill_outcome(
        contract,
        day,
        OutcomeParts {
            kind: outcome_type,
            silver: contract.reward_silver + bonus,
            trust: if bonus > 0 { 2 } else { 1 },
            standing: 1,
            heat: -1,
            summary,
        },
    )
}

fn expiry_outcome(contract: &ActiveContract, day: i64) -> ContractOutcome {
    if contract.delivered_quantity > 0 {
        let partial_pct = contract.delivered_quantity as f64 / contract.required_quantity as f64;
        let payout = py_round(contract.reward_silver as f64 * partial_pct * 0.5);
        fill_outcome(
            contract,
            day,
            OutcomeParts {
                kind: "expired",
                silver: payout,
                trust: -2,
                standing: -1,
                heat: 1,
                summary: format!(
                    "Contract expired: delivered {}/{} {} (partial payout: {} silver)",
                    contract.delivered_quantity,
                    contract.required_quantity,
                    contract.good_id,
                    payout
                ),
            },
        )
    } else {
        fill_outcome(
            contract,
            day,
            OutcomeParts {
                kind: "expired",
                silver: 0,
                trust: -3,
                standing: -2,
                heat: 2,
                summary: format!(
                    "Contract defaulted: failed to deliver {} to {}",
                    contract.good_id, contract.destination_port_id
                ),
            },
        )
    }
}

struct OutcomeParts {
    kind: &'static str,
    silver: i64,
    trust: i64,
    standing: i64,
    heat: i64,
    summary: String,
}

fn fill_outcome(contract: &ActiveContract, day: i64, parts: OutcomeParts) -> ContractOutcome {
    ContractOutcome {
        contract_id: contract.offer_id.clone(),
        outcome_type: parts.kind.to_string(),
        silver_delta: parts.silver,
        trust_delta: parts.trust,
        standing_delta: parts.standing,
        heat_delta: parts.heat,
        completion_day: day,
        summary: parts.summary,
        family: contract.family.clone(),
        good_id: contract.good_id.clone(),
        required_quantity: contract.required_quantity,
        delivered_quantity: contract.delivered_quantity,
        destination_port_id: contract.destination_port_id.clone(),
        deadline_day: contract.deadline_day,
        reward_silver: contract.reward_silver,
    }
}

fn make_offer(
    template: &ContractDef,
    issuer: &Port,
    dest: &Port,
    good_id: &str,
    qty: i64,
    day: i64,
    seq: usize,
) -> Contract {
    Contract {
        id: offer_id(&template.id, &issuer.id, day, seq),
        template_id: template.id.clone(),
        family: template.family.clone(),
        title: format_title(&template.title_pattern, good_id, &dest.name, &issuer.name),
        description: template.description.clone(),
        issuer_port_id: issuer.id.clone(),
        destination_port_id: dest.id.clone(),
        good_id: good_id.to_string(),
        quantity: qty,
        created_day: day,
        deadline_day: day + template.deadline_days,
        reward_silver: qty * template.reward_per_unit,
        bonus_reward: template.bonus_reward,
        required_trust_tier: template.trust_requirement.clone(),
        required_standing: template.standing_requirement,
        heat_ceiling: template.heat_ceiling,
        inspection_modifier: template.inspection_modifier,
        source_region: template.source_region.clone(),
        source_port: template.source_port.clone(),
        offer_reason: offer_reason(&template.family, &issuer.name, &dest.name, good_id),
        tags: template.tags.clone(),
        acceptance_window: 10,
    }
}

fn offer_reason(family: &str, issuer_name: &str, dest_name: &str, good_id: &str) -> String {
    match family {
        "procurement" => format!("{dest_name} needs {good_id} delivered"),
        "shortage" => format!("Shortage at {dest_name} — urgent demand for {good_id}"),
        "luxury_discreet" => format!("Discreet buyer at {dest_name} wants {good_id}"),
        "return_freight" => {
            format!("{issuer_name} has {good_id} that needs to reach {dest_name}")
        }
        "circuit" => format!("Trade circuit opportunity: {good_id} to {dest_name}"),
        "reputation_charter" => format!("Premium charter: deliver {good_id} to {dest_name}"),
        _ => format!("Deliver {good_id} to {dest_name}"),
    }
}

fn format_title(pattern: &str, good: &str, destination: &str, source: &str) -> String {
    pattern
        .replace("{good}", good)
        .replace("{destination}", destination)
        .replace("{source}", source)
}

fn offer_id(template_id: &str, port_id: &str, day: i64, seq: usize) -> String {
    let raw = format!("{template_id}:{port_id}:{day}:{seq}");
    let hash = Sha256::digest(raw.as_bytes());
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(12);
    for byte in hash.iter().take(6) {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0xf) as usize] as char);
    }
    out
}

fn pick_destination(
    template: &ContractDef,
    issuer_id: &str,
    world: &World,
    good_id: Option<&str>,
    rng: &mut PyRandom,
    player_ship_rank: Option<i64>,
) -> Option<usize> {
    let mut candidates = Vec::new();
    for (index, port) in world.ports.iter().enumerate() {
        if port.id == issuer_id {
            continue;
        }
        if !template.destination_regions.is_empty()
            && !template
                .destination_regions
                .iter()
                .any(|region| region == &port.region)
        {
            continue;
        }
        if let Some(good_id) = good_id {
            if !port.market.iter().any(|slot| slot.good_id == good_id) {
                continue;
            }
        }
        let direct = world.routes.iter().any(|route| {
            links(route, issuer_id, &port.id) && route_accessible(route, player_ship_rank)
        });
        let reachable = if direct {
            true
        } else if player_ship_rank.is_none() {
            world.routes.iter().any(|route| {
                (route.port_a == port.id || route.port_b == port.id)
                    && route_accessible(route, player_ship_rank)
            })
        } else {
            false
        };
        if reachable {
            candidates.push(index);
        }
    }
    if candidates.is_empty() {
        None
    } else {
        Some(candidates[rng.choice_index(candidates.len())])
    }
}

fn links(route: &crate::model::Route, a: &str, b: &str) -> bool {
    (route.port_a == a && route.port_b == b) || (route.port_a == b && route.port_b == a)
}

fn route_accessible(route: &crate::model::Route, player_ship_rank: Option<i64>) -> bool {
    match player_ship_rank {
        Some(rank) => rank >= class_rank(&route.min_ship_class),
        None => true,
    }
}

fn has_scarcity(world: &World, goods: &[String]) -> bool {
    world.ports.iter().any(|port| {
        port.market.iter().any(|slot| {
            goods.iter().any(|good| good == &slot.good_id)
                && (slot.stock_current as f64) < slot.stock_target as f64 * 0.5
        })
    })
}

fn trust_tier(trust: i64) -> &'static str {
    if trust >= 40 {
        "trusted"
    } else if trust >= 25 {
        "reliable"
    } else if trust >= 10 {
        "credible"
    } else if trust >= 1 {
        "new"
    } else {
        "unproven"
    }
}

fn trust_rank(tier: &str) -> i64 {
    match tier {
        "unproven" => 0,
        "new" => 1,
        "credible" => 2,
        "reliable" => 3,
        "trusted" => 4,
        _ => 0,
    }
}

/// CPython `Random.shuffle` (Fisher-Yates with `_randbelow`).
fn py_shuffle<T>(items: &mut [T], rng: &mut PyRandom) {
    for i in (1..items.len()).rev() {
        let j = rng.randbelow((i as u64) + 1) as usize;
        items.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ship_class_rank;
    use crate::world::new_game;

    fn board_rng(seed: i128) -> PyRandom {
        PyRandom::from_seed(seed + 7919)
    }

    fn offers_for(seed: i128) -> (World, Vec<Contract>) {
        let world = new_game("Ada", "merchant", seed, None).unwrap();
        let rank = ship_class_rank(&world.captain.ship.as_ref().unwrap().template_id);
        let mut rng = board_rng(seed);
        let offers = generate_offers(
            &world,
            &world.voyage.destination_id,
            &world.captain.captain_type,
            Some(rank),
            5,
            &mut rng,
        );
        (world, offers)
    }

    #[test]
    fn shuffle_matches_cpython() {
        let mut rng = PyRandom::from_seed(42);
        let mut items = ["a", "b", "c", "d", "e"];
        py_shuffle(&mut items, &mut rng);
        assert_eq!(items, ["d", "b", "c", "e", "a"]);

        let mut rng = PyRandom::from_seed(1);
        let mut nums = [0, 1, 2, 3, 4, 5, 6, 7];
        py_shuffle(&mut nums, &mut rng);
        assert_eq!(nums, [3, 6, 1, 5, 7, 0, 4, 2]);
    }

    #[test]
    fn merchant_seed_one_matches_python_offers() {
        let (_world, offers) = offers_for(1);
        let ids: Vec<_> = offers.iter().map(|offer| offer.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "63fc3f8be22a",
                "d2c430fdaf99",
                "71773aae754b",
                "dd38287c23aa",
                "0b22b57f35b1",
            ]
        );
        let grain = &offers[2];
        assert_eq!(grain.template_id, "short_grain_famine");
        assert_eq!(grain.family, "shortage");
        assert_eq!(grain.good_id, "grain");
        assert_eq!(grain.quantity, 23);
        assert_eq!(grain.destination_port_id, "corsairs_rest");
        assert_eq!(grain.deadline_day, 19);
        assert_eq!(grain.reward_silver, 552);
        assert_eq!(grain.bonus_reward, 60);
        assert_eq!(grain.title, "Famine relief: grain to Corsair's Rest");
        assert!(grain.offer_reason.contains("Corsair's Rest"));
        assert!(grain.source_region.is_none());
    }

    #[test]
    fn accept_rejects_a_missing_offer_and_a_fourth_contract() {
        let (world, offers) = offers_for(1);
        let mut board = ContractBoard {
            offers,
            last_refresh_day: world.day,
            ..ContractBoard::default()
        };
        assert_eq!(
            accept_offer(&mut board, "missing", world.day).unwrap_err(),
            SimError::OfferNotFound
        );
        for offer in board.offers.clone().into_iter().take(3) {
            accept_offer(&mut board, &offer.id, world.day).unwrap();
        }
        assert_eq!(board.active.len(), 3);
        assert_eq!(board.offers.len(), 2);
        let leftover = board.offers[0].id.clone();
        assert_eq!(
            accept_offer(&mut board, &leftover, world.day).unwrap_err(),
            SimError::TooManyContracts
        );
    }

    #[test]
    fn source_region_blocks_delivery_and_completion_pays_the_early_bonus() {
        let (world, offers) = offers_for(1);
        let spice = offers
            .iter()
            .find(|offer| offer.template_id == "ret_spice_restock")
            .unwrap()
            .clone();
        let mut board = ContractBoard {
            offers: vec![spice.clone()],
            ..ContractBoard::default()
        };
        let accepted = accept_offer(&mut board, &spice.id, world.day).unwrap();
        let credited = check_delivery(
            &mut board,
            &accepted.destination_port_id,
            &accepted.good_id,
            accepted.required_quantity,
            "porto_novo",
            "Mediterranean",
        );
        assert!(credited.is_empty());
        assert_eq!(board.active[0].delivered_quantity, 0);
        assert_eq!(
            complete_contract(&mut board, &spice.id, world.day).unwrap_err(),
            SimError::ContractNotFulfilled
        );

        board.active[0].source_region = None;
        let credited = check_delivery(
            &mut board,
            &accepted.destination_port_id,
            &accepted.good_id,
            accepted.required_quantity,
            "porto_novo",
            "Mediterranean",
        );
        assert_eq!(
            credited,
            vec![(spice.id.clone(), accepted.required_quantity)]
        );
        let outcome = complete_contract(&mut board, &spice.id, 4).unwrap();
        assert_eq!(outcome.outcome_type, "completed_bonus");
        assert_eq!(
            outcome.silver_delta,
            accepted.reward_silver + accepted.bonus_reward
        );
        assert_eq!(outcome.trust_delta, 2);
        assert!(board.active.is_empty());
        assert_eq!(board.completed.len(), 1);
    }

    #[test]
    fn expiry_pays_a_partial_and_escalates_the_second_breach() {
        let mut world = new_game("Ada", "merchant", 1, None).unwrap();
        let mut board = ContractBoard::default();
        board.active.push(sample_contract("c1", 10, 20, 100, 5));
        let outcomes = tick_contracts(&mut board, 6, Some(&mut world.captain));
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].outcome_type, "expired");
        assert_eq!(outcomes[0].silver_delta, 25);
        assert_eq!(outcomes[0].trust_delta, -2);
        assert_eq!(world.captain.wanted_level, 0);
        assert!(board.offers.is_empty());

        board.active.push(sample_contract("c2", 0, 8, 80, 3));
        let outcomes = tick_contracts(&mut board, 4, Some(&mut world.captain));
        assert_eq!(outcomes[0].silver_delta, 0);
        assert_eq!(outcomes[0].trust_delta, -3);
        assert_eq!(outcomes[0].standing_delta, -2);
        assert_eq!(outcomes[0].heat_delta, 2);
        assert_eq!(world.captain.wanted_level, 1);
        assert_eq!(breach_count_for_family(&board, "procurement"), 2);
    }

    #[test]
    fn stale_offers_leave_the_board_on_the_window() {
        let mut board = ContractBoard::default();
        board.offers.push(Contract {
            id: "offer".into(),
            template_id: "proc_grain_feed".into(),
            family: "procurement".into(),
            title: "Grain".into(),
            description: String::new(),
            issuer_port_id: "porto_novo".into(),
            destination_port_id: "al_manar".into(),
            good_id: "grain".into(),
            quantity: 8,
            created_day: 1,
            deadline_day: 26,
            reward_silver: 128,
            bonus_reward: 30,
            required_trust_tier: "unproven".into(),
            required_standing: 0,
            heat_ceiling: None,
            inspection_modifier: 0.0,
            source_region: None,
            source_port: None,
            offer_reason: String::new(),
            tags: Vec::new(),
            acceptance_window: 10,
        });
        tick_contracts(&mut board, 11, None);
        assert_eq!(board.offers.len(), 1);
        tick_contracts(&mut board, 12, None);
        assert!(board.offers.is_empty());
    }

    fn sample_contract(
        id: &str,
        delivered: i64,
        required: i64,
        reward: i64,
        deadline: i64,
    ) -> ActiveContract {
        ActiveContract {
            offer_id: id.into(),
            template_id: "proc_grain_feed".into(),
            family: "procurement".into(),
            title: "Grain".into(),
            accepted_day: 1,
            deadline_day: deadline,
            destination_port_id: "al_manar".into(),
            good_id: "grain".into(),
            required_quantity: required,
            delivered_quantity: delivered,
            reward_silver: reward,
            bonus_reward: 0,
            source_region: None,
            source_port: None,
            inspection_modifier: 0.0,
            status: STATUS_ACCEPTED.into(),
        }
    }
}
