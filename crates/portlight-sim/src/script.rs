//! Scripted session over [`crate::session::Session`].
//!
//! The commands are a thin wrapper: each one calls the same public method a
//! front end would call. Parity goldens therefore cover the stepwise API.

use crate::session::Session;
use crate::snapshot::{self, LogEntry, Snapshot};

pub fn run_script(script: &str) -> Snapshot {
    let mut session: Option<Session> = None;
    let mut log = Vec::new();
    for line in script.lines() {
        let raw = line.trim();
        if raw.is_empty() || raw.starts_with('#') {
            continue;
        }
        let tokens = tokenize(raw);
        let mut entry = LogEntry::new(raw);
        match dispatch(&mut session, &tokens, &mut entry) {
            Ok(()) => log.push(entry),
            Err(err) => {
                entry.error = Some(err);
                log.push(entry);
                break;
            }
        }
    }
    match session {
        Some(session) => {
            snapshot::capture(session.world(), session.trade_seq(), session.books(), log)
        }
        None => snapshot::empty(log),
    }
}

fn dispatch(
    session: &mut Option<Session>,
    tokens: &[String],
    entry: &mut LogEntry,
) -> Result<(), String> {
    let cmd = tokens.first().map(String::as_str).unwrap_or("");
    match cmd {
        "new" => {
            if tokens.len() < 4 {
                return Err("Usage: new <captain_type> <name> <seed> [port]".to_string());
            }
            let captain_type = &tokens[1];
            let name = &tokens[2];
            let seed: i128 = tokens[3]
                .parse()
                .map_err(|_| format!("Invalid number: {}", tokens[3]))?;
            let port = tokens.get(4).map(String::as_str);
            *session = Some(Session::new(name, captain_type, seed, port)?);
            Ok(())
        }
        "buy" => {
            let session = session
                .as_mut()
                .ok_or_else(|| "No active game".to_string())?;
            if tokens.len() != 3 {
                return Err("Usage: buy <good> <qty>".to_string());
            }
            let qty = parse_qty(&tokens[2])?;
            let receipt = session.buy(&tokens[1], qty)?;
            entry.receipt = Some(snapshot::from_receipt(&receipt));
            Ok(())
        }
        "sell" => {
            let session = session
                .as_mut()
                .ok_or_else(|| "No active game".to_string())?;
            if tokens.len() != 3 {
                return Err("Usage: sell <good> <qty>".to_string());
            }
            let qty = parse_qty(&tokens[2])?;
            let receipt = session.sell(&tokens[1], qty)?;
            entry.receipt = Some(snapshot::from_receipt(&receipt));
            Ok(())
        }
        "depart" => {
            let session = session
                .as_mut()
                .ok_or_else(|| "No active game".to_string())?;
            if tokens.len() != 2 {
                return Err("Usage: depart <port_id>".to_string());
            }
            session.depart(&tokens[1])
        }
        "advance" => {
            let session = session
                .as_mut()
                .ok_or_else(|| "No active game".to_string())?;
            let turn = session.advance()?;
            entry.events = turn.events.iter().map(snapshot::from_event).collect();
            entry.shocks = turn.shocks;
            Ok(())
        }
        other => Err(format!("Unknown command: {other}")),
    }
}

fn parse_qty(token: &str) -> Result<i64, String> {
    token
        .parse()
        .map_err(|_| format!("Invalid number: {token}"))
}

fn tokenize(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for ch in line.chars() {
        if ch == '"' {
            quoted = !quoted;
            continue;
        }
        if ch.is_whitespace() && !quoted {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(ch);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_command_stops_the_script() {
        let snap = run_script("new merchant Ada 1\nfly\nbuy grain 1\n");
        assert_eq!(snap.log.len(), 2);
        assert_eq!(snap.log[1].error.as_deref(), Some("Unknown command: fly"));
        assert_eq!(snap.captain.silver, 550);
    }

    #[test]
    fn seed_above_i64_is_accepted() {
        let snap = run_script("new merchant Ada 9223372036854775808\n");
        assert!(snap.log[0].error.is_none(), "{:?}", snap.log[0].error);
        assert_eq!(snap.seed, 1i128 << 63);
    }
}
