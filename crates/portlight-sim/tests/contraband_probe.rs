//! Contraband probe: measurement only, no game change.
//!
//! For every starting captain type it reports (a) whether the starting ship
//! can reach one black-market port from the other over open lanes, (b) the
//! cheapest ship that can, and (c) the day-0 contraband quotes at both black
//! markets with the profit of a stock-limited batch. It then applies the stop
//! rule: if the best realistic contraband profit is under about 3% of the
//! field median (1600 silver) a run, a contraband strategy cannot move any
//! type by a measurable amount.
//!
//! The probe runs with the normal test suite (it takes well under a second).
//! It writes its table to `$PT_S0R_OUT` (default `target/pt-s0r`) as
//! `contraband-probe.md`; the committed copy lives in `docs/playtest/balance/`.

use std::fmt::Write as _;
use std::path::PathBuf;

use portlight_sim::content::{self, class_rank, ship_class_rank};
use portlight_sim::session::Session;

const SEED: i128 = 1001;
const NAME: &str = "Balance";
/// 3% of the field median of 54686, rounded.
const STOP_BELOW: i64 = 1600;
/// The bot keeps a quarter of the haircut away: sale price is 80% of the
/// quote at the far market, because selling lowers the price.
const HAIRCUT_PCT: i64 = 80;
const MAX_UNITS: i64 = 12;
const MIN_NET_PER_UNIT: i64 = 20;
const MIN_RATIO_PCT: i64 = 125;

fn black_markets() -> Vec<String> {
    let mut ids: Vec<String> = content::content()
        .ports
        .iter()
        .filter(|p| p.features.iter().any(|f| f == "black_market"))
        .map(|p| p.id.clone())
        .collect();
    ids.sort();
    ids
}

fn contraband_goods() -> Vec<String> {
    let mut ids: Vec<String> = content::content()
        .goods
        .iter()
        .filter(|g| g.category == "contraband")
        .map(|g| g.id.clone())
        .collect();
    ids.sort();
    ids
}

/// Is `to` reachable from `from` over lanes the ship class can sail without a
/// warning (`allow_warning` false) or without being blocked (true)?
fn reachable(ship_rank: i64, allow_warning: bool, from: &str, to: &str) -> bool {
    let routes = &content::content().routes;
    let mut seen = vec![from.to_string()];
    let mut i = 0;
    while i < seen.len() {
        let here = seen[i].clone();
        i += 1;
        for r in routes {
            let next = if r.port_a == here {
                &r.port_b
            } else if r.port_b == here {
                &r.port_a
            } else {
                continue;
            };
            let gap = class_rank(&r.min_ship_class) - ship_rank;
            let ok = if allow_warning { gap < 2 } else { gap <= 0 };
            if ok && !seen.contains(next) {
                seen.push(next.clone());
            }
        }
    }
    seen.iter().any(|p| p == to)
}

/// Both black markets reachable from each other for this ship id.
fn loopable(ship_id: &str, allow_warning: bool) -> bool {
    let bm = black_markets();
    let rank = ship_class_rank(ship_id);
    bm.len() == 2 && reachable(rank, allow_warning, &bm[0], &bm[1])
}

/// Shortest open-lane distance between the two black markets, if any.
fn open_distance(ship_id: &str) -> Option<i64> {
    let bm = black_markets();
    let rank = ship_class_rank(ship_id);
    let routes = &content::content().routes;
    let mut ids: Vec<String> = content::content()
        .ports
        .iter()
        .map(|p| p.id.clone())
        .collect();
    ids.sort();
    let mut dist: Vec<(String, i64)> = vec![(bm[0].clone(), 0)];
    let mut done: Vec<String> = Vec::new();
    while let Some(best) = dist
        .iter()
        .filter(|(p, _)| !done.contains(p))
        .min_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)))
        .cloned()
    {
        done.push(best.0.clone());
        for r in routes {
            let next = if r.port_a == best.0 {
                &r.port_b
            } else if r.port_b == best.0 {
                &r.port_a
            } else {
                continue;
            };
            if class_rank(&r.min_ship_class) > rank {
                continue;
            }
            let d = best.1 + r.distance;
            match dist.iter_mut().find(|(p, _)| p == next) {
                Some(e) if e.1 <= d => {}
                Some(e) => e.1 = d,
                None => dist.push((next.clone(), d)),
            }
        }
    }
    dist.into_iter().find(|(p, _)| *p == bm[1]).map(|(_, d)| d)
}

fn cheapest_enabling_ship() -> (String, i64, i64, f64, i64) {
    let mut ships: Vec<_> = content::content().ships.iter().collect();
    ships.sort_by(|a, b| a.price.cmp(&b.price).then(a.id.cmp(&b.id)));
    let s = ships
        .into_iter()
        .find(|s| loopable(&s.id, false))
        .expect("some ship makes the pair reachable");
    (s.id.clone(), s.price, s.cargo_capacity, s.speed, s.crew_min)
}

#[derive(Clone)]
struct Batch {
    good: String,
    from: String,
    to: String,
    buy_first: i64,
    sell_there: i64,
    quote: i64,
    stock: i64,
    units: i64,
    cost: i64,
    profit: i64,
}

/// Day-0 batch from `from` to `to` for one type, using the real buy path so
/// each unit lifts the next price, and the far price with the 20% haircut.
fn batch(captain_type: &str, from: &str, to: &str, good: &str, hold: i64) -> Option<Batch> {
    let mut at_from = Session::new(NAME, captain_type, SEED, Some(from)).ok()?;
    let at_to = Session::new(NAME, captain_type, SEED, Some(to)).ok()?;
    let far = at_to
        .world()
        .port(to)?
        .market
        .iter()
        .find(|s| s.good_id == good)?;
    let quote = far.sell_price * HAIRCUT_PCT / 100;
    let slot = at_from
        .world()
        .port(from)?
        .market
        .iter()
        .find(|s| s.good_id == good)?;
    let (stock, first) = (slot.stock_current, slot.buy_price);
    let cap = hold.min(stock - 1).min(MAX_UNITS);
    let (mut units, mut cost) = (0, 0);
    while units < cap {
        let next = at_from
            .world()
            .port(from)?
            .market
            .iter()
            .find(|s| s.good_id == good)?
            .buy_price;
        if quote - next < MIN_NET_PER_UNIT || quote * 100 < MIN_RATIO_PCT * next {
            break;
        }
        if at_from.buy(good, 1).is_err() {
            break;
        }
        units += 1;
        cost += next;
    }
    Some(Batch {
        good: good.to_string(),
        from: from.to_string(),
        to: to.to_string(),
        buy_first: first,
        sell_there: far.sell_price,
        quote,
        stock,
        units,
        cost,
        profit: units * quote - cost,
    })
}

struct Row {
    captain: String,
    ship: String,
    silver: i64,
    ok_loop: bool,
    warn_loop: bool,
    ok_distance: Option<i64>,
    afford: bool,
    best_leg: Batch,
    round_trip: i64,
    batches: Vec<Batch>,
}

fn probe() -> (Vec<Row>, String) {
    let bm = black_markets();
    assert_eq!(bm.len(), 2, "the probe assumes two black markets: {bm:?}");
    let goods = contraband_goods();
    let (eship, eprice, ehold, espeed, ecrew) = cheapest_enabling_ship();
    let mut rows = Vec::new();
    let mut captains: Vec<_> = Session::starting_captains()
        .into_iter()
        .map(|c| c.id)
        .collect();
    captains.sort();
    for ty in captains {
        let s = Session::new(NAME, &ty, SEED, None).unwrap();
        let ship = s.world().captain.ship.as_ref().unwrap().template_id.clone();
        let silver = s.world().captain.silver;
        let mut batches = Vec::new();
        for (a, b) in [(&bm[0], &bm[1]), (&bm[1], &bm[0])] {
            for g in &goods {
                if let Some(x) = batch(&ty, a, b, g, ehold) {
                    batches.push(x);
                }
            }
        }
        let best_of = |from: &str| {
            batches
                .iter()
                .filter(|b| b.from == from)
                .max_by(|a, b| a.profit.cmp(&b.profit).then(b.good.cmp(&a.good)))
                .cloned()
                .unwrap()
        };
        let (b0, b1) = (best_of(&bm[0]), best_of(&bm[1]));
        let best_leg = if b0.profit >= b1.profit {
            b0.clone()
        } else {
            b1.clone()
        };
        rows.push(Row {
            captain: ty,
            ship: ship.clone(),
            silver,
            ok_loop: loopable(&ship, false),
            warn_loop: loopable(&ship, true),
            ok_distance: open_distance(&ship),
            afford: silver * 100 >= 150 * eprice,
            best_leg,
            round_trip: b0.profit.max(0) + b1.profit.max(0),
            batches,
        });
    }
    let enabling =
        format!("{eship} ({eprice} silver, hold {ehold}, speed {espeed}, crew min {ecrew})");
    (rows, enabling)
}

fn report(rows: &[Row], enabling: &str) -> String {
    let bm = black_markets();
    let mut out = String::new();
    let _ = writeln!(out, "# Contraband probe (day 0, seed {SEED})\n");
    let _ = writeln!(out, "Black markets: {} and {}. Cheapest ship that makes the pair reachable over open lanes: {enabling}.\n", bm[0], bm[1]);
    let _ = writeln!(out, "## Reachability and the ship\n");
    let _ = writeln!(out, "| Type | Start ship | Silver | Loop on open lanes | Loop with warning lanes | Open distance | Can afford the enabling ship at 1.5x |");
    let _ = writeln!(out, "|---|---|---:|---|---|---:|---|");
    for r in rows {
        let d = r.ok_distance.map_or("none".to_string(), |d| d.to_string());
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} |",
            r.captain,
            r.ship,
            r.silver,
            yn(r.ok_loop),
            yn(r.warn_loop),
            d,
            yn(r.afford)
        );
    }
    let _ = writeln!(out, "\n## Day-0 quotes and stock-limited batches\n");
    let _ = writeln!(out, "Batch = units bought one at a time (each lifts the next price) while the quote beats the next price by 20 and by 25%, capped by stock minus 1, 12 units and the enabling ship's hold. Quote = 80% of the far market's sell price. Profit = units x quote - cost.\n");
    let _ = writeln!(out, "| Type | From | To | Good | Buy first | Sell there | Quote | Stock | Units | Cost | Profit |");
    let _ = writeln!(out, "|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|");
    for r in rows {
        for b in &r.batches {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                r.captain,
                b.from,
                b.to,
                b.good,
                b.buy_first,
                b.sell_there,
                b.quote,
                b.stock,
                b.units,
                b.cost,
                b.profit
            );
        }
    }
    let _ = writeln!(out, "\n## Best realistic profit and the stop rule\n");
    let _ = writeln!(out, "One round trip is the best batch each way at day-0 stock. Two round trips assumes the stock fully restocks between visits, which the restock rate does not give, so it is an upper bound. Seizure cost is the batch cost plus 90 per unit.\n");
    let _ = writeln!(out, "| Type | Best leg | Round trip | Two round trips (upper bound) | Seizure cost of best leg | Stop (under {STOP_BELOW}) |");
    let _ = writeln!(out, "|---|---|---:|---:|---:|---|");
    let mut all_stop = true;
    for r in rows {
        let two = 2 * r.round_trip;
        let stop = two < STOP_BELOW;
        all_stop &= stop;
        let _ = writeln!(
            out,
            "| {} | {} {}->{} {} | {} | {} | {} | {} |",
            r.captain,
            r.best_leg.good,
            r.best_leg.from,
            r.best_leg.to,
            r.best_leg.profit,
            r.round_trip,
            two,
            r.best_leg.cost + 90 * r.best_leg.units,
            if stop { "STOP" } else { "go" }
        );
    }
    let _ = writeln!(
        out,
        "\nVerdict: {}",
        if all_stop {
            format!("STOP. Even the two-round-trip upper bound is under {STOP_BELOW} silver a run for every type, so a contraband strategy cannot move any type by a measurable amount.")
        } else {
            "At least one type clears the stop rule; read the table before building the strategy."
                .to_string()
        }
    );
    out
}

fn yn(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "no"
    }
}

#[test]
fn contraband_probe_reachability_and_table() {
    let (rows, enabling) = probe();
    let text = report(&rows, &enabling);
    let dir = std::env::var_os("PT_S0R_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/pt-s0r"));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("contraband-probe.md"), &text).unwrap();
    println!("{text}");

    // Reachability facts are content only, so they hold in both modes. A
    // content change that opens a lane fails here and is noticed.
    assert_eq!(rows.len(), 9);
    for r in &rows {
        assert!(!r.ok_loop, "{}: {} can already loop", r.captain, r.ship);
    }
    // Only the two cutter starters reach both markets, and only over warning
    // lanes (the long way round).
    let warned: Vec<_> = rows
        .iter()
        .filter(|r| r.warn_loop)
        .map(|r| r.captain.as_str())
        .collect();
    assert_eq!(warned, ["bounty_hunter", "privateer"]);
    assert!(
        rows.iter().all(|r| !r.afford),
        "a type can afford the enabling ship at day 0"
    );
    assert!(enabling.contains("brigantine"), "{enabling}");
}
