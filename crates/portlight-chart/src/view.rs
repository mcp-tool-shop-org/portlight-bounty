//! Chart view model. Every gameplay fact here is copied from the sim.

use portlight_sim::model::{VoyageStatus, World};
use portlight_sim::{LaneSuitability, SailLane};

use crate::assets::{self, lane_asset, ship_asset, TILE_QUAY};
use crate::project::{
    chart_to_screen, chart_to_screen_f, facing_from_chart_delta, frame_to_view,
    water_sprite_origin, Facing, Frame, ScreenRect,
};

/// First-playable waters. Ports outside this region are drawn only when the
/// sim has the player there, or as the end of a lane the picker listed.
pub const MEDITERRANEAN: &str = "Mediterranean";

/// Default new-game arguments for the chart. Seed 1 completes Porto Novo
/// to Al-Manar without a pending duel (see the playable test).
pub const FIRST_PLAYABLE_NAME: &str = "Ada";
pub const FIRST_PLAYABLE_CAPTAIN: &str = "merchant";
pub const FIRST_PLAYABLE_SEED: i64 = 1;

/// Viewport the Godot chart camera fits. The market panel sits beside it.
pub const CHART_VIEW_W: f32 = 900.0;
pub const CHART_VIEW_H: f32 = 720.0;

/// One ship occupies one chart cell. This stage has no larger footprint.
pub const SHIP_FOOTPRINT_CELLS: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartPort {
    pub id: String,
    pub name: String,
    pub chart_x: i64,
    pub chart_y: i64,
    pub anchor: (i32, i32),
    /// Top-left of the quay sprite, water datum applied.
    pub sprite_origin: (i32, i32),
    pub tile_id: &'static str,
    pub marker_id: &'static str,
    pub is_here: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartLane {
    pub destination_id: String,
    pub destination_name: String,
    pub suitability: LaneSuitability,
    pub suitability_note: Option<String>,
    pub estimated_days: i64,
    pub distance: i64,
    pub min_ship_class: String,
    pub from: (i32, i32),
    pub to: (i32, i32),
    pub destination_on_chart: bool,
    pub asset_id: &'static str,
    pub color: Rgba,
}

/// The voyage currently being sailed. Not a picker lane: `sail_lanes` is
/// empty at sea, and this leg does not invent a suitability.
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveLeg {
    pub origin_id: String,
    pub destination_id: String,
    pub from: (i32, i32),
    pub to: (i32, i32),
    pub progress: i64,
    pub distance: i64,
    pub color: Rgba,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShipMarker {
    pub chart_x: f64,
    pub chart_y: f64,
    pub cell_x: i64,
    pub cell_y: i64,
    pub footprint: u8,
    pub facing: Facing,
    pub anchor: (f32, f32),
    pub sprite_origin: (f32, f32),
    pub asset_id: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartModel {
    pub ports: Vec<ChartPort>,
    pub lanes: Vec<ChartLane>,
    pub leg: Option<ActiveLeg>,
    pub ship: ShipMarker,
    pub focus: ScreenRect,
    pub frame: Frame,
}

pub fn project_chart(world: &World) -> ChartModel {
    let lanes_src = portlight_sim::sail_lanes(world);
    let ports = visible_ports(world);
    let port_ids: Vec<&str> = ports.iter().map(|port| port.id.as_str()).collect();
    let lanes = lanes_src
        .iter()
        .map(|lane| chart_lane(world, lane, &port_ids))
        .collect();
    let leg = active_leg(world);
    let ship = ship_marker(world);
    let focus = focus_rect(&ports, &ship);
    let frame = frame_to_view(focus, CHART_VIEW_W, CHART_VIEW_H);
    ChartModel {
        ports,
        lanes,
        leg,
        ship,
        focus,
        frame,
    }
}

fn visible_ports(world: &World) -> Vec<ChartPort> {
    let here = current_id(world);
    let mut ports = Vec::new();
    for port in &world.ports {
        if !show_port(&port.region, &port.id, world) {
            continue;
        }
        let anchor = chart_to_screen(port.map_x, port.map_y);
        ports.push(ChartPort {
            id: port.id.clone(),
            name: port.name.clone(),
            chart_x: port.map_x,
            chart_y: port.map_y,
            anchor,
            sprite_origin: water_sprite_origin(anchor.0, anchor.1),
            tile_id: TILE_QUAY,
            marker_id: assets::PORT_MARKER,
            is_here: port.id == here && world.voyage.status != VoyageStatus::AtSea,
        });
    }
    ports.sort_by(|a, b| a.anchor.1.cmp(&b.anchor.1).then(a.id.cmp(&b.id)));
    ports
}

fn show_port(region: &str, id: &str, world: &World) -> bool {
    if region == MEDITERRANEAN {
        return true;
    }
    let voyage = &world.voyage;
    match voyage.status {
        VoyageStatus::AtSea | VoyageStatus::Arrived => {
            id == voyage.origin_id || id == voyage.destination_id
        }
        VoyageStatus::InPort => id == voyage.destination_id,
    }
}

fn current_id(world: &World) -> &str {
    world.voyage.destination_id.as_str()
}

fn chart_lane(world: &World, lane: &SailLane, port_ids: &[&str]) -> ChartLane {
    let origin = world
        .port(current_id(world))
        .map(|port| chart_to_screen(port.map_x, port.map_y))
        .unwrap_or((0, 0));
    let dest = world
        .port(&lane.destination_id)
        .map(|port| chart_to_screen(port.map_x, port.map_y))
        .unwrap_or(origin);
    ChartLane {
        destination_id: lane.destination_id.clone(),
        destination_name: lane.destination_name.clone(),
        suitability: lane.suitability,
        suitability_note: lane.suitability_note.clone(),
        estimated_days: lane.estimated_days,
        distance: lane.distance,
        min_ship_class: lane.min_ship_class.clone(),
        from: origin,
        to: dest,
        destination_on_chart: port_ids.contains(&lane.destination_id.as_str()),
        asset_id: lane_asset(lane.suitability),
        color: lane_color(lane.suitability),
    }
}

pub fn lane_color(suitability: LaneSuitability) -> Rgba {
    match suitability {
        LaneSuitability::Ok => Rgba {
            r: 36,
            g: 168,
            b: 150,
            a: 255,
        },
        LaneSuitability::Warning => Rgba {
            r: 214,
            g: 164,
            b: 48,
            a: 255,
        },
        LaneSuitability::Blocked => Rgba {
            r: 196,
            g: 72,
            b: 72,
            a: 255,
        },
    }
}

fn active_leg(world: &World) -> Option<ActiveLeg> {
    if world.voyage.status != VoyageStatus::AtSea {
        return None;
    }
    if world.voyage.origin_id == world.voyage.destination_id {
        return None;
    }
    let origin = world.port(&world.voyage.origin_id)?;
    let dest = world.port(&world.voyage.destination_id)?;
    Some(ActiveLeg {
        origin_id: origin.id.clone(),
        destination_id: dest.id.clone(),
        from: chart_to_screen(origin.map_x, origin.map_y),
        to: chart_to_screen(dest.map_x, dest.map_y),
        progress: world.voyage.progress,
        distance: world.voyage.distance,
        color: Rgba {
            r: 236,
            g: 224,
            b: 196,
            a: 255,
        },
    })
}

fn ship_marker(world: &World) -> ShipMarker {
    let origin = world
        .port(&world.voyage.origin_id)
        .map(|port| (port.map_x, port.map_y))
        .unwrap_or((0, 0));
    let dest = world
        .port(&world.voyage.destination_id)
        .map(|port| (port.map_x, port.map_y))
        .unwrap_or(origin);
    let t = if world.voyage.status == VoyageStatus::AtSea && world.voyage.distance > 0 {
        (world.voyage.progress as f64 / world.voyage.distance as f64).clamp(0.0, 1.0)
    } else if world.voyage.status == VoyageStatus::AtSea {
        0.0
    } else {
        1.0
    };
    let chart_x = origin.0 as f64 + (dest.0 as f64 - origin.0 as f64) * t;
    let chart_y = origin.1 as f64 + (dest.1 as f64 - origin.1 as f64) * t;
    let facing = if origin == dest {
        Facing::E
    } else {
        facing_from_chart_delta(dest.0 - origin.0, dest.1 - origin.1)
    };
    let anchor = chart_to_screen_f(chart_x, chart_y);
    let asset = ship_asset(facing);
    ShipMarker {
        chart_x,
        chart_y,
        cell_x: chart_x.round() as i64,
        cell_y: chart_y.round() as i64,
        footprint: SHIP_FOOTPRINT_CELLS,
        facing,
        anchor,
        sprite_origin: (anchor.0 - asset.sit_x as f32, anchor.1 - asset.sit_y as f32),
        asset_id: asset.id,
    }
}

fn focus_rect(ports: &[ChartPort], ship: &ShipMarker) -> ScreenRect {
    let mut rect =
        ScreenRect::from_point(ship.anchor.0.round() as i32, ship.anchor.1.round() as i32);
    for port in ports {
        rect.include(port.anchor.0, port.anchor.1);
    }
    rect.pad(160, 140)
}

#[cfg(test)]
mod tests {
    use super::*;
    use portlight_sim::{new_game, LaneSuitability, Session};

    fn merchant_at(port: Option<&str>) -> World {
        new_game("Ada", "merchant", 42, port).expect("game")
    }

    #[test]
    fn starting_chart_is_four_mediterranean_ports_and_picker_lanes() {
        let world = merchant_at(None);
        assert_eq!(world.voyage.destination_id, "porto_novo");
        let chart = project_chart(&world);
        let ids: Vec<_> = chart.ports.iter().map(|port| port.id.as_str()).collect();
        assert_eq!(ids.len(), 4);
        for id in ["porto_novo", "al_manar", "silva_bay", "corsairs_rest"] {
            assert!(ids.contains(&id), "{id}");
        }
        let here = chart.ports.iter().find(|port| port.is_here).unwrap();
        assert_eq!(here.id, "porto_novo");
        assert_eq!(here.chart_x, 18);
        assert_eq!(here.chart_y, 8);
        assert_eq!(here.anchor, (640, 832));
        assert_eq!(here.sprite_origin, (640 - 64, 832 - 48));
        assert_eq!(here.tile_id, TILE_QUAY);

        let sim = portlight_sim::sail_lanes(&world);
        let drawn: Vec<_> = chart
            .lanes
            .iter()
            .map(|lane| lane.destination_id.as_str())
            .collect();
        let expected: Vec<_> = sim
            .iter()
            .map(|lane| lane.destination_id.as_str())
            .collect();
        assert_eq!(drawn, expected);
        let open = chart
            .lanes
            .iter()
            .filter(|lane| lane.suitability == LaneSuitability::Ok)
            .count();
        assert_eq!(open, 3);
        assert!(drawn.contains(&"al_manar"));

        let ironhaven = chart
            .lanes
            .iter()
            .find(|lane| lane.destination_id == "ironhaven")
            .unwrap();
        assert_eq!(ironhaven.suitability, LaneSuitability::Warning);
        assert_eq!(ironhaven.estimated_days, 4);
        assert_eq!(ironhaven.asset_id, assets::LANE_WARNING);
        assert!(!ironhaven.destination_on_chart);
        assert_eq!(chart.ship.footprint, 1);
        assert_eq!(chart.ship.cell_x, 18);
        assert_eq!(chart.ship.cell_y, 8);
        assert!(chart.leg.is_none());
    }

    #[test]
    fn five_mediterranean_sloop_lanes_exist_in_the_sim() {
        let world = merchant_at(None);
        let med: Vec<_> = world
            .routes
            .iter()
            .filter(|route| {
                route.min_ship_class == "sloop"
                    && world.port(&route.port_a).unwrap().region == MEDITERRANEAN
                    && world.port(&route.port_b).unwrap().region == MEDITERRANEAN
            })
            .collect();
        assert_eq!(med.len(), 5);
        assert_eq!(
            world
                .ports
                .iter()
                .filter(|port| port.region == MEDITERRANEAN)
                .count(),
            4
        );
    }

    #[test]
    fn blocked_lanes_stay_on_the_chart() {
        let world = merchant_at(Some("corsairs_rest"));
        let chart = project_chart(&world);
        let stormwall = chart
            .lanes
            .iter()
            .find(|lane| lane.destination_id == "stormwall")
            .expect("blocked lane listed");
        assert_eq!(stormwall.suitability, LaneSuitability::Blocked);
        assert_eq!(stormwall.asset_id, assets::LANE_BLOCKED);
        assert!(stormwall
            .suitability_note
            .as_deref()
            .unwrap()
            .starts_with("BLOCKED"));
        assert_eq!(stormwall.color, lane_color(LaneSuitability::Blocked));
    }

    #[test]
    fn focus_keeps_the_mediterranean_and_lets_distant_lanes_run_off() {
        let world = merchant_at(None);
        let chart = project_chart(&world);
        for port in &chart.ports {
            assert!(chart.focus.contains(port.anchor.0, port.anchor.1));
        }
        let ironhaven = world.port("ironhaven").unwrap();
        let (x, y) = chart_to_screen(ironhaven.map_x, ironhaven.map_y);
        assert!(!chart.focus.contains(x, y));
    }

    #[test]
    fn grain_road_faces_east_and_the_ship_is_one_cell() {
        let mut session = Session::new_game("Ada", "merchant", FIRST_PLAYABLE_SEED, None).unwrap();
        session.depart("al_manar").unwrap();
        let chart = project_chart(session.world());
        assert!(chart.lanes.is_empty());
        let leg = chart.leg.expect("underway");
        assert_eq!(leg.origin_id, "porto_novo");
        assert_eq!(leg.destination_id, "al_manar");
        assert_eq!(chart.ship.facing, Facing::E);
        assert_eq!(chart.ship.asset_id, "chart.ship.sloop.e");
        assert_eq!(chart.ship.footprint, 1);
        assert_eq!((chart.ship.cell_x, chart.ship.cell_y), (18, 8));

        session.advance();
        let chart = project_chart(session.world());
        assert_eq!(chart.ship.footprint, 1);
        assert!(chart.ship.chart_x > 18.0);
        assert_eq!(chart.ship.facing, Facing::E);
        let on_segment = chart.ship.cell_x >= 18 && chart.ship.cell_x <= 24;
        assert!(on_segment, "cell {}", chart.ship.cell_x);
    }

    #[test]
    fn first_playable_sails_to_al_manar_and_trades() {
        let mut session = Session::new_game(
            FIRST_PLAYABLE_NAME,
            FIRST_PLAYABLE_CAPTAIN,
            FIRST_PLAYABLE_SEED,
            None,
        )
        .unwrap();
        assert_eq!(session.docked_port_id(), Some("porto_novo"));
        session.buy("grain", 5).unwrap();
        session.depart("al_manar").unwrap();
        let mut docked = false;
        for _ in 0..12 {
            assert!(
                session.world().pending_duel.is_none(),
                "seed {FIRST_PLAYABLE_SEED} hit a duel"
            );
            let report = session.advance();
            if session.docked_port_id() == Some("al_manar") {
                assert!(report.docked);
                docked = true;
                break;
            }
        }
        assert!(docked, "did not arrive");
        let held = portlight_sim::economy::cargo_quantity(&session.world().captain.cargo, "grain");
        if held > 0 {
            session.sell("grain", held.min(5)).unwrap();
        }
        session.buy("spice", 1).unwrap();
        session.sell("spice", 1).unwrap();
        let chart = project_chart(session.world());
        assert!(chart
            .ports
            .iter()
            .any(|port| port.id == "al_manar" && port.is_here));
        assert!(chart
            .lanes
            .iter()
            .any(|lane| lane.destination_id == "porto_novo"));
        assert!(
            chart
                .lanes
                .iter()
                .any(|lane| lane.suitability == LaneSuitability::Blocked),
            "Al-Manar still lists the lane depart will refuse"
        );
    }
}
