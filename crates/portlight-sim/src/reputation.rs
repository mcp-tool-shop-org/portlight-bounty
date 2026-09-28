//! Reputation mutations used by trade, inspection, arrival, and the daily tick.
//!
//! Ported from `portlight.engine.reputation` for the calls the scripted slice makes.

use crate::model::Standing;
use crate::util::py_trunc;

const MAX_STAT: i64 = 100;

fn clamp(value: i64) -> i64 {
    value.clamp(0, MAX_STAT)
}

fn suspicion(
    good_category: &str,
    quantity: i64,
    stock_target: i64,
    margin_pct: f64,
    flood_penalty: f64,
    captain_type: &str,
    region_heat: i64,
) -> i64 {
    let mut score = 0;
    if margin_pct > 200.0 {
        score += 4;
    } else if margin_pct > 150.0 {
        score += 3;
    } else if margin_pct > 100.0 {
        score += 2;
    } else if margin_pct > 50.0 {
        score += 1;
    }
    if stock_target > 0 {
        let dump_ratio = quantity as f64 / stock_target as f64;
        if dump_ratio > 0.8 {
            score += 3;
        } else if dump_ratio > 0.5 {
            score += 2;
        } else if dump_ratio > 0.3 {
            score += 1;
        }
    }
    if good_category == "luxury" {
        score += 2;
    } else if good_category == "contraband" {
        score += 4;
    }
    if flood_penalty > 0.3 {
        score += 2;
    } else if flood_penalty > 0.1 {
        score += 1;
    }
    if region_heat >= 25 {
        score += 2;
    } else if region_heat >= 10 {
        score += 1;
    }
    if captain_type == "smuggler" {
        score += 1;
    }
    score
}

#[allow(clippy::too_many_arguments)]
pub fn record_trade_outcome(
    rep: &mut Standing,
    captain_type: &str,
    day: i64,
    port_id: &str,
    region: &str,
    good_id: &str,
    good_category: &str,
    quantity: i64,
    margin_pct: f64,
    stock_target: i64,
    flood_penalty: f64,
) -> i64 {
    let region_heat = rep.heat_of(region);
    let suspicion = suspicion(
        good_category,
        quantity,
        stock_target,
        margin_pct,
        flood_penalty,
        captain_type,
        region_heat,
    );
    let margin_i = py_trunc(margin_pct);
    let mut heat_delta = 0;
    let mut standing_delta = 0;
    let mut trust_delta = 0;
    let desc = if suspicion >= 6 {
        heat_delta = suspicion.min(8);
        standing_delta = -2;
        trust_delta = -1;
        format!("Suspicious {good_id} dump ({margin_i}% margin, {quantity} units)")
    } else if suspicion >= 3 {
        heat_delta = suspicion;
        format!("Aggressive {good_id} sale drew attention ({margin_i}% margin)")
    } else if margin_pct > 20.0 {
        standing_delta = 1;
        trust_delta = 1;
        format!("Profitable {good_id} trade (+{margin_i}% margin)")
    } else {
        format!("Routine {good_id} sale")
    };
    if captain_type == "merchant" && suspicion < 3 && margin_pct > 20.0 {
        trust_delta += 1;
    }
    rep.set_heat(region, clamp(region_heat + heat_delta));
    rep.set_regional(region, clamp(rep.regional_of(region) + standing_delta));
    rep.commercial_trust = clamp(rep.commercial_trust + trust_delta);
    let current = rep.port_value(port_id).unwrap_or(0);
    let bump = standing_delta.max(0) + if suspicion < 3 { 1 } else { 0 };
    rep.set_port(port_id, clamp(current + bump));
    if heat_delta != 0 || standing_delta != 0 || trust_delta != 0 {
        rep.incidents.insert(
            0,
            crate::model::Incident {
                day,
                port_id: port_id.to_string(),
                region: region.to_string(),
                incident_type: "trade".to_string(),
                description: desc,
                heat_delta,
                standing_delta,
                trust_delta,
            },
        );
        if rep.incidents.len() > 20 {
            rep.incidents.truncate(20);
        }
    }
    heat_delta
}

pub fn record_port_arrival(rep: &mut Standing, port_id: &str, region: &str) {
    let current = rep.port_value(port_id).unwrap_or(0);
    rep.set_port(port_id, clamp(current + 1));
    rep.set_regional(region, clamp(rep.regional_of(region) + 1));
    let heat = rep.heat_of(region);
    if heat > 0 {
        let decay = 1.max(heat / 10);
        rep.set_heat(region, clamp(heat - decay));
    }
}

pub fn record_inspection_outcome(
    rep: &mut Standing,
    day: i64,
    port_id: &str,
    region: &str,
    fine_amount: i64,
    cargo_seized: bool,
) {
    let mut heat_delta = 2;
    let mut standing_delta = 0;
    let mut trust_delta = 0;
    let desc = if cargo_seized {
        heat_delta = 5;
        standing_delta = -3;
        trust_delta = -2;
        format!("Cargo seized during inspection (fined {fine_amount} silver)")
    } else if fine_amount > 15 {
        heat_delta = 3;
        trust_delta = -1;
        format!("Heavy inspection fine ({fine_amount} silver)")
    } else {
        format!("Routine inspection ({fine_amount} silver fee)")
    };
    rep.set_heat(region, clamp(rep.heat_of(region) + heat_delta));
    rep.set_regional(region, clamp(rep.regional_of(region) + standing_delta));
    rep.commercial_trust = clamp(rep.commercial_trust + trust_delta);
    rep.incidents.insert(
        0,
        crate::model::Incident {
            day,
            port_id: port_id.to_string(),
            region: region.to_string(),
            incident_type: "inspection".to_string(),
            description: desc,
            heat_delta,
            standing_delta,
            trust_delta,
        },
    );
    if rep.incidents.len() > 20 {
        rep.incidents.truncate(20);
    }
}

pub fn tick_reputation(rep: &mut Standing) {
    for heat in &mut rep.heat {
        if *heat >= 20 {
            *heat -= 2;
        } else if *heat >= 5 {
            *heat -= 1;
        }
    }
}

/// `get_service_modifier`. Higher port standing lowers provision cost.
pub fn service_modifier(rep: &Standing, port_id: &str) -> f64 {
    let standing = rep.port_value(port_id).unwrap_or(0);
    if standing >= 30 {
        0.8
    } else if standing >= 15 {
        0.9
    } else if standing >= 5 {
        0.95
    } else {
        1.0
    }
}

pub fn inspection_modifier(rep: &Standing, region: &str) -> f64 {
    let heat = rep.heat_of(region);
    if heat >= 40 {
        1.8
    } else if heat >= 25 {
        1.4
    } else if heat >= 15 {
        1.2
    } else {
        1.0
    }
}
