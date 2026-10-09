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

/// Char budget for the one-line paid notice. The Market box is 376 px wide
/// (x874-1249) and the notice is the theme fallback font at 13 px. Measured
/// with that font over every catalog good and port, qty up to 999, silver up
/// to +99999 and `(+9 more)`: the widest 56-char line is 367 px, and 57 chars
/// can reach 375 px. So 56 keeps every notice on one line with room to spare.
pub(crate) const PAID_NOTICE_CHARS: usize = 56;

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

/// The GOLD Market notice for the contracts one sale settled: one line, the
/// latest contract, the rest counted as ` (+N more)`. Empty when the sale
/// settled nothing. AV.1: one line keeps two goods rows in view at 720.
pub(crate) fn paid_notice_lines(outcomes: &[ContractOutcome]) -> Vec<String> {
    let Some(latest) = outcomes.last() else {
        return Vec::new();
    };
    // The bonus is the one thing a player earns by being early, so with a
    // bonus it is the tail; without one the field tail stands.
    let tail = contracts_screen::early_bonus_tail(latest)
        .unwrap_or_else(|| contracts_screen::outcome_tail(latest));
    vec![contracts_screen::outcome_line_with_tail(
        contracts_screen::notice_label(latest),
        latest,
        outcomes.len() - 1,
        PAID_NOTICE_CHARS,
        &tail,
    )]
}

/// CI frame `market-contract-paid-more.png`. One real sale settles one
/// contract, so the capture stages one more delivery to the same port before
/// the real outcome, which stays the latest: `(+1 more)`. Same builder as a
/// live sale.
pub(crate) fn smoke_paid_more(paid: &ContractOutcome) -> Vec<ContractOutcome> {
    let staged = ContractOutcome {
        contract_id: "staged-weapons".into(),
        outcome_type: "completed".into(),
        silver_delta: 384,
        trust_delta: 1,
        standing_delta: 1,
        heat_delta: -1,
        summary: format!("Delivered 12 weapons to {}", paid.destination_port_id),
        good_id: "weapons".into(),
        required_quantity: 12,
        delivered_quantity: 12,
        reward_silver: 384,
        ..paid.clone()
    };
    vec![staged, paid.clone()]
}

#[cfg(test)]
mod tests {
    use portlight_sim::model::ContractOutcome;

    use super::{
        clamp_buy, clamp_sell, next_trade_qty, paid_notice_lines, qty_label, smoke_paid_more,
        wire_qty, BuyRoom, PAID_NOTICE_CHARS,
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
    fn paid_notice_is_one_line_with_silver_first() {
        assert!(paid_notice_lines(&[]).is_empty());
        assert_eq!(
            paid_notice_lines(&[paid("a", 615, 2)]),
            vec!["Contract paid: Silver +615 - 23 Grain to Corsair's Rest"]
        );
        // Trust, standing and heat ride on the outcome but the sim never
        // applies them, so the notice never prints them.
        for line in paid_notice_lines(&[paid("a", 615, 0)]) {
            for term in ["Trust", "Standing", "Heat", "Delivered", "(+"] {
                assert!(!line.contains(term), "{line}");
            }
        }
    }

    /// AV.1: one line total. The latest contract leads; the rest are counted
    /// inline, and the tail is trimmed so the count and silver survive.
    #[test]
    fn paid_notice_counts_the_rest_inline() {
        let two = paid_notice_lines(&[paid("a", 400, 1), paid("b", 615, 2)]);
        assert_eq!(
            two,
            vec!["Contract paid: Silver +615 (+1 more) - 23 Grain to..."]
        );
        let four = paid_notice_lines(&[
            paid("a", 615, 2),
            paid("b", 400, 1),
            paid("c", 300, 1),
            paid("d", 200, 1),
        ]);
        assert_eq!(
            four,
            vec!["Contract paid: Silver +200 (+3 more) - 23 Grain to..."]
        );
        for line in two.iter().chain(&four) {
            assert!(line.is_ascii(), "{line}");
            assert!(!line.contains("+0"), "{line}");
            assert!(line.len() <= PAID_NOTICE_CHARS, "{line}");
            assert!(!line.contains('\n'), "{line}");
        }
    }

    /// A long tail is cut in Rust with ASCII `...`, never the head.
    #[test]
    fn paid_notice_trims_a_long_tail() {
        let mut long = paid("a", 99_999, 1);
        long.good_id = "black_powder".into();
        long.destination_port_id = "typhoon_anchorage".into();
        long.delivered_quantity = 999;
        let many: Vec<_> = (0..10).map(|_| long.clone()).collect();
        assert_eq!(
            paid_notice_lines(&many),
            vec!["Contract paid: Silver +99999 (+9 more) - 999 Black..."]
        );
        assert_eq!(
            paid_notice_lines(&[long]),
            vec!["Contract paid: Silver +99999 - 999 Black Powder to..."]
        );
    }

    #[test]
    fn staged_more_frame_is_one_line_and_one_more() {
        let mut real = paid("a", 612, 2);
        real.outcome_type = "completed_bonus".into();
        real.reward_silver = 552;
        assert_eq!(
            paid_notice_lines(&[real.clone()]),
            vec!["Contract paid: Silver +612 - early bonus +60"]
        );
        assert_eq!(
            paid_notice_lines(&smoke_paid_more(&real)),
            vec!["Contract paid: Silver +612 (+1 more) - early bonus +60"]
        );
    }

    fn bonus_paid() -> ContractOutcome {
        let mut real = paid("a", 612, 2);
        real.outcome_type = "completed_bonus".into();
        real.reward_silver = 552;
        real
    }

    #[test]
    fn paid_notice_with_bonus_keeps_the_bonus() {
        assert_eq!(
            paid_notice_lines(&[bonus_paid()]),
            vec!["Contract paid: Silver +612 - early bonus +60"]
        );
    }

    #[test]
    fn paid_notice_with_bonus_and_more_fits_the_budget() {
        let line = &paid_notice_lines(&smoke_paid_more(&bonus_paid()))[0];
        assert_eq!(
            line,
            "Contract paid: Silver +612 (+1 more) - early bonus +60"
        );
        assert!(line.len() <= PAID_NOTICE_CHARS, "{line}");
        let mut big = bonus_paid();
        big.silver_delta = 99_999;
        big.reward_silver = 90_000;
        let many: Vec<_> = (0..10).map(|_| big.clone()).collect();
        let line = &paid_notice_lines(&many)[0];
        assert!(line.len() <= PAID_NOTICE_CHARS, "{line}");
        assert!(
            line.starts_with("Contract paid: Silver +99999 (+9 more)"),
            "{line}"
        );
    }

    #[test]
    fn paid_notice_without_bonus_is_unchanged() {
        assert_eq!(
            paid_notice_lines(&[paid("a", 615, 2)]),
            vec!["Contract paid: Silver +615 - 23 Grain to Corsair's Rest"]
        );
        // A completed_bonus outcome that paid no extra keeps the field tail.
        let mut flat = bonus_paid();
        flat.silver_delta = flat.reward_silver;
        assert_eq!(
            paid_notice_lines(&[flat]),
            vec!["Contract paid: Silver +552 - 23 Grain to Corsair's Rest"]
        );
    }
}
