//! Thin command line over [`portlight_sim`].
//!
//! `portlight script` runs an action script and prints the canonical state
//! JSON that the parity harness compares with the Python game.

use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

use portlight_sim::{load_snapshot, new_game, run_and_save, run_script};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(cmd) = args.next() else {
        print_help();
        return ExitCode::from(2);
    };
    match cmd.as_str() {
        "help" | "--help" | "-h" => {
            print_help();
            ExitCode::SUCCESS
        }
        "script" => script_cmd(&args.collect::<Vec<_>>()),
        "load" => load_cmd(&args.collect::<Vec<_>>()),
        "new" => new_cmd(&args.collect::<Vec<_>>()),
        other => {
            eprintln!("Unknown command: {other}");
            print_help();
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    println!(
        "\
portlight — trade simulation CLI

Usage:
  portlight new --captain merchant --name Ada --seed 42 [--json]
  portlight script <file> [--save-dir DIR --slot NAME]
  portlight script -          read the script from stdin
  portlight load <dir> <slot> print the snapshot of a version-12 slot

Script commands:
  new <captain_type> <name> <seed> [port]
  buy <good> <qty>
  sell <good> <qty>
  depart <port_id>
  advance
  accept_contract <offer_id>
  complete_contract <offer_id>
  abandon_contract <offer_id>
  repair [points]
  rename_ship <new_name> [ship]
  dock_current_ship
  board_fleet_ship <ship>
  sell_fleet_ship <ship>
  fire <count> [role]
  train <style_id>
  recruit <companion_id>
  skill <skill_id>
  remember <captain_id> <outcome>
  agency
"
    );
}

fn script_cmd(args: &[String]) -> ExitCode {
    let mut file: Option<String> = None;
    let mut save_dir: Option<String> = None;
    let mut slot = "default".to_string();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--save-dir" => {
                index += 1;
                let Some(dir) = args.get(index) else {
                    eprintln!("Usage: portlight script <file> [--save-dir DIR --slot NAME]");
                    return ExitCode::from(2);
                };
                save_dir = Some(dir.clone());
            }
            "--slot" => {
                index += 1;
                let Some(name) = args.get(index) else {
                    eprintln!("Usage: portlight script <file> [--save-dir DIR --slot NAME]");
                    return ExitCode::from(2);
                };
                slot = name.clone();
            }
            other if file.is_none() && !other.starts_with('-') => file = Some(other.to_string()),
            _ => {
                eprintln!("Usage: portlight script <file> [--save-dir DIR --slot NAME]");
                return ExitCode::from(2);
            }
        }
        index += 1;
    }
    let Some(path) = file else {
        eprintln!("Usage: portlight script <file> [--save-dir DIR --slot NAME]");
        return ExitCode::from(2);
    };
    let text = if path == "-" {
        let mut buf = String::new();
        if io::stdin().read_to_string(&mut buf).is_err() {
            eprintln!("Failed to read stdin");
            return ExitCode::from(1);
        }
        buf
    } else {
        match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) => {
                eprintln!("Failed to read {path}: {err}");
                return ExitCode::from(1);
            }
        }
    };
    let snap = if let Some(dir) = save_dir {
        match run_and_save(&text, std::path::Path::new(&dir), &slot) {
            Ok(snap) => snap,
            Err(err) => {
                eprintln!("{err}");
                return ExitCode::from(1);
            }
        }
    } else {
        run_script(&text)
    };
    match serde_json::to_string_pretty(&snap) {
        Ok(json) => println!("{json}"),
        Err(err) => {
            eprintln!("Failed to encode snapshot: {err}");
            return ExitCode::from(1);
        }
    }
    if snap.log.iter().any(|entry| entry.error.is_some()) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn load_cmd(args: &[String]) -> ExitCode {
    if args.len() != 2 {
        eprintln!("Usage: portlight load <dir> <slot>");
        return ExitCode::from(2);
    }
    match load_snapshot(std::path::Path::new(&args[0]), &args[1]) {
        Ok(Some(snap)) => match serde_json::to_string_pretty(&snap) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("Failed to encode snapshot: {err}");
                ExitCode::from(1)
            }
        },
        Ok(None) => {
            eprintln!("No save in slot {}", args[1]);
            ExitCode::from(1)
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn new_cmd(args: &[String]) -> ExitCode {
    let mut captain = "merchant".to_string();
    let mut name = "Captain".to_string();
    let mut seed: i128 = 1;
    let mut json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--captain" => {
                i += 1;
                captain = args.get(i).cloned().unwrap_or_default();
            }
            "--name" => {
                i += 1;
                name = args.get(i).cloned().unwrap_or_default();
            }
            "--seed" => {
                i += 1;
                seed = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(1);
            }
            "--json" => json = true,
            other => {
                eprintln!("Unknown flag: {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }
    let world = match new_game(&name, &captain, seed, None) {
        Ok(world) => world,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::from(1);
        }
    };
    if json {
        let script = format!("new {captain} {name} {seed}\n");
        let snap = run_script(&script);
        println!(
            "{}",
            serde_json::to_string_pretty(&snap).unwrap_or_default()
        );
        return ExitCode::SUCCESS;
    }
    let port = &world.voyage.destination_id;
    let ship = world.captain.ship.as_ref();
    println!(
        "{name} the {captain} — day {}, {} silver, docked at {port}",
        world.day, world.captain.silver
    );
    if let Some(ship) = ship {
        println!(
            "Ship: {} (hull {}/{}, crew {}, hold {})",
            ship.name, ship.hull, ship.hull_max, ship.crew, ship.cargo_capacity
        );
    }
    if let Some(market) = world.port(port) {
        println!("Market at {}:", market.name);
        for slot in &market.market {
            println!(
                "  {:<12} stock {:<4} buy {:<4} sell {}",
                slot.good_id, slot.stock_current, slot.buy_price, slot.sell_price
            );
        }
    }
    ExitCode::SUCCESS
}
