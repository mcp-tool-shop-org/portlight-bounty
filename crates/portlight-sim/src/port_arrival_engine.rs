//! What the captain sees when a voyage docks.
//!
//! Ported from `engine/port_arrival_engine.py`. `generate_arrival` reads
//! institutions, culture, politics, arrival weather, an active festival, and
//! the captain's mentor. It does not draw the RNG. `format_arrival_text`
//! keeps the Rich markup Python writes into the arrival event.

use crate::content;
use crate::model::World;
use crate::sea_culture;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortArrivalExperience {
    pub port_id: String,
    pub port_name: String,
    pub region: String,
    pub dock_scene: String,
    pub arrival_weather: String,
    pub harbor_master_greeting: String,
    pub harbor_master_name: String,
    pub exchange_greeting: String,
    pub exchange_name: String,
    pub tavern_greeting: String,
    pub tavern_name: String,
    pub local_custom: String,
    pub landmark: String,
    pub tavern_rumor: String,
    pub cultural_group: String,
    pub political_flavor: String,
    pub port_grudge: String,
    pub active_festival_name: String,
    pub active_festival_description: String,
    pub mentor_greeting: String,
    pub mentor_name: String,
}

pub fn generate_arrival(world: &World, port_id: &str) -> PortArrivalExperience {
    let Some(port) = world.port(port_id) else {
        return PortArrivalExperience {
            port_id: port_id.to_string(),
            port_name: "Unknown".to_string(),
            region: "Unknown".to_string(),
            ..empty(port_id)
        };
    };
    let mut experience = PortArrivalExperience {
        port_id: port_id.to_string(),
        port_name: port.name.clone(),
        region: port.region.clone(),
        ..empty(port_id)
    };
    let regional = world.captain.standing.regional_of(&port.region);
    let port_standing = world.captain.standing.port_value(port_id).unwrap_or(0);
    let tier = standing_tier(regional.max(port_standing));
    let catalog = content::content();
    if let Some(culture) = catalog.port_culture(port_id) {
        experience.dock_scene = culture.dock_scene.clone();
        experience.local_custom = culture.local_custom.clone();
        experience.landmark = culture.landmark.clone();
        experience.tavern_rumor = culture.tavern_rumor.clone();
        experience.cultural_group = culture.cultural_group.clone();
    }
    if let Some(profile) = catalog.port_institution(port_id) {
        for npc in &profile.npcs {
            let greeting = match tier {
                "friendly" => npc.greeting_friendly.as_str(),
                "hostile" => npc.greeting_hostile.as_str(),
                _ => npc.greeting_neutral.as_str(),
            };
            match npc.institution.as_str() {
                "harbor_master" => {
                    experience.harbor_master_greeting = greeting.to_string();
                    experience.harbor_master_name = npc.name.clone();
                }
                "exchange" => {
                    experience.exchange_greeting = greeting.to_string();
                    experience.exchange_name = npc.name.clone();
                }
                "tavern" => {
                    experience.tavern_greeting = greeting.to_string();
                    experience.tavern_name = npc.name.clone();
                }
                _ => {}
            }
        }
    }
    if let Some(politics) = catalog.port_politics(port_id) {
        experience.political_flavor = politics.political_flavor.clone();
        experience.port_grudge = politics.port_grudge.clone();
    }
    // `generate_arrival` calls `get_arrival_weather(region, day)` and does not
    // pass the port name, so the generic subject stays in the sentence.
    experience.arrival_weather = sea_culture::arrival_weather(&port.region, world.day, "");
    for fest in &world.culture.active_festivals {
        if fest.port_id == port_id && fest.start_day <= world.day && world.day <= fest.end_day {
            if let Some(region) = catalog.region_culture(&port.region) {
                if let Some(festival) = region
                    .festivals
                    .iter()
                    .find(|festival| festival.id == fest.festival_id)
                {
                    experience.active_festival_name = festival.name.clone();
                    experience.active_festival_description = festival.description.clone();
                }
            }
            break;
        }
    }
    if let Some(template) = crate::custom_captain::active_captain(world) {
        if !template.mentor_npc_id.is_empty() && template.home_port_id == port_id {
            if let Some(mentor) = catalog.npc(&template.mentor_npc_id) {
                experience.mentor_name = mentor.name.clone();
                experience.mentor_greeting = match tier {
                    "friendly" => mentor.greeting_friendly.clone(),
                    "neutral" => mentor.greeting_neutral.clone(),
                    "hostile" => mentor.greeting_hostile.clone(),
                    _ => mentor.greeting_friendly.clone(),
                };
            }
        }
    }
    experience
}

pub fn format_arrival_text(exp: &PortArrivalExperience) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("[bold]You arrive at {}.[/bold]", exp.port_name));
    lines.push(String::new());
    if !exp.arrival_weather.is_empty() {
        lines.push(exp.arrival_weather.clone());
        lines.push(String::new());
    }
    if !exp.dock_scene.is_empty() {
        lines.push(exp.dock_scene.clone());
        lines.push(String::new());
    }
    if !exp.harbor_master_greeting.is_empty() {
        lines.push(format!(
            "[bold]{}:[/bold] {}",
            exp.harbor_master_name, exp.harbor_master_greeting
        ));
    }
    if !exp.mentor_greeting.is_empty() && exp.mentor_name != exp.harbor_master_name {
        lines.push(format!(
            "[bold]{}:[/bold] {}",
            exp.mentor_name, exp.mentor_greeting
        ));
    }
    if !exp.active_festival_name.is_empty() {
        lines.push(String::new());
        lines.push(format!(
            "[bold yellow]FESTIVAL: {}[/bold yellow]",
            exp.active_festival_name
        ));
        lines.push(exp.active_festival_description.clone());
    }
    if !exp.landmark.is_empty() {
        lines.push(String::new());
        lines.push(format!("[dim]Landmark: {}[/dim]", exp.landmark));
    }
    if !exp.local_custom.is_empty() {
        lines.push(format!("[dim]Custom: {}[/dim]", exp.local_custom));
    }
    lines
}

fn standing_tier(standing: i64) -> &'static str {
    if standing >= 15 {
        "friendly"
    } else if standing >= 0 {
        "neutral"
    } else {
        "hostile"
    }
}

fn empty(port_id: &str) -> PortArrivalExperience {
    PortArrivalExperience {
        port_id: port_id.to_string(),
        port_name: String::new(),
        region: String::new(),
        dock_scene: String::new(),
        arrival_weather: String::new(),
        harbor_master_greeting: String::new(),
        harbor_master_name: String::new(),
        exchange_greeting: String::new(),
        exchange_name: String::new(),
        tavern_greeting: String::new(),
        tavern_name: String::new(),
        local_custom: String::new(),
        landmark: String::new(),
        tavern_rumor: String::new(),
        cultural_group: String::new(),
        political_flavor: String::new(),
        port_grudge: String::new(),
        active_festival_name: String::new(),
        active_festival_description: String::new(),
        mentor_greeting: String::new(),
        mentor_name: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::new_game;

    #[test]
    fn porto_novo_arrival_names_the_harbor_and_the_port() {
        let world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        let exp = generate_arrival(&world, "porto_novo");
        assert_eq!(exp.port_name, world.port("porto_novo").unwrap().name);
        assert!(!exp.harbor_master_name.is_empty());
        assert!(!exp.dock_scene.is_empty());
        let lines = format_arrival_text(&exp);
        assert!(lines[0].contains(&exp.port_name));
        assert!(lines
            .iter()
            .any(|line| line.contains(&exp.harbor_master_name)));
        assert_eq!(
            format_arrival_text(&generate_arrival(&world, "missing"))[0],
            "[bold]You arrive at Unknown.[/bold]"
        );
    }
}
