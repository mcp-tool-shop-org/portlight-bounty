//! Brokers, warehouses, dry dock, licenses, credit, and insurance.
//!
//! This is `portlight.engine.infrastructure` plus `GameSession.dry_dock`.
//! Money uses Python `int()` (`util::py_trunc`). The engine does not draw
//! the session RNG. Warehouse and policy ids are SHA-256 prefixes, matching
//! `_warehouse_id` and `_policy_id`.

use sha2::{Digest, Sha256};

use crate::content::{
    self, BrokerOfficeDef, CreditTierDef, LicenseDef, PolicyDef, WarehouseTierDef,
};
use crate::economy::{self, cargo_weight, item_weight};
use crate::error::SimError;
use crate::model::{
    ActivePolicy, BrokerOffice, Captain, CreditState, InfrastructureRecord, InsuranceClaim,
    OwnedLicense, Standing, StoredLot, WarehouseLease,
};
use crate::reputation::service_modifier;
use crate::ship::resolve_cargo_capacity;
use crate::util::py_trunc;

const TRUST_RANK: &[(&str, i64)] = &[
    ("unproven", 0),
    ("new", 1),
    ("credible", 2),
    ("reliable", 3),
    ("trusted", 4),
];

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
    TRUST_RANK
        .iter()
        .find(|(name, _)| *name == tier)
        .map(|(_, rank)| *rank)
        .unwrap_or(0)
}

fn broker_rank(tier: &str) -> i64 {
    match tier {
        "local" => 1,
        "established" => 2,
        _ => 0,
    }
}

fn credit_rank(tier: &str) -> i64 {
    match tier {
        "merchant_line" => 1,
        "house_credit" => 2,
        "premier_commercial" => 3,
        _ => 0,
    }
}

fn short_hash(raw: &str) -> String {
    let digest = Sha256::digest(raw.as_bytes());
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(12);
    for byte in digest.iter().take(6) {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0xf) as usize] as char);
    }
    out
}

fn warehouse_id(port_id: &str, tier: &str, day: i64) -> String {
    short_hash(&format!("wh:{port_id}:{tier}:{day}"))
}

fn policy_id(spec_id: &str, day: i64, seq: usize) -> String {
    short_hash(&format!("pol:{spec_id}:{day}:{seq}"))
}

fn reject(message: impl Into<String>) -> SimError {
    SimError::Rejected(message.into())
}

/// Python's default `str(float)` for the capacity errors: `20.0`, not `20`.
fn py_float(value: f64) -> String {
    if !value.is_finite() {
        return format!("{value}");
    }
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        let text = format!("{value}");
        if text.contains('.') {
            text
        } else {
            format!("{value:.1}")
        }
    }
}

pub fn active_warehouse<'a>(
    state: &'a InfrastructureRecord,
    port_id: &str,
) -> Option<&'a WarehouseLease> {
    state
        .warehouses
        .iter()
        .find(|lease| lease.port_id == port_id && lease.active)
}

fn active_warehouse_mut<'a>(
    state: &'a mut InfrastructureRecord,
    port_id: &str,
) -> Option<&'a mut WarehouseLease> {
    state
        .warehouses
        .iter_mut()
        .find(|lease| lease.port_id == port_id && lease.active)
}

fn lot_weight(lots: &[StoredLot]) -> f64 {
    lots.iter()
        .map(|lot| item_weight(&lot.good_id, lot.quantity))
        .sum()
}

fn free_capacity(lease: &WarehouseLease) -> f64 {
    (lease.capacity as f64 - lot_weight(&lease.inventory)).max(0.0)
}

/// `lease_warehouse`. The caller checks that the port offers `tier`.
pub fn lease_warehouse(
    state: &mut InfrastructureRecord,
    captain: &mut Captain,
    port_id: &str,
    spec: &WarehouseTierDef,
    day: i64,
) -> Result<usize, SimError> {
    let existing = state
        .warehouses
        .iter()
        .position(|lease| lease.port_id == port_id && lease.active);
    if let Some(index) = existing {
        if state.warehouses[index].tier == spec.tier {
            return Err(reject(format!("Already have a {} at this port", spec.name)));
        }
    }
    if spec.lease_cost > captain.silver {
        return Err(reject(format!(
            "Need {} silver to lease {}, have {}",
            spec.lease_cost, spec.name, captain.silver
        )));
    }
    let old_inventory = existing
        .map(|index| state.warehouses[index].inventory.clone())
        .unwrap_or_default();
    let old_total = lot_weight(&old_inventory);
    if old_total > spec.capacity as f64 {
        return Err(reject(format!(
            "Cannot downgrade: {} units in storage exceeds {} capacity ({}). Withdraw goods first.",
            py_float(old_total),
            spec.name,
            spec.capacity
        )));
    }
    if let Some(index) = existing {
        state.warehouses[index].active = false;
    }
    captain.silver -= spec.lease_cost;
    state.warehouses.push(WarehouseLease {
        id: warehouse_id(port_id, &spec.tier, day),
        port_id: port_id.to_string(),
        tier: spec.tier.clone(),
        capacity: spec.capacity,
        lease_cost: spec.lease_cost,
        upkeep_per_day: spec.upkeep_per_day,
        inventory: old_inventory,
        opened_day: day,
        upkeep_paid_through: day,
        active: true,
    });
    Ok(state.warehouses.len() - 1)
}

/// `deposit_cargo`. Returns the quantity moved.
pub fn deposit_cargo(
    state: &mut InfrastructureRecord,
    port_id: &str,
    captain: &mut Captain,
    good_id: &str,
    quantity: i64,
    day: i64,
) -> Result<i64, SimError> {
    let Some(warehouse) = active_warehouse(state, port_id) else {
        return Err(reject("No warehouse at this port"));
    };
    if quantity <= 0 {
        return Err(reject("Quantity must be positive"));
    }
    let have = economy::cargo_quantity(&captain.cargo, good_id);
    if have < quantity {
        return Err(reject(format!(
            "Only have {have} units of {good_id} in hold"
        )));
    }
    let added = item_weight(good_id, quantity);
    let free = free_capacity(warehouse);
    if added > free {
        return Err(reject(format!(
            "Warehouse only has {} units of space",
            py_float(free)
        )));
    }
    let slices = economy::consume_cargo_fifo(&mut captain.cargo, good_id, quantity);
    let warehouse = active_warehouse_mut(state, port_id).expect("warehouse");
    for slice in slices {
        let merged = warehouse.inventory.iter_mut().find(|lot| {
            lot.good_id == good_id
                && lot.acquired_port == slice.acquired_port
                && lot.acquired_region == slice.acquired_region
        });
        if let Some(lot) = merged {
            lot.quantity += slice.quantity;
            lot.acquired_day = lot.acquired_day.max(slice.acquired_day);
        } else {
            warehouse.inventory.push(StoredLot {
                good_id: good_id.to_string(),
                quantity: slice.quantity,
                acquired_port: slice.acquired_port,
                acquired_region: slice.acquired_region,
                acquired_day: slice.acquired_day,
                deposited_day: day,
            });
        }
    }
    Ok(quantity)
}

/// `withdraw_cargo`. `source_port` limits lots to that provenance.
pub fn withdraw_cargo(
    state: &mut InfrastructureRecord,
    port_id: &str,
    captain: &mut Captain,
    good_id: &str,
    quantity: i64,
    source_port: Option<&str>,
) -> Result<i64, SimError> {
    let Some(warehouse) = active_warehouse(state, port_id) else {
        return Err(reject("No warehouse at this port"));
    };
    if quantity <= 0 {
        return Err(reject("Quantity must be positive"));
    }
    let Some(ship) = captain.ship.as_ref() else {
        return Err(SimError::NoShip);
    };
    let free_space = resolve_cargo_capacity(ship) as f64 - cargo_weight(&captain.cargo);
    if item_weight(good_id, quantity) > free_space {
        return Err(reject(format!(
            "Ship only has {} units of cargo space",
            py_float(free_space)
        )));
    }
    let available: i64 = warehouse
        .inventory
        .iter()
        .filter(|lot| {
            lot.good_id == good_id && source_port.is_none_or(|port| lot.acquired_port == port)
        })
        .map(|lot| lot.quantity)
        .sum();
    if available < quantity {
        let suffix = source_port
            .map(|port| format!(" from {port}"))
            .unwrap_or_default();
        return Err(reject(format!(
            "Only {available} units of {good_id} in warehouse{suffix}"
        )));
    }
    let mut remaining = quantity;
    let warehouse = active_warehouse_mut(state, port_id).expect("warehouse");
    let mut transfers = Vec::new();
    for lot in warehouse.inventory.iter_mut() {
        if remaining <= 0 {
            break;
        }
        if lot.good_id != good_id || source_port.is_some_and(|port| lot.acquired_port != port) {
            continue;
        }
        let take = lot.quantity.min(remaining);
        lot.quantity -= take;
        remaining -= take;
        transfers.push((take, lot.clone()));
    }
    warehouse.inventory.retain(|lot| lot.quantity > 0);
    for (take, lot) in transfers {
        if let Some(existing) = captain
            .cargo
            .iter_mut()
            .find(|item| item.good_id == good_id && item.acquired_port == lot.acquired_port)
        {
            existing.quantity += take;
        } else {
            captain.cargo.push(crate::model::CargoItem {
                good_id: good_id.to_string(),
                quantity: take,
                cost_basis: 0,
                acquired_port: lot.acquired_port,
                acquired_region: lot.acquired_region,
                acquired_day: lot.acquired_day,
            });
        }
    }
    Ok(quantity)
}

fn pay_upkeep(silver: &mut i64, paid_through: &mut i64, day: i64, upkeep: i64) -> i64 {
    let days_owed = day - *paid_through;
    if days_owed <= 0 {
        return 0;
    }
    let cost = days_owed * upkeep;
    if *silver >= cost {
        *silver -= cost;
        *paid_through = day;
        return 0;
    }
    let affordable = if upkeep > 0 { *silver / upkeep } else { 0 };
    if affordable > 0 {
        *silver -= affordable * upkeep;
        *paid_through += affordable;
    }
    day - *paid_through
}

/// `tick_infrastructure`. Closes a warehouse after 3 unpaid days, a broker
/// or license after 5.
pub fn tick_infrastructure(
    state: &mut InfrastructureRecord,
    captain: &mut Captain,
    day: i64,
) -> Vec<String> {
    let mut messages = Vec::new();
    for warehouse in &mut state.warehouses {
        if !warehouse.active {
            continue;
        }
        let unpaid = pay_upkeep(
            &mut captain.silver,
            &mut warehouse.upkeep_paid_through,
            day,
            warehouse.upkeep_per_day,
        );
        if unpaid >= 3 {
            warehouse.active = false;
            let lost: Vec<(String, i64)> = warehouse
                .inventory
                .iter()
                .map(|lot| (lot.good_id.clone(), lot.quantity))
                .collect();
            warehouse.inventory.clear();
            if lost.is_empty() {
                messages.push(format!(
                    "Warehouse at {} closed for non-payment.",
                    warehouse.port_id
                ));
            } else {
                let goods = lost
                    .iter()
                    .map(|(good, qty)| format!("{qty}x {good}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                messages.push(format!(
                    "Warehouse at {} closed for non-payment. Goods seized: {goods}",
                    warehouse.port_id
                ));
            }
        }
    }
    for broker in &mut state.brokers {
        if !broker.active || broker.tier == "none" {
            continue;
        }
        let Some(spec) = content::content().broker(&broker.region, &broker.tier) else {
            continue;
        };
        let upkeep = spec.upkeep_per_day;
        let unpaid = pay_upkeep(
            &mut captain.silver,
            &mut broker.upkeep_paid_through,
            day,
            upkeep,
        );
        if unpaid >= 5 {
            broker.active = false;
            messages.push(format!(
                "Broker office in {} closed for non-payment.",
                broker.region
            ));
        }
    }
    for license in &mut state.licenses {
        if !license.active {
            continue;
        }
        let Some(spec) = content::content().license(&license.license_id) else {
            continue;
        };
        let upkeep = spec.upkeep_per_day;
        let unpaid = pay_upkeep(
            &mut captain.silver,
            &mut license.upkeep_paid_through,
            day,
            upkeep,
        );
        if unpaid >= 5 {
            license.active = false;
            messages.push(format!(
                "License '{}' revoked for non-payment.",
                license.license_id
            ));
        }
    }
    messages
}

pub fn active_broker<'a>(
    state: &'a InfrastructureRecord,
    region: &str,
) -> Option<&'a BrokerOffice> {
    state
        .brokers
        .iter()
        .find(|broker| broker.region == region && broker.active && broker.tier != "none")
}

pub fn broker_tier<'a>(state: &'a InfrastructureRecord, region: &str) -> &'a str {
    active_broker(state, region)
        .map(|broker| broker.tier.as_str())
        .unwrap_or("none")
}

/// `open_broker_office`.
pub fn open_broker_office(
    state: &mut InfrastructureRecord,
    captain: &mut Captain,
    region: &str,
    spec: &BrokerOfficeDef,
    day: i64,
) -> Result<(), SimError> {
    let existing = state
        .brokers
        .iter()
        .position(|broker| broker.region == region && broker.active && broker.tier != "none");
    if let Some(index) = existing {
        let current = &state.brokers[index];
        if current.tier == spec.tier {
            return Err(reject(format!("Already have a {} in {region}", spec.name)));
        }
        if current.tier == "established" && spec.tier == "local" {
            return Err(reject("Cannot downgrade a broker office"));
        }
    }
    if spec.purchase_cost > captain.silver {
        return Err(reject(format!(
            "Need {} silver to open {}, have {}",
            spec.purchase_cost, spec.name, captain.silver
        )));
    }
    captain.silver -= spec.purchase_cost;
    if let Some(index) = existing {
        let office = &mut state.brokers[index];
        office.tier = spec.tier.clone();
        office.opened_day = day;
        office.upkeep_paid_through = day;
    } else {
        state.brokers.push(BrokerOffice {
            region: region.to_string(),
            tier: spec.tier.clone(),
            opened_day: day,
            upkeep_paid_through: day,
            active: true,
        });
    }
    Ok(())
}

pub fn has_license(state: &InfrastructureRecord, license_id: &str) -> bool {
    state
        .licenses
        .iter()
        .any(|license| license.active && license.license_id == license_id)
}

/// `check_license_eligibility`. `Ok(())` means the player may buy it.
///
/// A global license (`region_scope` is `None`) skips the heat ceiling. Python
/// only reads heat when `region_scope` is set.
pub fn check_license_eligibility(
    state: &InfrastructureRecord,
    spec: &LicenseDef,
    rep: &Standing,
) -> Result<(), SimError> {
    if has_license(state, &spec.id) {
        return Err(reject("Already own this license"));
    }
    let player = trust_tier(rep.commercial_trust);
    if trust_rank(player) < trust_rank(&spec.required_trust_tier) {
        return Err(reject(format!(
            "Requires {} trust (currently {player})",
            spec.required_trust_tier
        )));
    }
    if let Some(region) = spec.region_scope.as_deref() {
        if spec.required_standing > 0 {
            let standing = rep.regional_of(region);
            if standing < spec.required_standing {
                return Err(reject(format!(
                    "Requires {} standing in {region} (currently {standing})",
                    spec.required_standing
                )));
            }
        }
        if let Some(max_heat) = spec.required_heat_max {
            let heat = rep.heat_of(region);
            if heat > max_heat {
                return Err(reject(format!(
                    "Heat too high in {region}: {heat} (max {max_heat})"
                )));
            }
        }
    }
    if let Some(required) = spec.required_broker_tier.as_deref() {
        let needed = broker_rank(required);
        let ok = if let Some(region) = spec.region_scope.as_deref() {
            broker_rank(broker_tier(state, region)) >= needed
        } else {
            state
                .brokers
                .iter()
                .any(|broker| broker.active && broker_rank(&broker.tier) >= needed)
        };
        if !ok {
            if let Some(region) = spec.region_scope.as_deref() {
                return Err(reject(format!(
                    "Requires {required} broker office in {region}"
                )));
            }
            return Err(reject(format!(
                "Requires {required} broker office in at least one region"
            )));
        }
    }
    Ok(())
}

/// `purchase_license`.
pub fn purchase_license(
    state: &mut InfrastructureRecord,
    captain: &mut Captain,
    spec: &LicenseDef,
    rep: &Standing,
    day: i64,
) -> Result<(), SimError> {
    check_license_eligibility(state, spec, rep)?;
    if spec.purchase_cost > captain.silver {
        return Err(SimError::NeedSilver {
            need: spec.purchase_cost,
            have: captain.silver,
        });
    }
    captain.silver -= spec.purchase_cost;
    state.licenses.push(OwnedLicense {
        license_id: spec.id.clone(),
        purchased_day: day,
        upkeep_paid_through: day,
        active: true,
    });
    Ok(())
}

/// Aggregate board modifiers for `region`. License multipliers stack.
/// `customs_mult` and the other numeric keys multiply. `luxury_access` is a flag.
pub fn board_effects(state: &InfrastructureRecord, region: &str) -> BoardEffects {
    let mut effects = BoardEffects::default();
    if let Some(broker) = active_broker(state, region) {
        if let Some(spec) = content::content().broker(region, &broker.tier) {
            effects.board_quality_bonus = spec.board_quality_bonus;
            effects.market_signal_bonus = spec.market_signal_bonus;
            effects.trade_term_modifier = spec.trade_term_modifier;
        }
    }
    let catalog = content::content();
    for owned in &state.licenses {
        if !owned.active {
            continue;
        }
        let Some(spec) = catalog.license(&owned.license_id) else {
            continue;
        };
        if spec
            .region_scope
            .as_ref()
            .is_some_and(|scope| scope != region)
        {
            continue;
        }
        for (key, value) in &spec.effects {
            match key.as_str() {
                "customs_mult" => effects.customs_mult *= value,
                "luxury_access" => effects.luxury_access = effects.luxury_access.max(*value),
                "premium_offer_mult" => effects.premium_offer_mult *= value,
                "lawful_board_mult" => effects.lawful_board_mult *= value,
                "board_quality_bonus" => effects.board_quality_bonus *= value,
                "market_signal_bonus" => effects.market_signal_bonus *= value,
                "trade_term_modifier" => effects.trade_term_modifier *= value,
                _ => {}
            }
        }
    }
    effects
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoardEffects {
    pub board_quality_bonus: f64,
    pub market_signal_bonus: f64,
    pub trade_term_modifier: f64,
    pub premium_offer_mult: f64,
    pub customs_mult: f64,
    pub lawful_board_mult: f64,
    pub luxury_access: f64,
}

impl Default for BoardEffects {
    fn default() -> Self {
        Self {
            board_quality_bonus: 1.0,
            market_signal_bonus: 0.0,
            trade_term_modifier: 1.0,
            premium_offer_mult: 1.0,
            customs_mult: 1.0,
            lawful_board_mult: 1.0,
            luxury_access: 0.0,
        }
    }
}

/// `purchase_policy`. Premium is `int(premium * (1 + heat * heat_premium_mult))`.
#[allow(clippy::too_many_arguments)]
pub fn purchase_policy(
    state: &mut InfrastructureRecord,
    captain: &mut Captain,
    spec: &PolicyDef,
    day: i64,
    heat: i64,
    target_id: &str,
    voyage_origin: &str,
    voyage_destination: &str,
) -> Result<(), SimError> {
    if spec.heat_max.is_some_and(|max| heat > max) {
        return Err(reject(format!(
            "Heat too high ({heat}) for {} — max {}",
            spec.name,
            spec.heat_max.unwrap_or(0)
        )));
    }
    let heat_surcharge = (heat.max(0) as f64) * spec.heat_premium_mult;
    let adjusted = py_trunc(spec.premium as f64 * (1.0 + heat_surcharge));
    if adjusted > captain.silver {
        return Err(reject(format!(
            "Need {adjusted} silver for {}, have {}",
            spec.name, captain.silver
        )));
    }
    if state
        .policies
        .iter()
        .any(|policy| policy.active && policy.spec_id == spec.id && policy.target_id == target_id)
    {
        return Err(reject(format!("Already have active {}", spec.name)));
    }
    captain.silver -= adjusted;
    let seq = state.policies.len();
    state.policies.push(ActivePolicy {
        id: policy_id(&spec.id, day, seq),
        spec_id: spec.id.clone(),
        family: spec.family.clone(),
        scope: spec.scope.clone(),
        purchased_day: day,
        coverage_pct: spec.coverage_pct,
        coverage_cap: spec.coverage_cap,
        premium_paid: adjusted,
        target_id: target_id.to_string(),
        claims_made: 0,
        total_paid_out: 0,
        active: true,
        voyage_origin: voyage_origin.to_string(),
        voyage_destination: voyage_destination.to_string(),
    });
    Ok(())
}

/// `resolve_claim`. Pays the captain and records every matching policy.
#[allow(clippy::too_many_arguments)]
pub fn resolve_claim(
    state: &mut InfrastructureRecord,
    captain: &mut Captain,
    incident_type: &str,
    loss_value: i64,
    day: i64,
    cargo_category: &str,
    contract_id: &str,
    voyage_destination: &str,
) -> Vec<InsuranceClaim> {
    let mut claims = Vec::new();
    let catalog = content::content();
    for policy in &mut state.policies {
        if !policy.active {
            continue;
        }
        let family_ok = match policy.family.as_str() {
            "hull" => matches!(incident_type, "storm" | "pirates"),
            "premium_cargo" => matches!(incident_type, "pirates" | "storm" | "inspection"),
            "contract_guarantee" => incident_type == "contract_failure",
            _ => false,
        };
        if !family_ok {
            continue;
        }
        if policy.scope == "named_contract"
            && (contract_id.is_empty() || policy.target_id != contract_id)
        {
            continue;
        }
        if policy.scope == "next_voyage"
            && !voyage_destination.is_empty()
            && !policy.voyage_destination.is_empty()
            && policy.voyage_destination != voyage_destination
        {
            continue;
        }
        let Some(spec) = catalog.policy(&policy.spec_id) else {
            continue;
        };
        if !spec.covered_risks.iter().any(|risk| risk == incident_type) {
            continue;
        }
        if spec.exclusions.iter().any(|item| item == "contraband") && cargo_category == "contraband"
        {
            claims.push(InsuranceClaim {
                policy_id: policy.id.clone(),
                day,
                incident_type: incident_type.to_string(),
                loss_value,
                payout: 0,
                denied: true,
                denial_reason: "Contraband cargo excluded from coverage".to_string(),
            });
            continue;
        }
        let raw = py_trunc(loss_value as f64 * policy.coverage_pct);
        let remaining = policy.coverage_cap - policy.total_paid_out;
        let payout = raw.min(remaining).max(0);
        if payout > 0 {
            captain.silver += payout;
            policy.claims_made += 1;
            policy.total_paid_out += payout;
        }
        let claim = InsuranceClaim {
            policy_id: policy.id.clone(),
            day,
            incident_type: incident_type.to_string(),
            loss_value,
            payout,
            denied: false,
            denial_reason: String::new(),
        };
        claims.push(claim);
    }
    state.claims.extend(claims.iter().cloned());
    claims
}

/// `expire_voyage_policies`. The note names the policy by its catalog name
/// (`Voyage policy expired: Basic Hull Insurance`); a spec missing from the
/// catalog falls back to the spec id. Python prints the spec id; this is a
/// listed parity divergence.
pub fn expire_voyage_policies(state: &mut InfrastructureRecord) -> Vec<String> {
    let mut messages = Vec::new();
    for policy in &mut state.policies {
        if policy.active && policy.scope == "next_voyage" {
            policy.active = false;
            let name = content::content()
                .policy(&policy.spec_id)
                .map(|spec| spec.name.as_str())
                .filter(|name| !name.is_empty())
                .unwrap_or(policy.spec_id.as_str());
            messages.push(format!("Voyage policy expired: {name}"));
        }
    }
    messages
}

fn ensure_credit(state: &mut InfrastructureRecord) -> &mut CreditState {
    if state.credit.is_none() {
        state.credit = Some(CreditState::default());
    }
    state.credit.as_mut().expect("credit")
}

/// `check_credit_eligibility`.
pub fn check_credit_eligibility(
    state: &mut InfrastructureRecord,
    spec: &CreditTierDef,
    rep: &Standing,
) -> Result<(), SimError> {
    let player = trust_tier(rep.commercial_trust);
    if trust_rank(player) < trust_rank(&spec.required_trust_tier) {
        return Err(reject(format!(
            "Requires {} trust (currently {player})",
            spec.required_trust_tier
        )));
    }
    if spec.required_standing > 0 {
        let best = rep.regional.iter().copied().max().unwrap_or(0);
        if best < spec.required_standing {
            return Err(reject(format!(
                "Requires {} standing in any region (best: {best})",
                spec.required_standing
            )));
        }
    }
    if let Some(max_heat) = spec.required_heat_max {
        let lowest = rep.heat.iter().copied().min().unwrap_or(0);
        if lowest > max_heat {
            return Err(reject(format!(
                "Heat too high (lowest: {lowest}, max: {max_heat})"
            )));
        }
    }
    if let Some(license) = spec.required_license.as_deref() {
        if !has_license(state, license) {
            return Err(reject(format!("Requires license: {license}")));
        }
    }
    let defaults = state
        .credit
        .as_ref()
        .map(|credit| credit.defaults)
        .unwrap_or(0);
    if defaults >= 3 {
        return Err(reject("Too many past defaults — credit locked"));
    }
    if defaults >= 1 && spec.tier == "premier_commercial" {
        return Err(reject("Premier credit unavailable with default history"));
    }
    Ok(())
}

/// `open_credit_line`.
pub fn open_credit_line(
    state: &mut InfrastructureRecord,
    spec: &CreditTierDef,
    rep: &Standing,
    day: i64,
) -> Result<(), SimError> {
    check_credit_eligibility(state, spec, rep)?;
    let current_rank = state
        .credit
        .as_ref()
        .map(|credit| credit_rank(&credit.tier))
        .unwrap_or(0);
    let active = state.credit.as_ref().is_some_and(|credit| credit.active);
    if active && current_rank >= credit_rank(&spec.tier) {
        let tier = state
            .credit
            .as_ref()
            .map(|credit| credit.tier.clone())
            .unwrap_or_else(|| "none".to_string());
        return Err(reject(format!("Already have {tier} or better")));
    }
    let credit = ensure_credit(state);
    credit.tier = spec.tier.clone();
    credit.credit_limit = spec.credit_limit;
    credit.active = true;
    if credit.last_interest_day == 0 {
        credit.last_interest_day = day;
    }
    if credit.next_due_day == 0 {
        credit.next_due_day = day + spec.interest_period;
    }
    Ok(())
}

/// `draw_credit`.
pub fn draw_credit(
    state: &mut InfrastructureRecord,
    captain: &mut Captain,
    amount: i64,
) -> Result<(), SimError> {
    let credit = ensure_credit(state);
    if !credit.active {
        return Err(reject("No credit line established"));
    }
    if amount <= 0 {
        return Err(reject("Amount must be positive"));
    }
    let available = credit.credit_limit - credit.outstanding;
    if amount > available {
        return Err(reject(format!(
            "Only {available} silver available on credit line (limit {}, outstanding {})",
            credit.credit_limit, credit.outstanding
        )));
    }
    credit.outstanding += amount;
    credit.total_borrowed += amount;
    captain.silver += amount;
    Ok(())
}

/// `repay_credit`. Interest is paid before principal.
pub fn repay_credit(
    state: &mut InfrastructureRecord,
    captain: &mut Captain,
    amount: i64,
) -> Result<(), SimError> {
    let credit = ensure_credit(state);
    if !credit.active {
        return Err(reject("No credit line"));
    }
    if amount <= 0 {
        return Err(reject("Amount must be positive"));
    }
    let total_owed = credit.outstanding + credit.interest_accrued;
    if total_owed == 0 {
        return Err(reject("No outstanding debt"));
    }
    let mut amount = amount.min(total_owed);
    if amount > captain.silver {
        return Err(reject(format!(
            "Need {amount} silver to repay, have {}",
            captain.silver
        )));
    }
    captain.silver -= amount;
    credit.total_repaid += amount;
    if credit.interest_accrued > 0 {
        let interest_payment = amount.min(credit.interest_accrued);
        credit.interest_accrued -= interest_payment;
        amount -= interest_payment;
    }
    if amount > 0 {
        credit.outstanding -= amount;
    }
    Ok(())
}

/// `tick_credit`. A default message contains `DEFAULT` so the session can
/// cut commercial trust by 15.
pub fn tick_credit(
    state: &mut InfrastructureRecord,
    captain: &mut Captain,
    day: i64,
) -> Vec<String> {
    let credit = ensure_credit(state);
    if !credit.active || credit.outstanding == 0 {
        return Vec::new();
    }
    let tier = credit.tier.clone();
    let Some(spec) = content::content().credit_tier(&tier) else {
        return Vec::new();
    };
    let mut messages = Vec::new();
    let days_since = day - credit.last_interest_day;
    if days_since >= spec.interest_period {
        let periods = days_since / spec.interest_period;
        let interest = py_trunc(credit.outstanding as f64 * spec.interest_rate * periods as f64);
        credit.interest_accrued += interest;
        credit.last_interest_day = day;
        if interest > 0 {
            messages.push(format!(
                "Interest accrued: {interest} silver on {} debt",
                credit.outstanding
            ));
        }
    }
    let total_owed = credit.outstanding + credit.interest_accrued;
    if day >= credit.next_due_day && credit.next_due_day > 0 && total_owed > 0 {
        let min_payment = credit.interest_accrued + 1.max(credit.outstanding / 10);
        if captain.silver >= min_payment {
            captain.silver -= min_payment;
            credit.total_repaid += min_payment;
            let interest_part = min_payment.min(credit.interest_accrued);
            credit.interest_accrued -= interest_part;
            let remaining = min_payment - interest_part;
            credit.outstanding -= remaining;
            credit.next_due_day = day + spec.interest_period;
            messages.push(format!(
                "Credit payment due: {min_payment} silver auto-deducted"
            ));
        } else {
            credit.defaults += 1;
            messages.push(format!(
                "CREDIT DEFAULT! Cannot pay {min_payment} silver. Default #{} recorded — trust damage applied.",
                credit.defaults
            ));
            if captain.silver > 0 {
                let mut partial = captain.silver;
                captain.silver = 0;
                credit.total_repaid += partial;
                if credit.interest_accrued > 0 {
                    let interest_part = partial.min(credit.interest_accrued);
                    credit.interest_accrued -= interest_part;
                    partial -= interest_part;
                }
                credit.outstanding -= partial;
            }
            credit.next_due_day = day + spec.interest_period;
            if credit.defaults >= 3 {
                credit.active = false;
                messages.push("Credit line frozen after 3 defaults.".to_string());
            }
        }
    }
    messages
}

/// `emergency_loan`. The player receives `amount` and owes `amount + interest`
/// as a deferred fee. Interest is `max(1, int(amount * 0.15))`.
pub fn emergency_loan(captain: &mut Captain, amount: i64) -> Result<i64, SimError> {
    if amount <= 0 {
        return Err(reject("Amount must be positive"));
    }
    if amount > 200 {
        return Err(reject("Emergency loans capped at 200 silver"));
    }
    let interest = 1.max(py_trunc(amount as f64 * 0.15));
    let total_debt = amount + interest;
    captain.silver += amount;
    captain.deferred_fees.push(crate::model::DeferredFee {
        fee_type: "emergency_loan".to_string(),
        amount: total_debt,
        day: captain.day,
    });
    Ok(amount)
}

/// `GameSession._do_dry_dock` for the captain's current ship.
///
/// Restores `hull_max` to the template. Cost per point is
/// `max(1, int(repair_cost * service_modifier * 5))`.
pub fn dry_dock(
    captain: &mut Captain,
    repair_cost: i64,
    service: f64,
) -> Result<(i64, i64), SimError> {
    let Some(ship) = captain.ship.as_ref() else {
        return Err(SimError::NoShip);
    };
    let template_id = ship.template_id.clone();
    let hull_max = ship.hull_max;
    let hull = ship.hull;
    let Some(template_hull) = content::content()
        .ship(&template_id)
        .map(|ship| ship.hull_max)
    else {
        return Err(SimError::UnknownShip(template_id));
    };
    let degradation = template_hull - hull_max;
    if degradation <= 0 {
        return Err(reject("Ship hull is not degraded"));
    }
    let cost_per = 1.max(py_trunc(repair_cost as f64 * service * 5.0));
    let cost = degradation * cost_per;
    if cost > captain.silver {
        return Err(reject(format!(
            "Need {cost} silver for dry dock ({degradation} points at {cost_per}/point), have {}",
            captain.silver
        )));
    }
    captain.silver -= cost;
    let ship = captain.ship.as_mut().expect("ship");
    ship.hull_max = template_hull;
    ship.hull = (hull + degradation).min(ship.hull_max);
    Ok((degradation, cost))
}

/// Service multiplier used by dry dock, from port standing.
pub fn dry_dock_service(rep: &Standing, port_id: &str) -> f64 {
    service_modifier(rep, port_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::new_game;

    fn merchant() -> crate::model::World {
        new_game("Ada", "merchant", 1, Some("porto_novo")).expect("game")
    }

    #[test]
    fn warehouse_id_matches_sha256_prefix() {
        assert_eq!(warehouse_id("porto_novo", "depot", 0), "09ce21fff2f3");
        assert_eq!(policy_id("hull_basic", 0, 0), "f8412649437a");
    }

    #[test]
    fn lease_depot_and_charge_upkeep_on_later_days() {
        let mut world = merchant();
        let mut infra = InfrastructureRecord::default();
        let spec = content::content().warehouse_tier("depot").unwrap().clone();
        let before = world.captain.silver;
        lease_warehouse(&mut infra, &mut world.captain, "porto_novo", &spec, 0).unwrap();
        assert_eq!(world.captain.silver, before - 50);
        assert!(tick_infrastructure(&mut infra, &mut world.captain, 0).is_empty());
        assert_eq!(infra.warehouses[0].upkeep_paid_through, 0);
        assert!(tick_infrastructure(&mut infra, &mut world.captain, 1).is_empty());
        assert_eq!(world.captain.silver, before - 51);
        assert_eq!(infra.warehouses[0].upkeep_paid_through, 1);
    }

    #[test]
    fn unpaid_warehouse_seizes_goods_after_three_days() {
        let mut world = merchant();
        let mut infra = InfrastructureRecord::default();
        let spec = content::content().warehouse_tier("depot").unwrap().clone();
        lease_warehouse(&mut infra, &mut world.captain, "porto_novo", &spec, 0).unwrap();
        world.captain.cargo.push(crate::model::CargoItem {
            good_id: "grain".to_string(),
            quantity: 4,
            cost_basis: 10,
            acquired_port: "porto_novo".to_string(),
            acquired_region: "Mediterranean".to_string(),
            acquired_day: 0,
        });
        deposit_cargo(&mut infra, "porto_novo", &mut world.captain, "grain", 4, 0).unwrap();
        world.captain.silver = 0;
        assert!(tick_infrastructure(&mut infra, &mut world.captain, 1).is_empty());
        assert!(tick_infrastructure(&mut infra, &mut world.captain, 2).is_empty());
        let messages = tick_infrastructure(&mut infra, &mut world.captain, 3);
        assert_eq!(
            messages,
            vec!["Warehouse at porto_novo closed for non-payment. Goods seized: 4x grain"]
        );
        assert!(!infra.warehouses[0].active);
        assert!(infra.warehouses[0].inventory.is_empty());
    }

    #[test]
    fn broker_upgrade_replaces_tier_in_place() {
        let mut world = merchant();
        let mut infra = InfrastructureRecord::default();
        let local = content::content()
            .broker("Mediterranean", "local")
            .unwrap()
            .clone();
        let house = content::content()
            .broker("Mediterranean", "established")
            .unwrap()
            .clone();
        open_broker_office(&mut infra, &mut world.captain, "Mediterranean", &local, 0).unwrap();
        open_broker_office(&mut infra, &mut world.captain, "Mediterranean", &house, 2).unwrap();
        assert_eq!(infra.brokers.len(), 1);
        assert_eq!(infra.brokers[0].tier, "established");
        assert_eq!(infra.brokers[0].opened_day, 2);
        let err = open_broker_office(&mut infra, &mut world.captain, "Mediterranean", &local, 3);
        assert_eq!(
            err.unwrap_err().to_string(),
            "Cannot downgrade a broker office"
        );
    }

    #[test]
    fn merchant_can_draw_the_merchant_line() {
        let mut world = merchant();
        let mut infra = InfrastructureRecord::default();
        let spec = content::content()
            .credit_tier("merchant_line")
            .unwrap()
            .clone();
        let before = world.captain.silver;
        open_credit_line(&mut infra, &spec, &world.captain.standing, 0).unwrap();
        draw_credit(&mut infra, &mut world.captain, 100).unwrap();
        assert_eq!(world.captain.silver, before + 100);
        assert_eq!(infra.credit.as_ref().unwrap().outstanding, 100);
        assert_eq!(infra.credit.as_ref().unwrap().total_borrowed, 100);
        assert_eq!(infra.credit.as_ref().unwrap().next_due_day, 10);
    }

    #[test]
    fn interest_and_minimum_payment_match_python_int() {
        let mut world = merchant();
        let mut infra = InfrastructureRecord::default();
        let spec = content::content()
            .credit_tier("merchant_line")
            .unwrap()
            .clone();
        open_credit_line(&mut infra, &spec, &world.captain.standing, 0).unwrap();
        draw_credit(&mut infra, &mut world.captain, 100).unwrap();
        let messages = tick_credit(&mut infra, &mut world.captain, 10);
        assert_eq!(
            messages,
            vec![
                "Interest accrued: 8 silver on 100 debt".to_string(),
                "Credit payment due: 18 silver auto-deducted".to_string(),
            ]
        );
        let credit = infra.credit.as_ref().unwrap();
        assert_eq!(credit.outstanding, 90);
        assert_eq!(credit.interest_accrued, 0);
        assert_eq!(credit.next_due_day, 20);
        assert_eq!(credit.total_repaid, 18);
    }

    #[test]
    fn hull_policy_pays_half_of_a_storm_loss() {
        let mut world = merchant();
        let mut infra = InfrastructureRecord::default();
        let spec = content::content().policy("hull_basic").unwrap().clone();
        let before = world.captain.silver;
        purchase_policy(
            &mut infra,
            &mut world.captain,
            &spec,
            0,
            0,
            "",
            "porto_novo",
            "al_manar",
        )
        .unwrap();
        assert_eq!(world.captain.silver, before - 40);
        assert_eq!(infra.policies[0].id, "f8412649437a");
        let claims = resolve_claim(
            &mut infra,
            &mut world.captain,
            "storm",
            80,
            3,
            "",
            "",
            "al_manar",
        );
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].payout, 40);
        assert_eq!(world.captain.silver, before - 40 + 40);
        let expired = expire_voyage_policies(&mut infra);
        assert_eq!(expired, vec!["Voyage policy expired: Basic Hull Insurance"]);
        assert!(!infra.policies[0].active);
    }

    #[test]
    fn contraband_cargo_claim_is_denied() {
        let mut world = merchant();
        let mut infra = InfrastructureRecord::default();
        let spec = content::content().policy("cargo_standard").unwrap().clone();
        purchase_policy(&mut infra, &mut world.captain, &spec, 0, 0, "", "", "").unwrap();
        let claims = resolve_claim(
            &mut infra,
            &mut world.captain,
            "inspection",
            200,
            1,
            "contraband",
            "",
            "",
        );
        assert!(claims[0].denied);
        assert_eq!(claims[0].payout, 0);
        assert_eq!(
            claims[0].denial_reason,
            "Contraband cargo excluded from coverage"
        );
    }

    #[test]
    fn dry_dock_restores_template_hull_at_five_times_repair() {
        let mut world = merchant();
        let repair = world.port("porto_novo").unwrap().repair_cost;
        let template = {
            let ship = world.captain.ship.as_mut().unwrap();
            ship.hull_max -= 4;
            ship.hull = ship.hull.min(ship.hull_max);
            content::content().ship(&ship.template_id).unwrap().hull_max
        };
        let before = world.captain.silver;
        let (points, cost) = dry_dock(&mut world.captain, repair, 1.0).unwrap();
        assert_eq!(points, 4);
        assert_eq!(cost, 4 * 1.max(py_trunc(repair as f64 * 5.0)));
        assert_eq!(world.captain.silver, before - cost);
        assert_eq!(world.captain.ship.as_ref().unwrap().hull_max, template);
    }

    #[test]
    fn deposit_and_withdraw_keep_provenance() {
        let mut world = merchant();
        let mut infra = InfrastructureRecord::default();
        let spec = content::content().warehouse_tier("depot").unwrap().clone();
        lease_warehouse(&mut infra, &mut world.captain, "porto_novo", &spec, 0).unwrap();
        world.captain.cargo.push(crate::model::CargoItem {
            good_id: "grain".to_string(),
            quantity: 6,
            cost_basis: 30,
            acquired_port: "al_manar".to_string(),
            acquired_region: "Mediterranean".to_string(),
            acquired_day: 2,
        });
        deposit_cargo(&mut infra, "porto_novo", &mut world.captain, "grain", 6, 3).unwrap();
        assert!(world.captain.cargo.is_empty());
        withdraw_cargo(
            &mut infra,
            "porto_novo",
            &mut world.captain,
            "grain",
            2,
            None,
        )
        .unwrap();
        let item = &world.captain.cargo[0];
        assert_eq!(item.quantity, 2);
        assert_eq!(item.acquired_port, "al_manar");
        assert_eq!(item.cost_basis, 0);
        assert_eq!(infra.warehouses[0].inventory[0].quantity, 4);
    }
}
