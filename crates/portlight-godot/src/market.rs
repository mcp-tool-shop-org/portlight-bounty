//! Market quantity toggle (T-Q) and the paid-contract notice (T-N).
//!
//! Godot memory only. The qty is never saved and a new game or load starts
//! at `Qty 1`. Every press is clamped here before the `Session::buy` or
//! `Session::sell` call, so the sim never sees a qty it would reject when a
//! smaller one would succeed.

use portlight_sim::model::ContractOutcome;

use crate::contracts_screen;

/// The Qty button cycles these, then wraps.
pub(crate) const TRADE_QTYS: [i64; 3] = [1, 5, 10];

/// Paid lines shown before the `+N more` tail.
const PAID_LINES_MAX: usize = 2;

/// `Qty 1` -> `Qty 5` -> `Qty 10` -> `Qty 1`. An unknown qty restarts at 1.
pub(crate) fn next_trade_qty(qty: i64) -> i64 {
    match TRADE_QTYS.iter().position(|step| *step == qty) {
        Some(index) => TRADE_QTYS[(index + 1) % TRADE_QTYS.len()],
        None => TRADE_QTYS[0],
    }
}

pub(crate) fn qty_label(qty: i64) -> String {
    format!("Qty {qty}")
}

/// What the docked market and the hold allow for one Buy press.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BuyRoom {
    pub stock: i64,
    pub unit_price: i64,
    pub silver: i64,
    /// `ship::resolve_cargo_capacity`, as the sim reads it.
    pub capacity: f64,
    /// `economy::cargo_weight` of the hold before this press.
    pub current_weight: f64,
    pub weight_per_unit: f64,
}

impl BuyRoom {
    /// The sim's own hold test (`economy::execute_buy`), float for float.
    fn hold_fits(&self, units: i64) -> bool {
        self.current_weight + units as f64 * self.weight_per_unit <= self.capacity
    }
}

/// Buy qty clamped to stock, affordable silver and free hold. Can be 0.
pub(crate) fn clamp_buy(qty: i64, room: &BuyRoom) -> i64 {
    let mut units = qty.min(room.stock);
    if room.unit_price > 0 {
        units = units.min(room.silver.div_euclid(room.unit_price));
    }
    if room.weight_per_unit > 0.0 {
        // Small epsilon so 30.0 / 1.0 is not floored to 29 by float noise,
        // then step down until the sim's exact test passes, so float noise
        // can never send a qty the sim rejects as hold full.
        let free = room.capacity - room.current_weight;
        let mut fits = ((free / room.weight_per_unit) + 1e-9).floor() as i64;
        while fits > 0 && !room.hold_fits(fits) {
            fits -= 1;
        }
        units = units.min(fits);
    }
    units.max(0)
}

/// Sell qty clamped to what the hold carries. Can be 0.
pub(crate) fn clamp_sell(qty: i64, held: i64) -> i64 {
    qty.min(held).max(0)
}

/// The qty sent to the sim. A clamp of 0 keeps today's one-unit call, so the
/// sim's own error line still reaches the log.
pub(crate) fn wire_qty(clamped: i64) -> i64 {
    if clamped > 0 {
        clamped
    } else {
        1
    }
}

/// GOLD Market lines for the contracts one sale settled. Max two, then
/// `+N more`. Empty when the sale settled nothing.
pub(crate) fn paid_notice_lines(outcomes: &[ContractOutcome]) -> Vec<String> {
    let mut lines: Vec<String> = outcomes
        .iter()
        .take(PAID_LINES_MAX)
        .map(|outcome| {
            format!(
                "Contract paid: {}{}",
                contracts_screen::outcome_summary(outcome),
                contracts_screen::outcome_terms(outcome)
            )
        })
        .collect();
    if outcomes.len() > PAID_LINES_MAX {
        lines.push(format!("+{} more", outcomes.len() - PAID_LINES_MAX));
    }
    lines
}

/// CI frame `market-contract-paid-more.png`. One real sale settles one
/// contract, so the capture stages two more deliveries to the same port after
/// the real outcome: two lines, then `+1 more`. Same builder as a live sale.
pub(crate) fn smoke_paid_more(paid: &ContractOutcome) -> Vec<ContractOutcome> {
    let staged = |id: &str, good: &str, qty: i64, silver: i64| ContractOutcome {
        contract_id: id.into(),
        silver_delta: silver,
        trust_delta: 1,
        standing_delta: 1,
        summary: format!("Delivered {qty} {good} to {}", paid.destination_port_id),
        good_id: good.into(),
        required_quantity: qty,
        delivered_quantity: qty,
        reward_silver: silver,
        ..paid.clone()
    };
    vec![
        paid.clone(),
        staged("staged-weapons", "weapons", 12, 384),
        staged("staged-rum", "rum", 18, 270),
    ]
}

#[cfg(test)]
mod tests {
    use portlight_sim::model::ContractOutcome;

    use super::{
        clamp_buy, clamp_sell, next_trade_qty, paid_notice_lines, qty_label, smoke_paid_more,
        wire_qty, BuyRoom,
    };

    fn room() -> BuyRoom {
        BuyRoom {
            stock: 40,
            unit_price: 8,
            silver: 550,
            capacity: 30.0,
            current_weight: 0.0,
            weight_per_unit: 1.0,
        }
    }

    #[test]
    fn qty_cycles_one_five_ten_and_wraps() {
        assert_eq!(next_trade_qty(1), 5);
        assert_eq!(next_trade_qty(5), 10);
        assert_eq!(next_trade_qty(10), 1);
        assert_eq!(next_trade_qty(7), 1);
        assert_eq!(qty_label(1), "Qty 1");
        assert_eq!(qty_label(10), "Qty 10");
    }

    #[test]
    fn buy_clamps_to_stock_silver_and_hold() {
        assert_eq!(clamp_buy(10, &room()), 10);
        // Stock.
        assert_eq!(clamp_buy(10, &BuyRoom { stock: 4, ..room() }), 4);
        // Silver: 35 buys four at 8.
        assert_eq!(
            clamp_buy(
                10,
                &BuyRoom {
                    silver: 35,
                    ..room()
                }
            ),
            4
        );
        // Hold: 6.5 free at 1.0 a unit fits six; 3.0 at 1.5 fits two.
        assert_eq!(
            clamp_buy(
                10,
                &BuyRoom {
                    current_weight: 23.5,
                    ..room()
                }
            ),
            6
        );
        assert_eq!(
            clamp_buy(
                10,
                &BuyRoom {
                    current_weight: 27.0,
                    weight_per_unit: 1.5,
                    ..room()
                }
            ),
            2
        );
        // Exactly full after this press is allowed.
        assert_eq!(
            clamp_buy(
                10,
                &BuyRoom {
                    current_weight: 20.0,
                    ..room()
                }
            ),
            10
        );
        // Nothing fits: 0, and the wire falls back to today's one-unit call.
        let broke = BuyRoom {
            silver: 7,
            ..room()
        };
        assert_eq!(clamp_buy(5, &broke), 0);
        assert_eq!(wire_qty(clamp_buy(5, &broke)), 1);
        assert_eq!(
            clamp_buy(
                5,
                &BuyRoom {
                    current_weight: 32.0,
                    ..room()
                }
            ),
            0
        );
        assert_eq!(clamp_buy(5, &BuyRoom { stock: 0, ..room() }), 0);
    }

    /// Float edge: 0.1 aboard, 3.0 capacity, 0.1 a unit. 29 more fits on
    /// paper, but the sim's own test reads 0.1 + 29 * 0.1 as
    /// 3.0000000000000004 > 3.0 and would refuse the press as hold full. The
    /// clamp rounds down to 28, which the sim takes.
    #[test]
    fn hold_clamp_rounds_down_at_the_float_edge() {
        let edge = BuyRoom {
            stock: 100,
            unit_price: 1,
            silver: 1_000,
            capacity: 3.0,
            current_weight: 0.1,
            weight_per_unit: 0.1,
        };
        // The float edge this test pins: the sim's test refuses 29.
        assert!(!edge.hold_fits(29), "the float edge this test pins");
        assert!(edge.hold_fits(28));
        assert_eq!(clamp_buy(100, &edge), 28);
        assert!(edge.hold_fits(clamp_buy(100, &edge)));
        // Whole units on whole weights keep the exact fill.
        let whole = BuyRoom {
            capacity: 30.0,
            current_weight: 20.0,
            weight_per_unit: 1.0,
            ..edge
        };
        assert_eq!(clamp_buy(100, &whole), 10);
    }

    #[test]
    fn sell_clamps_to_held() {
        assert_eq!(clamp_sell(10, 23), 10);
        assert_eq!(clamp_sell(10, 3), 3);
        assert_eq!(clamp_sell(1, 3), 1);
        assert_eq!(clamp_sell(5, 0), 0);
        assert_eq!(wire_qty(clamp_sell(5, 0)), 1);
        assert_eq!(wire_qty(clamp_sell(10, 3)), 3);
    }

    fn paid(id: &str, silver: i64, standing: i64) -> ContractOutcome {
        ContractOutcome {
            contract_id: id.into(),
            outcome_type: "completed".into(),
            silver_delta: silver,
            trust_delta: 1,
            standing_delta: standing,
            heat_delta: -1,
            completion_day: 9,
            summary: "Delivered 23 grain to corsairs_rest".into(),
            family: "shortage".into(),
            good_id: "grain".into(),
            required_quantity: 23,
            delivered_quantity: 23,
            destination_port_id: "corsairs_rest".into(),
            deadline_day: 19,
            reward_silver: silver,
        }
    }

    #[test]
    fn paid_notice_is_exact_copy_with_zero_terms_dropped() {
        assert!(paid_notice_lines(&[]).is_empty());
        assert_eq!(
            paid_notice_lines(&[paid("a", 615, 2)]),
            vec!["Contract paid: Delivered 23 Grain to Corsair's Rest - Silver +615"]
        );
        // Trust, standing and heat ride on the outcome but the sim never
        // applies them, so the notice never prints them.
        for line in paid_notice_lines(&[paid("a", 615, 0)]) {
            for term in ["Trust", "Standing", "Heat"] {
                assert!(!line.contains(term), "{line}");
            }
        }
    }

    #[test]
    fn paid_notice_caps_at_two_lines_plus_more() {
        let two = paid_notice_lines(&[paid("a", 615, 2), paid("b", 400, 1)]);
        assert_eq!(two.len(), 2);
        assert!(two.iter().all(|line| line.starts_with("Contract paid: ")));
        let four = paid_notice_lines(&[
            paid("a", 615, 2),
            paid("b", 400, 1),
            paid("c", 300, 1),
            paid("d", 200, 1),
        ]);
        assert_eq!(four.len(), 3);
        assert_eq!(four[2], "+2 more");
        for line in &four {
            assert!(line.is_ascii(), "{line}");
            assert!(!line.contains("+0"), "{line}");
        }
    }

    #[test]
    fn staged_more_frame_is_two_lines_and_one_more() {
        let lines = paid_notice_lines(&smoke_paid_more(&paid("a", 615, 2)));
        assert_eq!(
            lines,
            vec![
                "Contract paid: Delivered 23 Grain to Corsair's Rest - Silver +615",
                "Contract paid: Delivered 12 Weapons to Corsair's Rest - Silver +384",
                "+1 more",
            ]
        );
    }
}
