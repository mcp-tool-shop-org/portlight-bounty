//! New-game factory. Prices are computed without captain modifiers, matching
//! `portlight.content.world.new_game`. The script layer applies modifiers later.

use crate::content;
use crate::economy::recalculate_prices;
use crate::error::SimError;
use crate::model::{Captain, Port, Route, Ship, Standing, Voyage, VoyageStatus, World};

pub fn new_game(
    captain_name: &str,
    captain_type: &str,
    seed: i128,
    starting_port: Option<&str>,
) -> Result<World, SimError> {
    let catalog = content::content();
    let captain_def = catalog
        .captain(captain_type)
        .ok_or_else(|| SimError::UnknownCaptainType(captain_type.to_string()))?;
    let ship_def = catalog
        .ship(&captain_def.starting_ship_id)
        .ok_or_else(|| SimError::UnknownShip(captain_def.starting_ship_id.clone()))?;
    let port_id = starting_port
        .unwrap_or(captain_def.home_port_id.as_str())
        .to_string();
    let mut ports: Vec<Port> = catalog.ports.iter().map(Port::from).collect();
    for port in &mut ports {
        recalculate_prices(port, None);
    }
    let routes: Vec<Route> = catalog.routes.iter().map(Route::from).collect();
    Ok(World {
        captain: Captain {
            name: captain_name.to_string(),
            captain_type: captain_type.to_string(),
            silver: captain_def.starting_silver,
            ship: Some(Ship::from_template(ship_def)),
            cargo: Vec::new(),
            provisions: captain_def.starting_provisions,
            day: 1,
            standing: Standing::from_captain(captain_def),
            wanted_level: 0,
            active_bounties: Vec::new(),
            deferred_fees: Vec::new(),
        },
        ports,
        routes,
        voyage: Voyage {
            origin_id: port_id.clone(),
            destination_id: port_id,
            distance: 0,
            progress: 0,
            days_elapsed: 0,
            status: VoyageStatus::InPort,
            recent_events: Vec::new(),
        },
        day: 1,
        seed,
        pending_duel: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_captain_starts_at_home_with_template_silver() {
        let cases = [
            ("merchant", 550, "porto_novo", 30),
            ("smuggler", 500, "palm_cove", 35),
            ("navigator", 450, "silva_bay", 30),
            ("privateer", 475, "stormwall", 30),
            ("corsair", 500, "corsairs_rest", 30),
            ("scholar", 400, "monsoon_reach", 35),
            ("merchant_prince", 700, "al_manar", 30),
            ("dockhand", 300, "crosswind_isle", 30),
            ("bounty_hunter", 425, "crosswind_isle", 30),
        ];
        for (kind, silver, port, provisions) in cases {
            let world = new_game("C", kind, 1, None).expect(kind);
            assert_eq!(world.captain.silver, silver, "{kind}");
            assert_eq!(world.voyage.destination_id, port, "{kind}");
            assert_eq!(world.captain.provisions, provisions, "{kind}");
            assert_eq!(world.ports.len(), 20, "{kind}");
            assert!(!world.routes.is_empty());
            let home = world.port(port).expect("home");
            assert!(home.market.iter().all(|slot| slot.buy_price >= 1));
        }
    }

    #[test]
    fn ports_keep_python_map_coordinates_on_the_board_grid() {
        use crate::model::{MAP_GRID_HEIGHT, MAP_GRID_WIDTH};

        let expected = [
            ("porto_novo", 18, 8),
            ("al_manar", 24, 6),
            ("silva_bay", 14, 10),
            ("corsairs_rest", 21, 13),
            ("ironhaven", 8, 4),
            ("stormwall", 4, 8),
            ("thornport", 11, 10),
            ("sun_harbor", 14, 22),
            ("palm_cove", 10, 26),
            ("iron_point", 18, 24),
            ("pearl_shallows", 12, 30),
            ("jade_port", 34, 10),
            ("monsoon_reach", 38, 14),
            ("silk_haven", 42, 8),
            ("crosswind_isle", 32, 16),
            ("dragons_gate", 44, 12),
            ("spice_narrows", 38, 20),
            ("ember_isle", 34, 28),
            ("typhoon_anchorage", 40, 30),
            ("coral_throne", 44, 26),
        ];
        let world = new_game("C", "merchant", 1, None).expect("game");
        assert_eq!(world.ports.len(), expected.len());
        for (id, x, y) in expected {
            let port = world.port(id).unwrap_or_else(|| panic!("missing {id}"));
            assert_eq!((port.map_x, port.map_y), (x, y), "{id}");
            assert!(
                (0..=MAP_GRID_WIDTH).contains(&port.map_x)
                    && (0..=MAP_GRID_HEIGHT).contains(&port.map_y),
                "{id} sits outside the 50x36 grid"
            );
        }
        assert_eq!(world.routes.len(), 43);
        assert!(world.routes.iter().all(|route| {
            matches!(
                route.min_ship_class.as_str(),
                "sloop" | "cutter" | "brigantine" | "galleon" | "man_of_war"
            )
        }));
    }
}
