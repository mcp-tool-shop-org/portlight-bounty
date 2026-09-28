//! Price formula, market ticks, and buy/sell execution.
//!
//! Mirrors `portlight.engine.economy` for the trade path. Captain price
//! modifiers are optional: `new_game` prices the world without them, and the
//! script runner reapplies them the way `Session._recalc` does after a trade.

use sha2::{Digest, Sha256};

use crate::content::{self, PricingDef};
use crate::error::SimError;
use crate::model::{Captain, CargoItem, Port, CONTRABAND};
use crate::pyrand::PyRandom;
use crate::util::{py_round, py_trunc};

#[derive(Debug, Clone)]
pub struct TradeReceipt {
    pub receipt_id: String,
    pub captain_name: String,
    pub port_id: String,
    pub good_id: String,
    pub action: &'static str,
    pub quantity: i64,
    pub unit_price: i64,
    pub total_price: i64,
    pub day: i64,
    pub stock_before: i64,
    pub stock_after: i64,
}

pub fn recalculate_prices(port: &mut Port, pricing: Option<&PricingDef>) {
    let catalog = content::content();
    for slot in &mut port.market {
        let Some(good) = catalog.good(&slot.good_id) else {
            continue;
        };
        let scarcity = slot.stock_target as f64 / slot.stock_current.max(1) as f64;
        let raw = good.base_price as f64 * scarcity / slot.local_affinity.max(0.1);
        let mut buy_mult = 1.0;
        let mut sell_mult = 1.0;
        if let Some(pricing) = pricing {
            buy_mult = pricing.buy_price_mult;
            sell_mult = pricing.sell_price_mult;
            if pricing.luxury_sell_bonus > 0.0 && good.category == "luxury" {
                sell_mult += pricing.luxury_sell_bonus;
            }
        }
        slot.buy_price = 1.max(py_round(raw * (1.0 + slot.spread / 2.0) * buy_mult));
        let flood_mult = 1.0 - slot.flood_penalty * 0.5;
        slot.sell_price = 1.max(py_round(
            raw * (1.0 - slot.spread / 2.0) * flood_mult * sell_mult,
        ));
    }
}

/// Advance every port market `days` times.
///
/// `current_day == 0` skips seasonal demand, which is what `Session.advance`
/// does: it calls `tick_markets` without `current_day`.
pub fn tick_markets(
    ports: &mut [Port],
    days: i64,
    rng: &mut PyRandom,
    current_day: i64,
) -> Vec<String> {
    let catalog = content::content();
    let mut messages = Vec::new();
    for port in ports.iter_mut() {
        let seasonal = if current_day > 0 {
            catalog.season_profile(&port.region, current_day)
        } else {
            None
        };
        for slot in &mut port.market {
            for _ in 0..days {
                let diff = slot.stock_target - slot.stock_current;
                if diff == 0 {
                    // Already at target.
                } else if (diff.abs() as f64) > slot.restock_rate {
                    let mut pull = slot.restock_rate
                        * (1.0 + diff.abs() as f64 / slot.stock_target.max(1) as f64 * 0.5);
                    if diff < 0 {
                        pull = -pull * 0.5;
                    }
                    slot.stock_current += py_round(pull);
                } else {
                    slot.stock_current += diff;
                }
                if slot.flood_penalty > 0.0 {
                    slot.flood_penalty = 0.0_f64.max(slot.flood_penalty - 0.05);
                }
                if rng.random() < 0.08 {
                    let shock = rng.randint(-4, 4);
                    slot.stock_current = 0.max(slot.stock_current + shock);
                }
                if let Some(profile) = seasonal {
                    if let Some(demand_mult) = profile.market_effects.get(&slot.good_id).copied() {
                        if demand_mult > 1.0 {
                            let drain = py_trunc((demand_mult - 1.0) * slot.restock_rate * 0.5);
                            slot.stock_current = 0.max(slot.stock_current - drain);
                        } else if demand_mult < 1.0 {
                            let surplus = py_trunc((1.0 - demand_mult) * slot.restock_rate * 0.5);
                            slot.stock_current += surplus;
                        }
                    }
                }
            }
        }
        if rng.random() < 0.03 * days as f64 && !port.market.is_empty() {
            let idx = rng.choice_index(port.market.len());
            let direction = if rng.choice_index(2) == 0 { -1 } else { 1 };
            let magnitude = rng.randint(5, 12);
            let good_name = port.market[idx].good_id.clone();
            let port_name = port.name.clone();
            let slot = &mut port.market[idx];
            slot.stock_current = 0.max(slot.stock_current + direction * magnitude);
            if direction > 0 {
                messages.push(format!("Supply glut: {good_name} floods {port_name}"));
            } else {
                messages.push(format!("Shortage: {good_name} scarce at {port_name}"));
            }
        }
    }
    messages
}

pub fn item_weight(good_id: &str, quantity: i64) -> f64 {
    let per = content::content()
        .good(good_id)
        .map(|g| g.weight_per_unit)
        .unwrap_or(1.0);
    quantity as f64 * per
}

pub fn cargo_weight(items: &[CargoItem]) -> f64 {
    items
        .iter()
        .map(|item| item_weight(&item.good_id, item.quantity))
        .sum()
}

pub fn cargo_quantity(items: &[CargoItem], good_id: &str) -> i64 {
    items
        .iter()
        .filter(|item| item.good_id == good_id)
        .map(|item| item.quantity)
        .sum()
}

pub fn consume_cargo_fifo(items: &mut Vec<CargoItem>, good_id: &str, qty: i64) -> Vec<CargoItem> {
    let mut remaining = qty;
    let mut consumed = Vec::new();
    let mut i = 0;
    while remaining > 0 && i < items.len() {
        if items[i].good_id != good_id {
            i += 1;
            continue;
        }
        let take = remaining.min(items[i].quantity);
        let item_cost = items[i].cost_basis;
        let cost_per = if items[i].quantity != 0 {
            item_cost as f64 / items[i].quantity as f64
        } else {
            0.0
        };
        let leftover_qty = items[i].quantity - take;
        let leftover_cost = if leftover_qty != 0 {
            py_round(cost_per * leftover_qty as f64)
        } else {
            0
        };
        let take_cost = item_cost - leftover_cost;
        consumed.push(CargoItem {
            good_id: items[i].good_id.clone(),
            quantity: take,
            cost_basis: take_cost,
            acquired_port: items[i].acquired_port.clone(),
            acquired_region: items[i].acquired_region.clone(),
            acquired_day: items[i].acquired_day,
        });
        items[i].cost_basis = leftover_cost;
        items[i].quantity = leftover_qty;
        remaining -= take;
        if items[i].quantity <= 0 {
            items.remove(i);
        } else {
            i += 1;
        }
    }
    consumed
}

/// `work_docks`. One `randint(3, 5)`, then silver and `captain.day`.
///
/// The session copies `captain.day` onto `world.day` afterwards. This draw
/// does not tick markets, provisions, wages, or reputation.
pub fn work_docks(captain: &mut Captain, rng: &mut PyRandom) -> i64 {
    let earned = rng.randint(3, 5);
    captain.silver += earned;
    captain.day += 1;
    earned
}

fn receipt_id(captain_name: &str, port_id: &str, good_id: &str, day: i64, seq: u64) -> String {
    let raw = format!("{captain_name}:{port_id}:{good_id}:{day}:{seq}");
    let hash = Sha256::digest(raw.as_bytes());
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(16);
    for byte in hash.iter().take(8) {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0xf) as usize] as char);
    }
    out
}

pub fn execute_buy(
    captain: &mut Captain,
    port: &mut Port,
    good_id: &str,
    qty: i64,
    seq: u64,
) -> Result<TradeReceipt, SimError> {
    let Some(slot_idx) = port.market.iter().position(|s| s.good_id == good_id) else {
        let catalog = content::content();
        for slot in &port.market {
            if let Some(good) = catalog.good(&slot.good_id) {
                let normalized = good.name.to_lowercase().replace(' ', "_");
                if normalized == good_id.to_lowercase() {
                    return Err(SimError::GoodNotAvailable {
                        good_id: good_id.to_string(),
                        port_name: port.name.clone(),
                        suggestion: Some(slot.good_id.clone()),
                    });
                }
            }
        }
        return Err(SimError::GoodNotAvailable {
            good_id: good_id.to_string(),
            port_name: port.name.clone(),
            suggestion: None,
        });
    };
    if qty <= 0 {
        return Err(SimError::QuantityMustBePositive);
    }
    if qty > port.market[slot_idx].stock_current {
        let stock = port.market[slot_idx].stock_current;
        return Err(SimError::OnlyStock {
            stock,
            good_id: good_id.to_string(),
        });
    }
    let unit = port.market[slot_idx].buy_price;
    let total = unit * qty;
    if total > captain.silver {
        return Err(SimError::NeedSilver {
            need: total,
            have: captain.silver,
        });
    }
    let Some(ship) = captain.ship.as_ref() else {
        return Err(SimError::NoShip);
    };
    let capacity = crate::ship::resolve_cargo_capacity(ship);
    let current_weight = cargo_weight(&captain.cargo);
    let weight_per = content::content()
        .good(good_id)
        .map(|g| g.weight_per_unit)
        .unwrap_or(1.0);
    if current_weight + qty as f64 * weight_per > capacity as f64 {
        return Err(SimError::NotEnoughCargoSpace);
    }

    let stock_before = port.market[slot_idx].stock_current;
    captain.silver -= total;
    port.market[slot_idx].stock_current -= qty;
    let port_id = port.id.clone();
    let region = port.region.clone();
    if let Some(existing) = captain
        .cargo
        .iter_mut()
        .find(|c| c.good_id == good_id && c.acquired_port == port_id)
    {
        existing.cost_basis += total;
        existing.quantity += qty;
        existing.acquired_day = existing.acquired_day.max(captain.day);
    } else {
        captain.cargo.push(CargoItem {
            good_id: good_id.to_string(),
            quantity: qty,
            cost_basis: total,
            acquired_port: port_id.clone(),
            acquired_region: region,
            acquired_day: captain.day,
        });
    }
    let stock_after = port.market[slot_idx].stock_current;
    Ok(TradeReceipt {
        receipt_id: receipt_id(&captain.name, &port.id, good_id, captain.day, seq),
        captain_name: captain.name.clone(),
        port_id: port.id.clone(),
        good_id: good_id.to_string(),
        action: "buy",
        quantity: qty,
        unit_price: unit,
        total_price: total,
        day: captain.day,
        stock_before,
        stock_after,
    })
}

pub fn execute_sell(
    captain: &mut Captain,
    port: &mut Port,
    good_id: &str,
    qty: i64,
    seq: u64,
) -> Result<TradeReceipt, SimError> {
    if let Some(good) = content::content().good(good_id) {
        if good.category == "contraband" && !port.has_feature("black_market") {
            return Err(SimError::Harbormaster {
                good_id: good_id.to_string(),
            });
        }
    }
    let Some(slot_idx) = port.market.iter().position(|s| s.good_id == good_id) else {
        return Err(SimError::PortDoesNotTrade {
            port_name: port.name.clone(),
            good_id: good_id.to_string(),
        });
    };
    if qty <= 0 {
        return Err(SimError::QuantityMustBePositive);
    }
    let have = cargo_quantity(&captain.cargo, good_id);
    if have < qty {
        return Err(SimError::OnlyHave {
            have,
            good_id: good_id.to_string(),
        });
    }

    let slices = consume_cargo_fifo(&mut captain.cargo, good_id, qty);
    let stock_before = port.market[slot_idx].stock_current;
    let sell_price = port.market[slot_idx].sell_price;
    let mut total = 0i64;
    for slice in &slices {
        let mut unit_price = sell_price;
        let same_port = slice.acquired_port == port.id && (captain.day - slice.acquired_day) <= 3;
        if same_port && slice.quantity > 0 {
            let cost_per_unit = slice.cost_basis as f64 / slice.quantity as f64;
            if (unit_price as f64) > cost_per_unit {
                unit_price = 1.max(py_round(cost_per_unit));
            }
        }
        total += unit_price * slice.quantity;
    }
    captain.silver += total;
    port.market[slot_idx].stock_current += qty;
    let target = port.market[slot_idx].stock_target.max(1) as f64;
    let flood_increase = qty as f64 / target * 0.3;
    port.market[slot_idx].flood_penalty =
        1.0_f64.min(port.market[slot_idx].flood_penalty + flood_increase);
    let unit_price = if qty != 0 {
        py_round(total as f64 / qty as f64)
    } else {
        0
    };
    let stock_after = port.market[slot_idx].stock_current;
    Ok(TradeReceipt {
        receipt_id: receipt_id(&captain.name, &port.id, good_id, captain.day, seq),
        captain_name: captain.name.clone(),
        port_id: port.id.clone(),
        good_id: good_id.to_string(),
        action: "sell",
        quantity: qty,
        unit_price,
        total_price: total,
        day: captain.day,
        stock_before,
        stock_after,
    })
}

/// Session's post-sell cost estimate. Looks at cargo *after* the sale.
pub fn estimate_cost_basis(captain: &Captain, good_id: &str, qty: i64) -> i64 {
    for item in &captain.cargo {
        if item.good_id == good_id && item.quantity > 0 {
            let avg = item.cost_basis as f64 / item.quantity as f64;
            return py_trunc(avg * qty as f64);
        }
    }
    content::content()
        .good(good_id)
        .map(|g| g.base_price * qty)
        .unwrap_or(qty * 10)
}

pub fn is_contraband(good_id: &str) -> bool {
    CONTRABAND.contains(&good_id)
}
