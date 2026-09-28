//! Fighting-style training from `engine/training.py`.
//!
//! [`crate::session::Session::train_crew`] is the player-facing wrapper. The
//! Python module trains the captain, not anonymous sailors.

use crate::content;

/// `None` means the style can be learned.
pub fn can_learn_style(
    learned_styles: &[String],
    injured_body_parts: &[&str],
    silver: i64,
    port_id: &str,
    style_id: &str,
) -> Option<String> {
    let Some(style) = content::content().fighting_style(style_id) else {
        return Some(format!("Unknown fighting style: {style_id}"));
    };
    if learned_styles.iter().any(|id| id == style_id) {
        return Some(format!("You already know {}.", style.name));
    }
    if !style.training_port_ids.iter().any(|id| id == port_id) {
        return Some(format!("No {} master at this port.", style.name));
    }
    if silver < style.silver_cost {
        return Some(format!(
            "Not enough silver. {} training costs {} silver.",
            style.name, style.silver_cost
        ));
    }
    if (learned_styles.len() as i64) < style.prerequisite_styles {
        return Some(format!(
            "{} requires knowledge of {} other style(s) first.",
            style.name, style.prerequisite_styles
        ));
    }
    let mut blocked: Vec<&str> = style
        .required_body_parts
        .iter()
        .filter(|part| injured_body_parts.iter().any(|hurt| hurt == part))
        .map(|part| part.as_str())
        .collect();
    blocked.sort_unstable();
    if !blocked.is_empty() {
        return Some(format!(
            "Your injuries ({}) prevent learning {}.",
            blocked.join(", "),
            style.name
        ));
    }
    None
}

/// Append the style and subtract its cost. Returns training days.
pub fn learn_style(learned_styles: &mut Vec<String>, silver: &mut i64, style_id: &str) -> i64 {
    let style = content::content()
        .fighting_style(style_id)
        .expect("learn_style requires a known style");
    let days = style.training_days;
    *silver -= style.silver_cost;
    learned_styles.push(style_id.to_string());
    days
}

pub fn available_training(port_id: &str) -> Vec<String> {
    content::content()
        .styles_at(port_id)
        .into_iter()
        .map(|style| style.id.clone())
        .collect()
}

pub fn masters_at_port(port_id: &str) -> Vec<String> {
    content::content()
        .masters_at(port_id)
        .into_iter()
        .map(|master| master.id.clone())
        .collect()
}

pub fn style_usable(style_id: &str, injured_body_parts: &[&str]) -> bool {
    let Some(style) = content::content().fighting_style(style_id) else {
        return false;
    };
    !style
        .required_body_parts
        .iter()
        .any(|part| injured_body_parts.iter().any(|hurt| hurt == part))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_destreza_is_taught_at_porto_novo() {
        assert_eq!(
            available_training("porto_novo"),
            vec!["la_destreza".to_string()]
        );
        assert_eq!(masters_at_port("porto_novo"), vec!["maestro_luciano"]);
        let style = content::content().fighting_style("la_destreza").unwrap();
        assert_eq!(style.passive_thrust_bonus, 1);
        assert_eq!(
            style
                .special_action
                .as_ref()
                .map(|action| action.id.as_str()),
            Some("estocada")
        );
        let mut learned = Vec::new();
        let mut silver = 550;
        assert!(can_learn_style(&learned, &[], silver, "porto_novo", "la_destreza").is_none());
        let days = learn_style(&mut learned, &mut silver, "la_destreza");
        assert_eq!(days, 5);
        assert_eq!(silver, 470);
        assert_eq!(
            can_learn_style(&learned, &[], silver, "porto_novo", "la_destreza").unwrap(),
            "You already know La Destreza."
        );
    }

    #[test]
    fn silat_needs_a_prior_style_and_a_working_hand() {
        let learned = Vec::new();
        let error = can_learn_style(&learned, &[], 500, "jade_port", "silat").unwrap();
        assert!(error.contains("requires knowledge of 1 other style"));
        let learned = vec!["la_destreza".to_string()];
        let error = can_learn_style(&learned, &["hand"], 500, "jade_port", "silat").unwrap();
        assert_eq!(error, "Your injuries (hand) prevent learning Silat.");
        assert!(!style_usable("silat", &["leg"]));
        assert!(style_usable("silat", &["arm"]));
        assert!(!style_usable("missing", &[]));
    }
}
