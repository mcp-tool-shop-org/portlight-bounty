//! Stance duel from `engine/duel.py`.
//!
//! This is the resolution `portlight duel <stances>` and
//! `GameSession._resolve_pending_duel` run. It clears a pending pirate
//! challenge. The interactive encounter machine (negotiate, flee, naval,
//! boarding, `engine/combat.py`) is a separate system and is not ported here.

use crate::pyrand::PyRandom;

const STANCES: [&str; 3] = ["thrust", "slash", "parry"];

pub struct DuelRound {
    pub player_stance: String,
    pub opponent_stance: String,
    pub damage_to_opponent: i64,
    pub damage_to_player: i64,
    pub flavor: String,
}

pub struct DuelOutcome {
    pub opponent_id: String,
    pub opponent_name: String,
    pub player_won: bool,
    pub draw: bool,
    pub rounds: Vec<DuelRound>,
    pub silver_delta: i64,
    pub standing_delta: i64,
}

fn beats(player: &str, opponent: &str) -> &'static str {
    if player == opponent {
        "draw"
    } else if (player == "thrust" && opponent == "slash")
        || (player == "slash" && opponent == "parry")
        || (player == "parry" && opponent == "thrust")
    {
        "win"
    } else {
        "lose"
    }
}

fn personality_weights(personality: &str) -> [f64; 3] {
    match personality {
        "aggressive" => [0.60, 0.25, 0.15],
        "defensive" => [0.15, 0.25, 0.60],
        "balanced" => [0.35, 0.35, 0.30],
        _ => [0.33, 0.34, 0.33],
    }
}

fn pick_opponent_stance(personality: &str, history: &[String], rng: &mut PyRandom) -> String {
    let mut weights = personality_weights(personality);
    if personality == "balanced" && history.len() >= 2 {
        let n = history.len();
        if history[n - 2] == history[n - 1] {
            let idx = match history[n - 1].as_str() {
                "thrust" => 2,
                "slash" => 0,
                "parry" => 1,
                _ => 3,
            };
            if idx < 3 {
                weights[idx] += 0.30;
            }
        }
    }
    STANCES[rng.choices_weighted(&weights)].to_string()
}

fn flavor(stance: &str, outcome: &str) -> String {
    let text = match (stance, outcome) {
        ("thrust", "win") => "Your blade finds its mark — a clean thrust past their guard.",
        ("thrust", "lose") => "You lunge forward, but their parry turns your blade aside.",
        ("thrust", "draw") => {
            "Both blades extend. Steel rings against steel. Neither gives ground."
        }
        ("slash", "win") => "A wide arc catches them mid-parry. The cut connects.",
        ("slash", "lose") => "Your slash meets empty air — they were already thrusting.",
        ("slash", "draw") => "Blades cross in a shower of sparks. Matched power.",
        ("parry", "win") => {
            "You read the thrust perfectly. Their blade slides past as yours finds the opening."
        }
        ("parry", "lose") => {
            "You brace for a thrust that never comes — the slash catches your side."
        }
        ("parry", "draw") => "Both hold their ground, blades locked. A test of strength.",
        _ => "Blades clash.",
    };
    text.to_string()
}

fn resolve_round(
    player_stance: &str,
    opponent_stance: &str,
    rng: &mut PyRandom,
    crew_bonus: i64,
) -> DuelRound {
    let outcome = beats(player_stance, opponent_stance);
    let (damage_to_opponent, damage_to_player) = match outcome {
        "win" => {
            let base = rng.randint(2, 3);
            (base + (crew_bonus / 5).min(1), 0)
        }
        "lose" => (0, rng.randint(2, 3)),
        _ => (1, 1),
    };
    DuelRound {
        player_stance: player_stance.to_string(),
        opponent_stance: opponent_stance.to_string(),
        damage_to_opponent,
        damage_to_player,
        flavor: flavor(player_stance, outcome),
    }
}

/// `resolve_duel`. Stances are already validated by the caller.
pub fn resolve_duel(
    stances: &[String],
    opponent_id: &str,
    opponent_name: &str,
    personality: &str,
    strength: i64,
    rng: &mut PyRandom,
    player_crew: i64,
) -> DuelOutcome {
    let mut player_hp = 10;
    let mut opponent_hp = (10 + (strength - 5)).max(6);
    let crew_bonus = (player_crew - 5).max(0);
    let mut rounds = Vec::new();
    let mut history = Vec::new();
    for stance in stances {
        if player_hp <= 0 || opponent_hp <= 0 {
            break;
        }
        let opp = pick_opponent_stance(personality, &history, rng);
        let round = resolve_round(stance, &opp, rng, crew_bonus);
        player_hp -= round.damage_to_player;
        opponent_hp -= round.damage_to_opponent;
        history.push(stance.clone());
        rounds.push(round);
    }
    let player_won = opponent_hp <= 0 && player_hp > 0;
    let draw = player_hp <= 0 && opponent_hp <= 0;
    let (silver_delta, standing_delta) = if player_won {
        (20 + strength * 5, 5)
    } else if draw {
        (0, 2)
    } else {
        (-(15 + strength * 3), -2)
    };
    DuelOutcome {
        opponent_id: opponent_id.to_string(),
        opponent_name: opponent_name.to_string(),
        player_won,
        draw,
        rounds,
        silver_delta,
        standing_delta,
    }
}

/// Five stances from `rng.choice`, matching `_resolve_pending_duel`.
pub fn auto_stances(rng: &mut PyRandom) -> Vec<String> {
    (0..5)
        .map(|_| STANCES[rng.choice_index(3)].to_string())
        .collect()
}
