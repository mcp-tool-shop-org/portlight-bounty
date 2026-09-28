//! Personal fight after boarding. Port of `engine/combat.py`.
//!
//! Shoot, throw, dodge, fighting styles, and stamina. This is not the voyage
//! stance duel in `duel.rs`. Injury rolls go through [`crate::injuries`] so
//! the catalog and the session RNG stay aligned.

use std::collections::BTreeMap;

use crate::content::{self, FightingStyleDef};
use crate::pyrand::PyRandom;
use crate::util::py_trunc;

const CORE: [&str; 3] = ["thrust", "slash", "parry"];
const BASE_PLAYER_HP: i64 = 12;
const BASE_PLAYER_STAMINA: i64 = 8;
const BASE_OPPONENT_HP: i64 = 10;
const STAMINA_REGEN: i64 = 1;
const PARRY_STAMINA_BONUS: i64 = 1;
const GENERIC_FIREARM_MIN: i64 = 4;
const GENERIC_FIREARM_MAX: i64 = 6;
const GENERIC_FIREARM_ACCURACY: f64 = 0.65;
const GENERIC_FIREARM_RELOAD: i64 = 1;
const DEFAULT_OPPONENT_FIREARM: &str = "matchlock_pistol";

type InjuryEffects = crate::injuries::Effects;

fn injury_effects(ids: &[String]) -> InjuryEffects {
    crate::injuries::effects(ids)
}

fn roll_injury(damage: i64, attack_type: &str, rng: &mut PyRandom, bonus: f64) -> Option<String> {
    crate::injuries::roll_injury(damage, attack_type, rng, bonus)
}

fn quality_mods(quality: &str) -> (i64, f64) {
    match quality {
        "rusted" => (-1, -0.10),
        "worn" => (0, -0.05),
        "fine" => (1, 0.05),
        "masterwork" => (2, 0.10),
        _ => (0, 0.0),
    }
}

#[derive(Debug, Clone)]
pub struct CombatantState {
    pub hp: i64,
    pub hp_max: i64,
    pub stamina: i64,
    pub stamina_max: i64,
    pub ammo: i64,
    pub throwing_weapons: i64,
    pub throwing_weapon_ids: Vec<String>,
    pub active_style: Option<String>,
    pub injury_ids: Vec<String>,
    pub reload_turns: i64,
    pub last_action: Option<String>,
    pub style_cooldowns: BTreeMap<String, i64>,
    pub stun_turns: i64,
    pub firearm_id: Option<String>,
    pub mechanical_weapon_id: Option<String>,
    pub mechanical_ammo: i64,
    pub mechanical_reload: i64,
    pub armor_dr: i64,
    pub dodge_stamina_penalty: i64,
    pub melee_weapon_id: Option<String>,
    pub melee_quality: String,
    pub ranged_quality: String,
}

#[derive(Debug, Clone)]
pub struct CombatRound {
    pub turn: i64,
    pub player_action: String,
    pub opponent_action: String,
    pub damage_to_opponent: i64,
    pub damage_to_player: i64,
    pub player_stamina_delta: i64,
    pub opponent_stamina_delta: i64,
    pub injury_inflicted: Option<String>,
    pub opponent_injury: Option<String>,
    pub flavor: String,
    pub style_effect: Option<String>,
    pub stun_to_opponent: i64,
    pub stun_to_player: i64,
}

#[allow(clippy::too_many_arguments)]
pub fn create_player_combatant(
    crew: i64,
    active_style: Option<&str>,
    injury_ids: &[String],
    firearm_id: Option<&str>,
    firearm_ammo: i64,
    throwing_weapons: i64,
    throwing_weapon_ids: &[String],
    mechanical_weapon_id: Option<&str>,
    mechanical_ammo: i64,
    armor_id: Option<&str>,
    melee_weapon_id: Option<&str>,
    melee_quality: &str,
    ranged_quality: &str,
) -> CombatantState {
    let catalog = content::content();
    let effects = injury_effects(injury_ids);
    let crew_bonus = 0.max(crew - 5);
    let mut hp = BASE_PLAYER_HP + crew_bonus / 5 + effects.hp_max_mod;
    let style = active_style.and_then(|id| catalog.fighting_style(id));
    if let Some(style) = style {
        hp += style.passive_hp_bonus;
    }
    let mut stamina = BASE_PLAYER_STAMINA + effects.stamina_max_mod;
    let mut armor_dr = 0;
    let mut dodge_pen = 0;
    if let Some(armor) = armor_id.and_then(|id| catalog.armor(id)) {
        armor_dr = armor.damage_reduction;
        dodge_pen = armor.dodge_penalty;
        stamina += armor.stamina_penalty;
    }
    if let Some(weapon) = melee_weapon_id.and_then(|id| catalog.melee_weapon(id)) {
        stamina += weapon.speed_mod;
    }
    let mut tw_ids = throwing_weapon_ids.to_vec();
    if tw_ids.is_empty() && throwing_weapons > 0 {
        tw_ids = vec!["throwing_knife".to_string(); throwing_weapons as usize];
    }
    let hp = 1.max(hp);
    let stamina = 1.max(stamina);
    CombatantState {
        hp,
        hp_max: hp,
        stamina,
        stamina_max: stamina,
        ammo: firearm_ammo,
        throwing_weapons,
        throwing_weapon_ids: tw_ids,
        active_style: active_style.map(str::to_string),
        injury_ids: injury_ids.to_vec(),
        reload_turns: 0,
        last_action: None,
        style_cooldowns: BTreeMap::new(),
        stun_turns: 0,
        firearm_id: firearm_id.map(str::to_string),
        mechanical_weapon_id: mechanical_weapon_id.map(str::to_string),
        mechanical_ammo,
        mechanical_reload: 0,
        armor_dr,
        dodge_stamina_penalty: dodge_pen,
        melee_weapon_id: melee_weapon_id.map(str::to_string),
        melee_quality: melee_quality.to_string(),
        ranged_quality: ranged_quality.to_string(),
    }
}

pub fn create_opponent_combatant(
    strength: i64,
    ammo: i64,
    throwing_weapons: i64,
    firearm_id: Option<&str>,
    active_style: Option<&str>,
) -> CombatantState {
    let catalog = content::content();
    let hp = (BASE_OPPONENT_HP + (strength - 5) * 2).max(6);
    let stamina = (6 + strength / 2).max(3);
    let mut resolved = firearm_id.filter(|id| catalog.ranged_weapon(id).is_some());
    let mut ammo = ammo;
    if ammo > 0 && resolved.is_none() {
        resolved = Some(DEFAULT_OPPONENT_FIREARM);
        if catalog.ranged_weapon(DEFAULT_OPPONENT_FIREARM).is_none() {
            ammo = 0;
            resolved = None;
        }
    }
    if ammo <= 0 {
        ammo = 0;
        resolved = None;
    }
    CombatantState {
        hp,
        hp_max: hp,
        stamina,
        stamina_max: stamina,
        ammo,
        throwing_weapons,
        throwing_weapon_ids: Vec::new(),
        active_style: active_style.map(str::to_string),
        injury_ids: Vec::new(),
        reload_turns: 0,
        last_action: None,
        style_cooldowns: BTreeMap::new(),
        stun_turns: 0,
        firearm_id: resolved.map(str::to_string),
        mechanical_weapon_id: None,
        mechanical_ammo: 0,
        mechanical_reload: 0,
        armor_dr: 0,
        dodge_stamina_penalty: 0,
        melee_weapon_id: None,
        melee_quality: "standard".into(),
        ranged_quality: "standard".into(),
    }
}

fn select_shoot(state: &CombatantState) -> Option<&'static str> {
    let catalog = content::content();
    if state.ammo > 0
        && state.reload_turns <= 0
        && state
            .firearm_id
            .as_deref()
            .is_some_and(|id| catalog.ranged_weapon(id).is_some())
    {
        return Some("firearm");
    }
    if state.mechanical_ammo > 0
        && state.mechanical_reload <= 0
        && state
            .mechanical_weapon_id
            .as_deref()
            .is_some_and(|id| catalog.ranged_weapon(id).is_some())
    {
        return Some("mechanical");
    }
    None
}

fn weapon_id_for_shoot(state: &CombatantState) -> Option<&str> {
    match select_shoot(state) {
        Some("firearm") => state.firearm_id.as_deref(),
        Some("mechanical") => state.mechanical_weapon_id.as_deref(),
        _ => None,
    }
}

fn consume_shot(state: &mut CombatantState, kind: Option<&str>) {
    let catalog = content::content();
    if kind == Some("firearm") {
        state.ammo = 0.max(state.ammo - 1);
        let reload = state
            .firearm_id
            .as_deref()
            .and_then(|id| catalog.ranged_weapon(id))
            .map(|w| w.reload_turns)
            .unwrap_or(GENERIC_FIREARM_RELOAD);
        state.reload_turns = reload;
    } else if kind == Some("mechanical") {
        state.mechanical_ammo = 0.max(state.mechanical_ammo - 1);
        let reload = state
            .mechanical_weapon_id
            .as_deref()
            .and_then(|id| catalog.ranged_weapon(id))
            .map(|w| w.reload_turns)
            .unwrap_or(GENERIC_FIREARM_RELOAD);
        state.mechanical_reload = reload;
    }
}

pub fn available_actions(state: &CombatantState) -> Vec<String> {
    if state.stun_turns > 0 {
        return vec!["dodge".to_string()];
    }
    let effects = injury_effects(&state.injury_ids);
    let mut actions: Vec<String> = CORE.iter().map(|s| (*s).to_string()).collect();
    if effects.can_use_firearms && select_shoot(state).is_some() {
        actions.push("shoot".to_string());
    }
    if state.throwing_weapons > 0 {
        actions.push("throw".to_string());
    }
    if effects.can_dodge && state.last_action.as_deref() != Some("dodge") {
        actions.push("dodge".to_string());
    }
    if let Some(style_id) = state.active_style.as_deref() {
        if let Some(style) = content::content().fighting_style(style_id) {
            if let Some(sa) = style.special_action.as_ref() {
                let cooldown = state.style_cooldowns.get(&sa.id).copied().unwrap_or(0);
                if cooldown <= 0 && state.stamina >= sa.stamina_cost {
                    actions.push(sa.id.clone());
                }
            }
        }
    }
    actions
}

fn beats(action: &str) -> Option<&'static str> {
    match action {
        "thrust" => Some("slash"),
        "slash" => Some("parry"),
        "parry" => Some("thrust"),
        _ => None,
    }
}

fn melee_outcome(attacker: &str, defender: &str) -> &'static str {
    if attacker == defender {
        "draw"
    } else if beats(attacker) == Some(defender) {
        "win"
    } else {
        "lose"
    }
}

fn stamina_cost(action: &str) -> i64 {
    match action {
        "thrust" | "slash" | "parry" => 1,
        "dodge" => 2,
        _ => 0,
    }
}

fn melee_flavor(action: &str, outcome: &str) -> &'static str {
    match (action, outcome) {
        ("thrust", "win") => "Your blade finds its mark — a clean thrust past their guard.",
        ("thrust", "lose") => "You lunge forward, but their parry turns your blade aside.",
        ("thrust", "draw") => "Both blades extend. Steel rings against steel.",
        ("slash", "win") => "A wide arc catches them mid-parry. The cut connects.",
        ("slash", "lose") => "Your slash meets empty air — they were already thrusting.",
        ("slash", "draw") => "Blades cross in a shower of sparks.",
        ("parry", "win") => "You read the thrust perfectly. Your counter finds the opening.",
        ("parry", "lose") => {
            "You brace for a thrust that never comes — the slash catches your side."
        }
        ("parry", "draw") => "Both hold their ground, blades locked.",
        _ => "Blades clash.",
    }
}

fn calc_melee_damage(
    action: &str,
    outcome: &str,
    rng: &mut PyRandom,
    style: Option<&FightingStyleDef>,
    effects: &InjuryEffects,
    melee_weapon_id: Option<&str>,
    weapon_quality: &str,
) -> i64 {
    if outcome != "win" {
        return 0;
    }
    let mut base = rng.randint(2, 3);
    let mut bonus = effects.melee_damage_mod;
    if action == "thrust" {
        base = py_trunc(base as f64 * effects.thrust_multiplier);
    }
    if let Some(style) = style {
        if action == "thrust" {
            bonus += style.passive_thrust_bonus;
        } else if action == "slash" {
            bonus += style.passive_slash_bonus;
        }
    }
    if let Some(weapon) = melee_weapon_id.and_then(|id| content::content().melee_weapon(id)) {
        bonus += weapon.damage_bonus;
        if action == "thrust" {
            bonus += weapon.thrust_bonus;
        } else if action == "slash" {
            bonus += weapon.slash_bonus;
        }
        if let Some(style) = style {
            if weapon.compatible_styles.iter().any(|id| id == &style.id) {
                bonus += 1;
            }
        }
    }
    bonus += quality_mods(weapon_quality).0;
    0.max(base + bonus)
}

fn calc_ranged_damage(
    weapon_id: Option<&str>,
    rng: &mut PyRandom,
    accuracy_mod: f64,
    weapon_quality: &str,
) -> (i64, bool) {
    let Some(weapon_id) = weapon_id else {
        return (0, false);
    };
    let Some(weapon) = content::content().ranged_weapon(weapon_id) else {
        return (0, false);
    };
    let accuracy = weapon.accuracy + accuracy_mod + quality_mods(weapon_quality).1;
    if rng.random() > accuracy {
        return (0, false);
    }
    (rng.randint(weapon.damage_min, weapon.damage_max), true)
}

fn calc_throw_damage(
    weapon_id: Option<&str>,
    rng: &mut PyRandom,
    accuracy_mod: f64,
) -> (i64, bool, i64) {
    let weapon = weapon_id.and_then(|id| content::content().ranged_weapon(id));
    let Some(weapon) = weapon else {
        let accuracy = 0.70 + accuracy_mod;
        if rng.random() > accuracy {
            return (0, false, 0);
        }
        return (rng.randint(2, 3), true, 0);
    };
    let accuracy = weapon.accuracy + accuracy_mod;
    if rng.random() > accuracy {
        return (0, false, 0);
    }
    (
        rng.randint(weapon.damage_min, weapon.damage_max),
        true,
        weapon.stun_turns,
    )
}

fn resolve_style_action(
    style_action_id: &str,
    opponent_action: &str,
    style: &FightingStyleDef,
) -> (&'static str, i64, String) {
    let Some(sa) = style.special_action.as_ref() else {
        return ("draw", 0, String::new());
    };
    if sa.id != style_action_id {
        return ("draw", 0, String::new());
    }
    if sa.beats.iter().any(|a| a == opponent_action) {
        ("win", sa.damage_bonus, sa.flavor.clone())
    } else if sa.loses_to.iter().any(|a| a == opponent_action) {
        ("lose", 0, format!("Your {} is countered!", sa.name))
    } else {
        ("draw", 0, format!("Your {} meets resistance.", sa.name))
    }
}

fn roll_player_shoot(
    player: &CombatantState,
    effects: &InjuryEffects,
    style: Option<&FightingStyleDef>,
    opp_action: &str,
    rng: &mut PyRandom,
) -> (i64, String) {
    let mut accuracy_mod = effects.ranged_accuracy_mod;
    if let Some(style) = style {
        accuracy_mod += style.passive_ranged_accuracy;
    }
    let (dmg, hit) = calc_ranged_damage(
        weapon_id_for_shoot(player),
        rng,
        accuracy_mod,
        &player.ranged_quality,
    );
    if hit {
        if opp_action == "parry" {
            (
                dmg,
                format!("They try to parry but a bullet cares nothing for steel. {dmg} damage!"),
            )
        } else {
            (dmg, format!("Your shot strikes true — {dmg} damage!"))
        }
    } else {
        (
            0,
            "Your shot goes wide. The powder smoke stings your eyes.".to_string(),
        )
    }
}

fn py_title(text: &str) -> String {
    let mut out = String::new();
    let mut new_word = true;
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            if new_word {
                out.extend(ch.to_uppercase());
                new_word = false;
            } else {
                out.extend(ch.to_lowercase());
            }
        } else {
            new_word = true;
            out.push(ch);
        }
    }
    out
}

fn roll_player_throw(
    player: &CombatantState,
    effects: &InjuryEffects,
    style: Option<&FightingStyleDef>,
    rng: &mut PyRandom,
) -> (i64, String, i64) {
    let throw_wid = player.throwing_weapon_ids.first().map(String::as_str);
    let mut accuracy_mod = effects.ranged_accuracy_mod;
    if let Some(style) = style {
        accuracy_mod += style.passive_ranged_accuracy;
    }
    let (dmg, hit, stun) = calc_throw_damage(throw_wid, rng, accuracy_mod);
    if hit {
        let weapon_name = throw_wid
            .map(|id| py_title(&id.replace('_', " ")))
            .unwrap_or_else(|| "blade".to_string());
        let mut flavor = format!("Your {weapon_name} strikes — {dmg} damage!");
        if stun > 0 {
            flavor.push_str(&format!(" They're tangled — stunned for {stun} turn!"));
        }
        (dmg, flavor, stun)
    } else {
        (0, "Your throw goes wide.".to_string(), 0)
    }
}

fn roll_opponent_shoot(
    player_action: &str,
    rng: &mut PyRandom,
    opponent: &CombatantState,
) -> (i64, String) {
    let catalog = content::content();
    let weapon_id = weapon_id_for_shoot(opponent);
    let weapon = weapon_id.and_then(|id| catalog.ranged_weapon(id));
    let (dmg_min, dmg_max, accuracy) = if let Some(weapon) = weapon {
        (weapon.damage_min, weapon.damage_max, weapon.accuracy)
    } else {
        (
            GENERIC_FIREARM_MIN,
            GENERIC_FIREARM_MAX,
            GENERIC_FIREARM_ACCURACY,
        )
    };
    let opp_dmg = rng.randint(dmg_min, dmg_max);
    if rng.random() <= accuracy {
        if player_action == "parry" {
            (
                opp_dmg,
                format!("You raise your guard but the bullet punches through. {opp_dmg} damage!"),
            )
        } else {
            (opp_dmg, format!("A gunshot cracks — {opp_dmg} damage!"))
        }
    } else {
        (0, "A gunshot cracks but the ball goes wide.".to_string())
    }
}

fn roll_opponent_throw(rng: &mut PyRandom) -> (i64, String) {
    let opp_dmg = if rng.random() < 0.70 {
        rng.randint(2, 3)
    } else {
        0
    };
    if opp_dmg > 0 {
        (
            opp_dmg,
            format!("A thrown blade catches you — {opp_dmg} damage!"),
        )
    } else {
        (0, "A thrown weapon clatters past you.".to_string())
    }
}

fn player_style_outgoing(
    player_action: &str,
    opp_action: &str,
    style: &FightingStyleDef,
    effects: &InjuryEffects,
    rng: &mut PyRandom,
) -> (i64, String, Option<String>) {
    let (outcome, bonus_dmg, effect_desc) = resolve_style_action(player_action, opp_action, style);
    if outcome == "win" {
        let base = rng.randint(2, 3) + bonus_dmg + effects.melee_damage_mod;
        (0.max(base), effect_desc.clone(), Some(effect_desc))
    } else if outcome == "lose" {
        (0, effect_desc.clone(), Some(effect_desc))
    } else {
        let text = if effect_desc.is_empty() {
            "A glancing exchange.".to_string()
        } else {
            effect_desc.clone()
        };
        (1, text, Some(effect_desc).filter(|s| !s.is_empty()))
    }
}

fn join_flavor(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

fn combat_weights(personality: &str) -> [(&'static str, f64); 6] {
    match personality {
        "aggressive" => [
            ("thrust", 0.40),
            ("slash", 0.30),
            ("parry", 0.10),
            ("shoot", 0.10),
            ("throw", 0.05),
            ("dodge", 0.05),
        ],
        "defensive" => [
            ("thrust", 0.15),
            ("slash", 0.15),
            ("parry", 0.35),
            ("shoot", 0.10),
            ("throw", 0.05),
            ("dodge", 0.20),
        ],
        "wild" => [
            ("thrust", 0.20),
            ("slash", 0.20),
            ("parry", 0.15),
            ("shoot", 0.15),
            ("throw", 0.15),
            ("dodge", 0.15),
        ],
        _ => [
            ("thrust", 0.25),
            ("slash", 0.25),
            ("parry", 0.20),
            ("shoot", 0.10),
            ("throw", 0.05),
            ("dodge", 0.15),
        ],
    }
}

fn bump(weights: &mut [(&str, f64)], action: &str, extra: f64) {
    if let Some(slot) = weights.iter_mut().find(|(name, _)| *name == action) {
        slot.1 += extra;
    }
}

pub fn pick_opponent_action(
    personality: &str,
    opponent: &CombatantState,
    player: &CombatantState,
    player_last_action: Option<&str>,
    rng: &mut PyRandom,
) -> String {
    if opponent.stun_turns > 0 {
        return "dodge".to_string();
    }
    let mut weights = combat_weights(personality);
    if select_shoot(opponent).is_none() {
        bump(&mut weights, "shoot", 0.0);
        if let Some(slot) = weights.iter_mut().find(|(name, _)| *name == "shoot") {
            slot.1 = 0.0;
        }
    }
    if opponent.throwing_weapons <= 0 {
        if let Some(slot) = weights.iter_mut().find(|(name, _)| *name == "throw") {
            slot.1 = 0.0;
        }
    }
    if opponent.last_action.as_deref() == Some("dodge") || opponent.stamina < 2 {
        if let Some(slot) = weights.iter_mut().find(|(name, _)| *name == "dodge") {
            slot.1 = 0.0;
        }
    }
    if personality == "balanced" {
        match player_last_action {
            Some("thrust") => bump(&mut weights, "parry", 0.25),
            Some("slash") => bump(&mut weights, "thrust", 0.25),
            Some("parry") => bump(&mut weights, "slash", 0.25),
            Some("shoot") => bump(&mut weights, "dodge", 0.30),
            _ => {}
        }
    }
    if personality == "aggressive" && (player.hp as f64) < player.hp_max as f64 * 0.4 {
        bump(&mut weights, "thrust", 0.20);
        bump(&mut weights, "slash", 0.15);
    }
    if personality == "defensive" && (opponent.hp as f64) < opponent.hp_max as f64 * 0.4 {
        bump(&mut weights, "parry", 0.15);
        bump(&mut weights, "dodge", 0.20);
    }
    let values: Vec<f64> = weights.iter().map(|(_, w)| w.max(0.0)).collect();
    let total: f64 = values.iter().sum();
    if total <= 0.0 {
        return "thrust".to_string();
    }
    weights[rng.choices_weighted(&values)].0.to_string()
}

fn is_core(action: &str) -> bool {
    CORE.contains(&action)
}

fn is_ranged(action: &str) -> bool {
    action == "shoot" || action == "throw"
}

fn attack_type_for(action: &str) -> &'static str {
    if action == "shoot" {
        "shoot"
    } else if action == "slash" || action == "cleave" {
        "slash"
    } else {
        "thrust"
    }
}

pub fn resolve_combat_round(
    player_action: &str,
    player: &CombatantState,
    opponent: &CombatantState,
    opponent_personality: &str,
    rng: &mut PyRandom,
) -> CombatRound {
    let catalog = content::content();
    let player_style = player
        .active_style
        .as_deref()
        .and_then(|id| catalog.fighting_style(id));
    let effects = injury_effects(&player.injury_ids);
    let opp_action = pick_opponent_action(
        opponent_personality,
        opponent,
        player,
        player.last_action.as_deref(),
        rng,
    );
    let mut dmg_to_opponent = 0;
    let mut dmg_to_player = 0;
    let mut p_stamina_delta = 0;
    let mut o_stamina_delta = 0;
    let mut flavor;
    let mut style_effect = None;
    let mut stun_to_opponent = 0;
    let player_special = player_style
        .and_then(|style| style.special_action.as_ref())
        .is_some_and(|sa| sa.id == player_action);

    let mut p_cost = stamina_cost(player_action);
    if player_action == "dodge" {
        p_cost += player.dodge_stamina_penalty;
    }
    if player_special {
        if let Some(sa) = player_style.and_then(|s| s.special_action.as_ref()) {
            p_cost = sa.stamina_cost;
        }
    }
    p_stamina_delta -= p_cost;
    o_stamina_delta -= stamina_cost(&opp_action);

    if player_action == "dodge" && opp_action == "dodge" {
        flavor = "Both fighters circle warily. Neither commits.".to_string();
    } else if player_action == "dodge" {
        flavor = "You dive aside, avoiding the attack entirely.".to_string();
        if let Some(style) = player_style {
            if style.passive_dodge_counter > 0 {
                dmg_to_opponent = style.passive_dodge_counter;
                flavor.push_str(&format!(
                    " Your counterstrike catches them for {dmg_to_opponent} damage."
                ));
            }
        }
    } else if opp_action == "dodge" {
        flavor = "They dodge your attack with practiced ease.".to_string();
    } else if is_core(player_action) && is_core(&opp_action) {
        let outcome = melee_outcome(player_action, &opp_action);
        if outcome == "win" {
            dmg_to_opponent = calc_melee_damage(
                player_action,
                outcome,
                rng,
                player_style,
                &effects,
                player.melee_weapon_id.as_deref(),
                &player.melee_quality,
            );
        } else if outcome == "lose" {
            dmg_to_player = rng.randint(2, 3);
        }
        flavor = melee_flavor(player_action, outcome).to_string();
        if player_action == "parry" {
            let mut bonus = PARRY_STAMINA_BONUS;
            if let Some(style) = player_style {
                bonus += style.passive_parry_bonus;
            }
            p_stamina_delta += bonus;
        }
        if opp_action == "parry" {
            o_stamina_delta += PARRY_STAMINA_BONUS;
        }
    } else if let Some(style) = player_style.filter(|_| player_special && !is_ranged(&opp_action)) {
        let (outcome, bonus_dmg, effect_desc) =
            resolve_style_action(player_action, &opp_action, style);
        style_effect = Some(effect_desc.clone());
        if outcome == "win" {
            let base = rng.randint(2, 3) + bonus_dmg + effects.melee_damage_mod;
            dmg_to_opponent = 0.max(base);
            flavor = effect_desc;
        } else if outcome == "lose" {
            dmg_to_player = rng.randint(2, 3);
            flavor = effect_desc;
        } else {
            dmg_to_opponent = 1;
            dmg_to_player = 1;
            flavor = if effect_desc.is_empty() {
                "A glancing exchange.".to_string()
            } else {
                effect_desc
            };
        }
    } else {
        let mut p_flavor = String::new();
        let mut o_flavor = String::new();
        if player_action == "shoot" {
            let (dmg, text) = roll_player_shoot(player, &effects, player_style, &opp_action, rng);
            dmg_to_opponent = dmg;
            p_flavor = text;
        } else if player_action == "throw" {
            let (dmg, text, stun) = roll_player_throw(player, &effects, player_style, rng);
            dmg_to_opponent = dmg;
            if stun > 0 {
                stun_to_opponent = stun_to_opponent.max(stun);
            }
            p_flavor = text;
        } else if player_special {
            if let Some(style) = player_style {
                let (dmg, text, effect) =
                    player_style_outgoing(player_action, &opp_action, style, &effects, rng);
                dmg_to_opponent = dmg;
                p_flavor = text;
                style_effect = effect;
            }
        } else if is_core(player_action)
            && ((opp_action == "shoot" && player_action != "parry") || opp_action == "throw")
        {
            let p_dmg = calc_melee_damage(
                player_action,
                "win",
                rng,
                player_style,
                &effects,
                player.melee_weapon_id.as_deref(),
                &player.melee_quality,
            );
            dmg_to_opponent = p_dmg;
            if p_dmg > 0 {
                p_flavor = format!("Your {player_action} connects for {p_dmg}.");
            }
        }
        if opp_action == "shoot" {
            let (dmg, text) = roll_opponent_shoot(player_action, rng, opponent);
            dmg_to_player = dmg;
            o_flavor = text;
        } else if opp_action == "throw" {
            let (dmg, text) = roll_opponent_throw(rng);
            dmg_to_player = dmg;
            o_flavor = text;
        } else if is_core(&opp_action) && is_ranged(player_action) {
            let opp_dmg = if opp_action == "parry" {
                0
            } else {
                rng.randint(2, 3)
            };
            if (opp_action == "thrust" || opp_action == "slash") && opp_dmg > 0 {
                dmg_to_player = opp_dmg;
                if player_action == "shoot" {
                    o_flavor = format!("But their {opp_action} finds you for {opp_dmg}.");
                } else {
                    o_flavor = format!("Their {opp_action} catches you for {opp_dmg}.");
                }
            }
        }
        flavor = if is_ranged(player_action) || player_special {
            join_flavor(&[&p_flavor, &o_flavor])
        } else {
            join_flavor(&[&o_flavor, &p_flavor])
        };
        if flavor.is_empty() {
            flavor = "An awkward exchange. Neither fighter gains ground.".to_string();
        }
    }

    p_stamina_delta += STAMINA_REGEN;
    o_stamina_delta += STAMINA_REGEN;
    if dmg_to_player > 0 && player.armor_dr > 0 {
        dmg_to_player = 0.max(dmg_to_player - player.armor_dr);
    }
    if dmg_to_opponent > 0 && opponent.armor_dr > 0 {
        dmg_to_opponent = 0.max(dmg_to_opponent - opponent.armor_dr);
    }
    let injury_bonus = player_style.map(|s| s.passive_injury_bonus).unwrap_or(0.0);
    let player_injury = if dmg_to_player >= 4 {
        roll_injury(dmg_to_player, attack_type_for(&opp_action), rng, 0.0)
    } else {
        None
    };
    let opponent_injury = if dmg_to_opponent >= 4 {
        roll_injury(
            dmg_to_opponent,
            attack_type_for(player_action),
            rng,
            injury_bonus,
        )
    } else {
        None
    };

    CombatRound {
        turn: 0,
        player_action: player_action.to_string(),
        opponent_action: opp_action,
        damage_to_opponent: dmg_to_opponent,
        damage_to_player: dmg_to_player,
        player_stamina_delta: p_stamina_delta,
        opponent_stamina_delta: o_stamina_delta,
        injury_inflicted: player_injury,
        opponent_injury,
        flavor,
        style_effect,
        stun_to_opponent,
        stun_to_player: 0,
    }
}

pub fn apply_round_to_states(
    round: &CombatRound,
    player: &mut CombatantState,
    opponent: &mut CombatantState,
) {
    player.hp = 0.max(player.hp - round.damage_to_player);
    opponent.hp = 0.max(opponent.hp - round.damage_to_opponent);
    player.stamina = 0.max(
        player
            .stamina_max
            .min(player.stamina + round.player_stamina_delta),
    );
    opponent.stamina = 0.max(
        opponent
            .stamina_max
            .min(opponent.stamina + round.opponent_stamina_delta),
    );
    player.last_action = Some(round.player_action.clone());
    opponent.last_action = Some(round.opponent_action.clone());

    let player_shot = if round.player_action == "shoot" {
        select_shoot(player)
    } else {
        None
    };
    let opponent_shot = if round.opponent_action == "shoot" {
        select_shoot(opponent)
    } else {
        None
    };
    if player.reload_turns > 0 && player_shot != Some("firearm") {
        player.reload_turns -= 1;
    }
    if player.mechanical_reload > 0 && player_shot != Some("mechanical") {
        player.mechanical_reload -= 1;
    }
    if opponent.reload_turns > 0 && opponent_shot != Some("firearm") {
        opponent.reload_turns -= 1;
    }
    if opponent.mechanical_reload > 0 && opponent_shot != Some("mechanical") {
        opponent.mechanical_reload -= 1;
    }
    if round.player_action == "shoot" {
        consume_shot(player, player_shot);
    }
    if round.player_action == "throw" {
        player.throwing_weapons = 0.max(player.throwing_weapons - 1);
        if !player.throwing_weapon_ids.is_empty() {
            player.throwing_weapon_ids.remove(0);
        }
    }
    if round.opponent_action == "shoot" {
        consume_shot(opponent, opponent_shot);
    }
    if round.opponent_action == "throw" {
        opponent.throwing_weapons = 0.max(opponent.throwing_weapons - 1);
    }
    for value in player.style_cooldowns.values_mut() {
        if *value > 0 {
            *value -= 1;
        }
    }
    for value in opponent.style_cooldowns.values_mut() {
        if *value > 0 {
            *value -= 1;
        }
    }
    if player.stun_turns > 0 {
        player.stun_turns -= 1;
    }
    if opponent.stun_turns > 0 {
        opponent.stun_turns -= 1;
    }
    if let Some(style_id) = player.active_style.as_deref() {
        if let Some(style) = content::content().fighting_style(style_id) {
            if let Some(sa) = style.special_action.as_ref() {
                if round.player_action == sa.id {
                    player
                        .style_cooldowns
                        .insert(round.player_action.clone(), sa.cooldown);
                }
            }
        }
    }
    if round.stun_to_opponent > 0 {
        opponent.stun_turns = opponent.stun_turns.max(round.stun_to_opponent);
    }
    if round.stun_to_player > 0 {
        player.stun_turns = player.stun_turns.max(round.stun_to_player);
    }
    if let Some(id) = &round.injury_inflicted {
        player.injury_ids.push(id.clone());
    }
    if let Some(id) = &round.opponent_injury {
        opponent.injury_ids.push(id.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_rounds_match_combat_py() {
        // create_player_combatant + seven actions, Random(1).
        // Checked against engine/combat.py.
        let mut rng = PyRandom::from_seed(1);
        let mut player = create_player_combatant(
            5,
            Some("silat"),
            &[],
            Some("matchlock_pistol"),
            3,
            2,
            &["throwing_knife".into(), "bolas".into()],
            None,
            0,
            Some("chain_shirt"),
            Some("rapier"),
            "standard",
            "standard",
        );
        let mut opponent = create_opponent_combatant(6, 1, 1, None, None);
        assert_eq!((player.hp, player.stamina, player.armor_dr), (12, 7, 1));
        assert_eq!((opponent.hp, opponent.stamina, opponent.ammo), (12, 9, 1));
        let script = [
            ("shoot", "thrust", 0, 1, 11, 12, 2, 1),
            ("throw", "thrust", 3, 2, 9, 9, 2, 0),
            ("thrust", "slash", 5, 0, 9, 4, 2, 0),
            ("dodge", "shoot", 0, 0, 9, 4, 2, 0),
            ("slash", "thrust", 0, 1, 8, 4, 2, 0),
            ("parry", "slash", 0, 2, 6, 4, 2, 0),
            ("keris_strike", "slash", 3, 0, 6, 1, 2, 0),
        ];
        for (action, opp, dmg_o, dmg_p, hp, ohp, ammo, reload) in script {
            let round = resolve_combat_round(action, &player, &opponent, "aggressive", &mut rng);
            assert_eq!(round.opponent_action, opp, "{action}");
            assert_eq!(round.damage_to_opponent, dmg_o, "{action}");
            assert_eq!(round.damage_to_player, dmg_p, "{action}");
            apply_round_to_states(&round, &mut player, &mut opponent);
            assert_eq!((player.hp, opponent.hp), (hp, ohp), "{action}");
            assert_eq!(
                (player.ammo, player.reload_turns),
                (ammo, reload),
                "{action}"
            );
        }
        assert_eq!(opponent.injury_ids, vec!["bruised_ribs".to_string()]);
    }
}
