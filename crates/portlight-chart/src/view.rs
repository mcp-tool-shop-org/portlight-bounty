//! Chart view model. Every gameplay fact here is copied from the sim.

use portlight_sim::model::{PendingDuel, VoyageStatus, World};
use portlight_sim::{LaneSuitability, SailLane, Session};

use crate::assets::{self, chart_water_id, ship_asset, Asset, PORT_MARKER, SLOOP_WAKE};
use crate::project::{
    chart_to_screen_f, facing_from_chart_delta, follow_ship, frame_to_view, sprite_origin,
    water_cell_bottom, water_cell_center, Facing, Frame, ScreenRect, DOCKED_OFFSET_X,
    DOCKED_OFFSET_Y,
};

/// First-playable waters. Ports outside this region are drawn only when the
/// sim has the player there, or as the end of a lane the picker listed.
pub const MEDITERRANEAN: &str = "Mediterranean";

/// Default new-game arguments for the chart. Seed 1 completes Porto Novo
/// to Al-Manar without a pending duel (see the playable test).
pub const FIRST_PLAYABLE_NAME: &str = "Ada";
pub const FIRST_PLAYABLE_CAPTAIN: &str = "merchant";
pub const FIRST_PLAYABLE_SEED: i128 = 1;

/// Viewport the Godot chart camera fits. The market panel sits beside it.
pub const CHART_VIEW_W: f32 = 900.0;
pub const CHART_VIEW_H: f32 = 720.0;

/// One ship has one sort point. It is not a grid occupant.
pub const SHIP_FOOTPRINT_CELLS: u8 = 1;

/// Fixed presentation length of one committed sim day.
pub const DAY_TWEEN_SECS: f32 = 0.45;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaterTile {
    pub u: i32,
    pub v: i32,
    pub asset_id: &'static str,
    /// Top-left so the tile's bottom vertex sits on the cell bottom.
    pub origin: (f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartPort {
    pub id: String,
    pub name: String,
    pub region: String,
    pub chart_x: i64,
    pub chart_y: i64,
    /// Projected anchor. The marker's `(64, 95)` pixel lands here.
    pub at: (f32, f32),
    pub sprite_origin: (f32, f32),
    pub marker_id: &'static str,
    pub region_color: Rgba,
    /// First feature only, matching the TUI badge. `S`, `B`, or `H`.
    pub badge: Option<char>,
    pub is_here: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartLane {
    pub destination_id: String,
    pub destination_name: String,
    pub lore_name: String,
    pub suitability: LaneSuitability,
    pub suitability_note: Option<String>,
    pub estimated_days: i64,
    pub distance: i64,
    pub danger: f64,
    pub min_ship_class: String,
    pub from: (f32, f32),
    pub to: (f32, f32),
    pub destination_on_chart: bool,
    pub color: Rgba,
}

/// The voyage currently being sailed. Not a picker lane: `sail_lanes` is
/// empty at sea, and this leg does not invent a suitability.
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveLeg {
    pub origin_id: String,
    pub destination_id: String,
    pub from: (f32, f32),
    pub to: (f32, f32),
    pub progress: i64,
    pub distance: i64,
    pub color: Rgba,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShipMarker {
    pub chart_x: f64,
    pub chart_y: f64,
    pub footprint: u8,
    pub facing: Facing,
    /// Rest point after the tick. Docked, this includes the +(32, 16) offset.
    pub at: (f32, f32),
    pub sprite_origin: (f32, f32),
    pub wake_origin: (f32, f32),
    pub asset_id: &'static str,
    pub wake_id: &'static str,
    pub docked: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartModel {
    pub tiles: Vec<WaterTile>,
    pub ports: Vec<ChartPort>,
    pub lanes: Vec<ChartLane>,
    pub leg: Option<ActiveLeg>,
    pub ship: ShipMarker,
    pub focus: ScreenRect,
    pub frame: Frame,
}

/// What a click on a port marker is allowed to do. `NoLane` and `AtSea`
/// must not call the sim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortPress {
    /// Already docked here. The view may open the market.
    OpenHere,
    /// A picker lane exists, including lanes `depart` will refuse.
    Depart(String),
    /// No direct lane from the current port.
    NoLane,
    /// The ship is at sea. Destination clicks do not call `depart`.
    AtSea,
}

pub fn project_chart(session: &Session) -> ChartModel {
    let world = session.world();
    let lanes_src = session.sail_lanes();
    let ports = visible_ports(world);
    let port_ids: Vec<&str> = ports.iter().map(|port| port.id.as_str()).collect();
    let lanes = lanes_src
        .iter()
        .map(|lane| chart_lane(world, lane, &port_ids))
        .collect();
    let leg = active_leg(world);
    let ship = ship_marker(world);
    let focus = focus_rect(&ports, &ship);
    let region = frame_to_view(focus, CHART_VIEW_W, CHART_VIEW_H);
    let at_sea = world.voyage.status == VoyageStatus::AtSea;
    let frame = follow_ship(region, ship.at, at_sea, CHART_VIEW_W, CHART_VIEW_H);
    let tiles = water_tiles(&focus);
    ChartModel {
        tiles,
        ports,
        lanes,
        leg,
        ship,
        focus,
        frame,
    }
}

pub fn press_port(session: &Session, port_id: &str) -> PortPress {
    let world = session.world();
    if world.voyage.status == VoyageStatus::AtSea {
        return PortPress::AtSea;
    }
    if world.voyage.destination_id == port_id {
        return PortPress::OpenHere;
    }
    if session
        .sail_lanes()
        .iter()
        .any(|lane| lane.destination_id == port_id)
    {
        PortPress::Depart(port_id.to_string())
    } else {
        PortPress::NoLane
    }
}

/// The map refuses Advance while a duel is pending. It does not call the sim.
pub fn advance_refusal(pending: Option<&PendingDuel>) -> Option<&'static str> {
    pending.map(|_| "A duel is pending. Advance is refused.")
}

pub fn hover_at(model: &ChartModel, x: f32, y: f32) -> Option<String> {
    if let Some(port) = model
        .ports
        .iter()
        .find(|port| dist(port.at, (x, y)) <= 36.0)
    {
        let badge = port.badge.map(|b| format!(" {b}")).unwrap_or_default();
        return Some(format!("{}{}  {}", port.name, badge, port.region));
    }
    let mut best: Option<(&ChartLane, f32)> = None;
    for lane in &model.lanes {
        let d = segment_distance(lane.from, lane.to, (x, y));
        if d > 14.0 {
            continue;
        }
        if best.is_none_or(|(_, best_d)| d < best_d) {
            best = Some((lane, d));
        }
    }
    best.map(|(lane, _)| lane_inspect(lane))
}

pub fn lane_inspect(lane: &ChartLane) -> String {
    let name = if lane.lore_name.is_empty() {
        lane.destination_name.clone()
    } else {
        format!("{} ({})", lane.destination_name, lane.lore_name)
    };
    format!(
        "{name}  {}  danger {:.2}  {}  {} days",
        lane.distance, lane.danger, lane.min_ship_class, lane.estimated_days
    )
}

fn visible_ports(world: &World) -> Vec<ChartPort> {
    let here = current_id(world);
    let mut ports = Vec::new();
    for port in &world.ports {
        if !show_port(&port.region, &port.id, world) {
            continue;
        }
        let at = chart_to_screen_f(port.map_x as f64, port.map_y as f64);
        let marker = assets::asset(PORT_MARKER).expect("port marker");
        ports.push(ChartPort {
            id: port.id.clone(),
            name: port.name.clone(),
            region: port.region.clone(),
            chart_x: port.map_x,
            chart_y: port.map_y,
            at,
            sprite_origin: sprite_origin(at, marker.anchor_x, marker.anchor_y),
            marker_id: PORT_MARKER,
            region_color: region_color(&port.region),
            badge: feature_badge(port.features.first().map(String::as_str)),
            is_here: port.id == here && world.voyage.status != VoyageStatus::AtSea,
        });
    }
    ports.sort_by(|a, b| {
        a.at.1
            .partial_cmp(&b.at.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.id.cmp(&b.id))
    });
    ports
}

fn feature_badge(feature: Option<&str>) -> Option<char> {
    match feature.map(str::to_ascii_lowercase).as_deref() {
        Some("shipyard") => Some('S'),
        Some("black_market") => Some('B'),
        Some("safe_harbor") | Some("safe_harbour") => Some('H'),
        _ => None,
    }
}

pub fn region_color(region: &str) -> Rgba {
    match region {
        "Mediterranean" | "North Atlantic" => Rgba {
            r: 64,
            g: 196,
            b: 210,
            a: 255,
        },
        "West Africa" => Rgba {
            r: 214,
            g: 186,
            b: 64,
            a: 255,
        },
        "East Indies" => Rgba {
            r: 196,
            g: 72,
            b: 72,
            a: 255,
        },
        "South Seas" => Rgba {
            r: 72,
            g: 168,
            b: 96,
            a: 255,
        },
        _ => Rgba {
            r: 180,
            g: 180,
            b: 180,
            a: 255,
        },
    }
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
        .map(|port| chart_to_screen_f(port.map_x as f64, port.map_y as f64))
        .unwrap_or((0.0, 0.0));
    let dest = world
        .port(&lane.destination_id)
        .map(|port| chart_to_screen_f(port.map_x as f64, port.map_y as f64))
        .unwrap_or(origin);
    let lore_name = world
        .find_route(current_id(world), &lane.destination_id)
        .map(|route| route.lore_name.clone())
        .unwrap_or_default();
    ChartLane {
        destination_id: lane.destination_id.clone(),
        destination_name: lane.destination_name.clone(),
        lore_name,
        suitability: lane.suitability,
        suitability_note: lane.suitability_note.clone(),
        estimated_days: lane.estimated_days,
        distance: lane.distance,
        danger: lane.danger,
        min_ship_class: lane.min_ship_class.clone(),
        from: origin,
        to: dest,
        destination_on_chart: port_ids.contains(&lane.destination_id.as_str()),
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
        from: chart_to_screen_f(origin.map_x as f64, origin.map_y as f64),
        to: chart_to_screen_f(dest.map_x as f64, dest.map_y as f64),
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
    let docked = world.voyage.status != VoyageStatus::AtSea;
    let t = if !docked && world.voyage.distance > 0 {
        (world.voyage.progress as f64 / world.voyage.distance as f64).clamp(0.0, 1.0)
    } else if !docked {
        0.0
    } else {
        1.0
    };
    let chart_x = origin.0 as f64 + (dest.0 as f64 - origin.0 as f64) * t;
    let chart_y = origin.1 as f64 + (dest.1 as f64 - origin.1 as f64) * t;
    let facing = if origin == dest {
        Facing::F1
    } else {
        facing_from_chart_delta((dest.0 - origin.0) as f64, (dest.1 - origin.1) as f64, None)
    };
    let mut at = chart_to_screen_f(chart_x, chart_y);
    if docked {
        at.0 += DOCKED_OFFSET_X;
        at.1 += DOCKED_OFFSET_Y;
    }
    let hull = ship_asset(facing);
    let wake = assets::asset(SLOOP_WAKE).expect("wake");
    ShipMarker {
        chart_x,
        chart_y,
        footprint: SHIP_FOOTPRINT_CELLS,
        facing,
        at,
        sprite_origin: placed(at, hull),
        wake_origin: placed(at, wake),
        asset_id: hull.id,
        wake_id: wake.id,
        docked,
    }
}

fn placed(at: (f32, f32), asset: &Asset) -> (f32, f32) {
    sprite_origin(at, asset.anchor_x, asset.anchor_y)
}

fn focus_rect(ports: &[ChartPort], ship: &ShipMarker) -> ScreenRect {
    let mut rect = ScreenRect::from_point(ship.at.0.round() as i32, ship.at.1.round() as i32);
    for port in ports {
        rect.include(port.at.0.round() as i32, port.at.1.round() as i32);
    }
    rect.pad(220, 180)
}

fn water_tiles(focus: &ScreenRect) -> Vec<WaterTile> {
    let water = assets::asset(assets::CHART_WATER_A).expect("chart water");
    let samples = [
        (focus.min_x, focus.min_y),
        (focus.max_x, focus.min_y),
        (focus.min_x, focus.max_y),
        (focus.max_x, focus.max_y),
    ];
    let mut min_u = i32::MAX;
    let mut max_u = i32::MIN;
    let mut min_v = i32::MAX;
    let mut max_v = i32::MIN;
    for (x, y) in samples {
        let (u, v) = screen_to_uv(x as f64, y as f64);
        min_u = min_u.min(u.floor() as i32);
        max_u = max_u.max(u.ceil() as i32);
        min_v = min_v.min(v.floor() as i32);
        max_v = max_v.max(v.ceil() as i32);
    }
    let mut tiles = Vec::new();
    for u in (min_u - 1)..=(max_u + 1) {
        for v in (min_v - 1)..=(max_v + 1) {
            let center = water_cell_center(u, v);
            if !focus
                .pad(96, 64)
                .contains(center.0.round() as i32, center.1.round() as i32)
            {
                continue;
            }
            let bottom = water_cell_bottom(u, v);
            tiles.push(WaterTile {
                u,
                v,
                asset_id: chart_water_id(u, v),
                origin: sprite_origin(bottom, water.anchor_x, water.anchor_y),
            });
        }
    }
    tiles
}

fn screen_to_uv(sx: f64, sy: f64) -> (f64, f64) {
    let a = sx / f64::from(crate::project::CELL_WIDTH / 2);
    let b = sy / f64::from(crate::project::CELL_HEIGHT / 2);
    ((a + b) / 2.0, (b - a) / 2.0)
}

fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    (dx * dx + dy * dy).sqrt()
}

fn segment_distance(from: (f32, f32), to: (f32, f32), p: (f32, f32)) -> f32 {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let len2 = dx * dx + dy * dy;
    if len2 < 1.0 {
        return dist(from, p);
    }
    let t = ((p.0 - from.0) * dx + (p.1 - from.1) * dy) / len2;
    let t = t.clamp(0.0, 1.0);
    dist((from.0 + dx * t, from.1 + dy * t), p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{chart_to_screen, chart_to_uv, uv_to_screen};
    use portlight_sim::LaneSuitability;

    fn merchant_at(port: Option<&str>) -> Session {
        Session::new("Ada", "merchant", 42, port).expect("game")
    }

    fn docked_port(world: &World) -> Option<&str> {
        if world.voyage.status == VoyageStatus::InPort {
            Some(world.voyage.destination_id.as_str())
        } else {
            None
        }
    }

    #[test]
    fn starting_chart_is_four_mediterranean_ports_and_picker_lanes() {
        let session = merchant_at(None);
        assert_eq!(session.world().voyage.destination_id, "porto_novo");
        let chart = project_chart(&session);
        let ids: Vec<_> = chart.ports.iter().map(|port| port.id.as_str()).collect();
        assert_eq!(ids.len(), 4);
        for id in ["porto_novo", "al_manar", "silva_bay", "corsairs_rest"] {
            assert!(ids.contains(&id), "{id}");
        }
        let here = chart.ports.iter().find(|port| port.is_here).unwrap();
        assert_eq!(here.id, "porto_novo");
        assert_eq!(here.chart_x, 18);
        assert_eq!(here.chart_y, 8);
        assert!((here.at.0 - 1629.2).abs() < 0.2);
        assert!((here.at.1 - 362.0).abs() < 0.2);
        assert_eq!(here.marker_id, PORT_MARKER);
        assert_eq!(here.badge, Some('S'));
        assert!((here.sprite_origin.0 - (here.at.0 - 64.0)).abs() < 0.01);
        assert!((here.sprite_origin.1 - (here.at.1 - 95.0)).abs() < 0.01);

        let sim = session.sail_lanes();
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

        let grain = chart
            .lanes
            .iter()
            .find(|lane| lane.destination_id == "al_manar")
            .unwrap();
        assert_eq!(grain.distance, 24);
        assert!((grain.danger - 0.12).abs() < 1e-9);
        assert_eq!(grain.min_ship_class, "sloop");
        assert_eq!(grain.estimated_days, 3);
        assert_eq!(grain.lore_name, "The Grain Road");
        assert!(lane_inspect(grain).contains("24"));
        assert!(lane_inspect(grain).contains("0.12"));
        assert!(lane_inspect(grain).contains("3 days"));

        let ironhaven = chart
            .lanes
            .iter()
            .find(|lane| lane.destination_id == "ironhaven")
            .unwrap();
        assert_eq!(ironhaven.suitability, LaneSuitability::Warning);
        assert_eq!(ironhaven.estimated_days, 4);
        assert!(!ironhaven.destination_on_chart);
        assert_eq!(chart.ship.footprint, 1);
        assert_eq!(chart.ship.facing, Facing::F1);
        assert_eq!(chart.ship.asset_id, "ship_sloop_f1");
        assert!(chart.ship.docked);
        assert!((chart.ship.at.0 - (here.at.0 + DOCKED_OFFSET_X)).abs() < 0.01);
        assert!((chart.ship.at.1 - (here.at.1 + DOCKED_OFFSET_Y)).abs() < 0.01);
        assert!(chart.ship.at.1 > here.at.1);
        assert!(chart.leg.is_none());
        assert!(!chart.tiles.is_empty());
    }

    #[test]
    fn anchors_round_trip_for_every_port() {
        let session = merchant_at(None);
        let world = session.world();
        assert_eq!(world.ports.len(), 20);
        for port in &world.ports {
            let (u, v) = chart_to_uv(port.map_x as f64, port.map_y as f64);
            let (sx, sy) = uv_to_screen(u, v);
            let (got_x, got_y) = chart_to_screen_f(port.map_x as f64, port.map_y as f64);
            assert!((sx as f32 - got_x).abs() < 1e-3);
            assert!((sy as f32 - got_y).abs() < 1e-3);
            let (x, y) = crate::project::uv_to_chart(u, v);
            assert!((x - port.map_x as f64).abs() < 1e-6);
            assert!((y - port.map_y as f64).abs() < 1e-6);
        }
    }

    #[test]
    fn five_mediterranean_sloop_lanes_exist_in_the_sim() {
        let session = merchant_at(None);
        let world = session.world();
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
        let session = merchant_at(Some("corsairs_rest"));
        let chart = project_chart(&session);
        let stormwall = chart
            .lanes
            .iter()
            .find(|lane| lane.destination_id == "stormwall")
            .expect("blocked lane listed");
        assert_eq!(stormwall.suitability, LaneSuitability::Blocked);
        assert!(stormwall
            .suitability_note
            .as_deref()
            .unwrap()
            .starts_with("BLOCKED"));
        assert_eq!(stormwall.color, lane_color(LaneSuitability::Blocked));
    }

    #[test]
    fn a_port_with_no_lane_does_not_offer_depart() {
        let session = merchant_at(Some("corsairs_rest"));
        assert_eq!(press_port(&session, "al_manar"), PortPress::NoLane);
        assert_eq!(
            press_port(&session, "porto_novo"),
            PortPress::Depart("porto_novo".into())
        );
        assert_eq!(press_port(&session, "corsairs_rest"), PortPress::OpenHere);
    }

    #[test]
    fn focus_keeps_the_mediterranean_and_lets_distant_lanes_run_off() {
        let session = merchant_at(None);
        let chart = project_chart(&session);
        for port in &chart.ports {
            assert!(chart
                .focus
                .contains(port.at.0.round() as i32, port.at.1.round() as i32));
        }
        let ironhaven = session.world().port("ironhaven").unwrap();
        let (x, y) = chart_to_screen(ironhaven.map_x, ironhaven.map_y);
        assert!(!chart.focus.contains(x, y));
    }

    #[test]
    fn grain_road_faces_bucket_seven_and_rests_on_the_lane() {
        let mut session = Session::new("Ada", "merchant", FIRST_PLAYABLE_SEED, None).unwrap();
        assert_eq!(session.world().captain.silver, 550);
        session.depart("al_manar").unwrap();
        assert_eq!(session.world().captain.silver, 547);
        assert_eq!(session.world().voyage.status, VoyageStatus::AtSea);
        assert_eq!(session.world().voyage.distance, 24);
        let chart = project_chart(&session);
        assert!(chart.lanes.is_empty());
        let leg = chart.leg.expect("underway");
        assert_eq!(leg.origin_id, "porto_novo");
        assert_eq!(leg.destination_id, "al_manar");
        assert_eq!(chart.ship.facing, Facing::F7);
        assert_eq!(chart.ship.asset_id, "ship_sloop_f7");
        assert_eq!(chart.ship.footprint, 1);
        assert!(!chart.ship.docked);
        assert!((chart.ship.chart_x - 18.0).abs() < 1e-6);
        assert!((chart.ship.chart_y - 8.0).abs() < 1e-6);

        session.advance().unwrap();
        let chart = project_chart(&session);
        assert!(chart.ship.chart_x > 18.0);
        assert!(chart.ship.chart_x <= 24.0);
        assert_eq!(chart.ship.facing, Facing::F7);
        let progress = session.world().voyage.progress as f64;
        let distance = session.world().voyage.distance as f64;
        let t = (progress / distance).clamp(0.0, 1.0);
        let expect_x = 18.0 + (24.0 - 18.0) * t;
        let expect_y = 8.0 + (6.0 - 8.0) * t;
        assert!((chart.ship.chart_x - expect_x).abs() < 1e-6);
        assert!((chart.ship.chart_y - expect_y).abs() < 1e-6);
    }

    #[test]
    fn projecting_the_chart_does_not_change_the_sim() {
        let mut with_view = Session::new("Ada", "merchant", 9, None).unwrap();
        let mut bare = Session::new("Ada", "merchant", 9, None).unwrap();
        with_view.depart("silva_bay").unwrap();
        bare.depart("silva_bay").unwrap();
        for _ in 0..6 {
            let _ = project_chart(&with_view);
            with_view.advance().unwrap();
            bare.advance().unwrap();
        }
        assert_eq!(
            format!("{:?}", with_view.world()),
            format!("{:?}", bare.world())
        );
        assert_eq!(with_view.trade_seq(), bare.trade_seq());
        assert_eq!(with_view.victory(), bare.victory());
    }

    #[test]
    fn pending_duel_refuses_advance_without_a_sim_call() {
        let mut world = merchant_at(None).world().clone();
        assert!(advance_refusal(world.pending_duel.as_ref()).is_none());
        world.pending_duel = Some(PendingDuel {
            captain_id: "scarlet_ana".into(),
            captain_name: "Scarlet Ana".into(),
            faction_id: "corsairs".into(),
            personality: "bold".into(),
            strength: 4,
            region: "Mediterranean".into(),
        });
        let day = world.day;
        assert!(advance_refusal(world.pending_duel.as_ref()).is_some());
        assert_eq!(world.day, day);
    }

    #[test]
    fn first_playable_sails_to_al_manar_and_trades() {
        let mut session = Session::new(
            FIRST_PLAYABLE_NAME,
            FIRST_PLAYABLE_CAPTAIN,
            FIRST_PLAYABLE_SEED,
            None,
        )
        .unwrap();
        assert_eq!(docked_port(session.world()), Some("porto_novo"));
        session.buy("grain", 5).unwrap();
        session.depart("al_manar").unwrap();
        let mut docked = false;
        for _ in 0..12 {
            assert!(
                session.world().pending_duel.is_none(),
                "seed {FIRST_PLAYABLE_SEED} hit a duel"
            );
            session.advance().unwrap();
            if docked_port(session.world()) == Some("al_manar") {
                assert_eq!(session.world().voyage.status, VoyageStatus::InPort);
                docked = true;
                break;
            }
        }
        assert!(docked, "did not arrive");
        let chart = project_chart(&session);
        let here = chart
            .ports
            .iter()
            .find(|port| port.id == "al_manar")
            .unwrap();
        assert!(here.is_here);
        assert!(chart.ship.docked);
        assert!((chart.ship.at.0 - (here.at.0 + 32.0)).abs() < 0.05);
        assert!((chart.ship.at.1 - (here.at.1 + 16.0)).abs() < 0.05);
        let held: i64 = session
            .world()
            .captain
            .cargo
            .iter()
            .filter(|item| item.good_id == "grain")
            .map(|item| item.quantity)
            .sum();
        if held > 0 {
            session.sell("grain", held.min(5)).unwrap();
        }
        session.buy("spice", 1).unwrap();
        session.sell("spice", 1).unwrap();
        let chart = project_chart(&session);
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
