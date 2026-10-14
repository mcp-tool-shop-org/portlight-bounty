//! First-voyage hints: one chart line that names the next verb.
//!
//! Hints are rows in [`FIRST_VOYAGE_HINTS`]. The active hint is the first
//! row, in table order, whose `show` holds and whose `done` does not, so at
//! most one hint is ever up. Callers never match on a hint id: adding or
//! removing a hint is a table edit plus its test.

/// Longest hint, in characters: the width of the chart note it replaces.
pub(crate) const HINT_MAX_CHARS: usize = 52;

/// Node name of the chart note label that carries the hint.
pub(crate) const HINT_NODE: &str = "ChartHint";

/// The muted chart note shown when no hint is active.
pub(crate) const CHART_NOTE: &str = "Lanes from the sail picker. Days use raw ship speed.";

/// What the table reads, built once per refresh. Nothing here is saved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HintFacts {
    pub docked: bool,
    pub active: usize,
    pub settled: usize,
    pub wins: i64,
    pub sailed_with_contract: bool,
    pub journal_seen: bool,
}

pub(crate) struct HintSpec {
    pub id: &'static str,
    pub text: &'static str,
    pub show: fn(&HintFacts) -> bool,
    pub done: fn(&HintFacts) -> bool,
}

pub(crate) const FIRST_VOYAGE_HINTS: &[HintSpec] = &[
    HintSpec {
        id: "first_contract",
        text: "Open Contracts for a starter job.",
        show: |f| f.docked && f.active == 0 && f.settled == 0,
        done: |f| f.active > 0 || f.settled > 0,
    },
    HintSpec {
        id: "first_sale",
        text: "Load the goods, sail there, then sell to get paid.",
        show: |f| f.active > 0 && f.settled == 0,
        done: |f| f.sailed_with_contract || f.settled > 0,
    },
    HintSpec {
        id: "journal_paths",
        text: "Open Journal to see your victory paths.",
        show: |f| f.wins > 0,
        done: |f| f.journal_seen,
    },
];

/// A personal fight's result counts as a win for the journal hint only when
/// the player won outright; a draw does not (the sim counts it in
/// `duels_won`, so the counters are not read for live play).
pub(crate) fn duel_is_a_win(player_won: bool, draw: bool) -> bool {
    player_won && !draw
}

/// The active hint for these facts, if any.
pub(crate) fn active_hint(facts: &HintFacts) -> Option<&'static HintSpec> {
    FIRST_VOYAGE_HINTS
        .iter()
        .find(|hint| (hint.show)(facts) && !(hint.done)(facts))
}

/// The table row whose text is exactly `text` (the bridge reads the label).
pub(crate) fn hint_by_text(text: &str) -> Option<&'static HintSpec> {
    FIRST_VOYAGE_HINTS.iter().find(|hint| hint.text == text)
}

/// Unsaved flags seeded from a loaded save, so a veteran save never shows
/// old hints: `(sailed_with_contract, journal_seen)`.
pub(crate) fn seed_from_load(settled: usize, at_sea: bool, wins: i64) -> (bool, bool) {
    (settled > 0 || at_sea, wins > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> HintFacts {
        HintFacts {
            docked: true,
            ..HintFacts::default()
        }
    }

    fn id(f: &HintFacts) -> Option<&'static str> {
        active_hint(f).map(|hint| hint.id)
    }

    #[test]
    fn table_ids_are_unique() {
        for (at, hint) in FIRST_VOYAGE_HINTS.iter().enumerate() {
            assert!(
                FIRST_VOYAGE_HINTS[at + 1..]
                    .iter()
                    .all(|other| other.id != hint.id),
                "duplicate hint id {}",
                hint.id
            );
        }
        assert_eq!(FIRST_VOYAGE_HINTS.len(), 3);
    }

    #[test]
    fn every_hint_is_ascii_one_line_with_a_full_stop() {
        for hint in FIRST_VOYAGE_HINTS {
            let text = hint.text;
            assert!(text.is_ascii(), "{text}");
            assert!(text.len() <= HINT_MAX_CHARS, "{text} is {}", text.len());
            assert!(text.ends_with('.'), "{text}");
            assert!(!text.contains('\n'), "{text}");
            assert!(!text.contains('\u{2014}'), "{text}");
            assert!(!text.contains(" -> "), "{text}");
        }
        assert_eq!(CHART_NOTE.len(), HINT_MAX_CHARS);
    }

    #[test]
    fn at_most_one_hint() {
        for docked in [false, true] {
            for active in 0..3 {
                for settled in 0..3 {
                    for wins in 0..3 {
                        for sailed_with_contract in [false, true] {
                            for journal_seen in [false, true] {
                                let f = HintFacts {
                                    docked,
                                    active,
                                    settled,
                                    wins,
                                    sailed_with_contract,
                                    journal_seen,
                                };
                                let live = FIRST_VOYAGE_HINTS
                                    .iter()
                                    .filter(|hint| (hint.show)(&f) && !(hint.done)(&f))
                                    .collect::<Vec<_>>();
                                // The first live row wins; the pick is unique.
                                assert_eq!(
                                    active_hint(&f).map(|hint| hint.id),
                                    live.first().map(|hint| hint.id)
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn first_contract_then_first_sale_then_none() {
        let mut f = facts();
        assert_eq!(id(&f), Some("first_contract"));
        f.active = 1;
        assert_eq!(id(&f), Some("first_sale"));
        f.docked = false;
        f.sailed_with_contract = true;
        assert_eq!(id(&f), None);
        f.docked = true;
        assert_eq!(id(&f), None);
        f.active = 0;
        f.settled = 1;
        assert_eq!(id(&f), None);
    }

    #[test]
    fn journal_hint_waits_for_a_win_and_clears_on_open() {
        let mut f = HintFacts {
            docked: false,
            ..HintFacts::default()
        };
        assert_eq!(id(&f), None);
        f.wins = 1;
        assert_eq!(id(&f), Some("journal_paths"));
        f.journal_seen = true;
        assert_eq!(id(&f), None);
    }

    #[test]
    fn a_draw_or_a_break_away_shows_no_hint() {
        // Draw: no outright win, so the won-fight flag stays down.
        assert!(!duel_is_a_win(false, true));
        // Break-away: neither won nor drawn, nothing sunk.
        assert!(!duel_is_a_win(false, false));
        assert!(duel_is_a_win(true, false));
        for won in [duel_is_a_win(false, true), duel_is_a_win(false, false)] {
            let f = HintFacts {
                docked: false,
                wins: i64::from(won),
                ..HintFacts::default()
            };
            assert_eq!(id(&f), None);
        }
    }

    #[test]
    fn loaded_veteran_save_shows_no_hint() {
        let (sailed_with_contract, journal_seen) = seed_from_load(2, false, 3);
        for (docked, active) in [(true, 0), (true, 1), (false, 1)] {
            let f = HintFacts {
                docked,
                active,
                settled: 2,
                wins: 3,
                sailed_with_contract,
                journal_seen,
            };
            assert_eq!(id(&f), None);
        }
    }

    #[test]
    fn fresh_load_mid_delivery_at_sea_shows_none() {
        let (sailed_with_contract, journal_seen) = seed_from_load(0, true, 0);
        let f = HintFacts {
            docked: false,
            active: 1,
            settled: 0,
            wins: 0,
            sailed_with_contract,
            journal_seen,
        };
        assert_eq!(id(&f), None);
    }

    #[test]
    fn the_bridge_finds_rows_by_text() {
        for hint in FIRST_VOYAGE_HINTS {
            assert_eq!(hint_by_text(hint.text).map(|row| row.id), Some(hint.id));
        }
        assert!(hint_by_text(CHART_NOTE).is_none());
    }
}
