//! Counting-house silver accounts (Rust only; the Python game has no bank).
//!
//! The seven ports with a commercial warehouse tier keep a counting house.
//! Deposits are free with a 50 silver minimum. Withdrawing at the same port
//! costs 1% (minimum 1); drawing on another port's account by letter costs
//! 5% (minimum 5). Fees come out of the amount drawn. No interest, no
//! upkeep. Banked silver is outside `captain.silver`, so the wreck, plunder
//! and credit default never reach it, and nothing spends it automatically.

use crate::content;
use crate::error::SimError;
use crate::model::{BankAccount, Captain, InfrastructureRecord};

/// The smallest deposit.
pub const BANK_MINIMUM: i64 = 50;

/// True where a counting house stands: ports offering the commercial
/// warehouse tier.
pub fn has_counting_house(port_id: &str) -> bool {
    content::content()
        .port_warehouse_tiers(port_id)
        .iter()
        .any(|tier| tier == "commercial")
}

/// Ports with a counting house, in port-id order.
pub fn counting_house_ports() -> Vec<String> {
    let mut ports: Vec<String> = content::content()
        .infrastructure
        .port_warehouse_tiers
        .keys()
        .filter(|port| has_counting_house(port))
        .cloned()
        .collect();
    ports.sort();
    ports
}

/// 1% of `amount`, rounded down, at least 1.
pub fn local_fee(amount: i64) -> i64 {
    (amount / 100).max(1)
}

/// 5% of `amount`, rounded down, at least 5.
pub fn letter_fee(amount: i64) -> i64 {
    (amount.max(0) / 20).max(5)
}

pub fn balance_at(state: &InfrastructureRecord, port_id: &str) -> i64 {
    state
        .bank
        .iter()
        .find(|account| account.port_id == port_id)
        .map_or(0, |account| account.balance)
}

fn port_name(port_id: &str) -> String {
    content::content()
        .port(port_id)
        .map(|port| port.name.clone())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| port_id.to_string())
}

/// Move `amount` carried silver into the account at `port_id`. Returns the
/// new balance there.
pub fn deposit(
    state: &mut InfrastructureRecord,
    captain: &mut Captain,
    port_id: &str,
    amount: i64,
    day: i64,
) -> Result<i64, SimError> {
    if !has_counting_house(port_id) {
        return Err(SimError::NoCountingHouse);
    }
    if amount < BANK_MINIMUM {
        return Err(SimError::BelowBankMinimum {
            minimum: BANK_MINIMUM,
        });
    }
    if captain.silver < amount {
        return Err(SimError::InsufficientSilver {
            have: captain.silver,
        });
    }
    captain.silver -= amount;
    let index = match state
        .bank
        .binary_search_by(|account| account.port_id.as_str().cmp(port_id))
    {
        Ok(index) => index,
        Err(index) => {
            state.bank.insert(
                index,
                BankAccount {
                    port_id: port_id.to_string(),
                    balance: 0,
                    opened_day: day,
                },
            );
            index
        }
    };
    state.bank[index].balance += amount;
    Ok(state.bank[index].balance)
}

/// Take `amount` from the account at `source`; the fee comes out of it.
/// Returns `(received, fee)`.
fn draw(
    state: &mut InfrastructureRecord,
    captain: &mut Captain,
    source: &str,
    amount: i64,
    fee: i64,
) -> Result<(i64, i64), SimError> {
    let have = balance_at(state, source);
    if amount <= 0 || have < amount {
        return Err(SimError::InsufficientBalance {
            port: port_name(source),
            have,
        });
    }
    if amount <= fee {
        return Err(SimError::BelowBankFee { fee });
    }
    if let Some(index) = state
        .bank
        .iter()
        .position(|account| account.port_id == source)
    {
        state.bank[index].balance -= amount;
        if state.bank[index].balance == 0 {
            state.bank.remove(index);
        }
    }
    let received = amount - fee;
    captain.silver += received;
    Ok((received, fee))
}

/// Withdraw from the account at this port (fee 1%, minimum 1).
pub fn withdraw(
    state: &mut InfrastructureRecord,
    captain: &mut Captain,
    port_id: &str,
    amount: i64,
) -> Result<(i64, i64), SimError> {
    if !has_counting_house(port_id) {
        return Err(SimError::NoCountingHouse);
    }
    draw(state, captain, port_id, amount, local_fee(amount))
}

/// Draw on another port's account by letter of credit (fee 5%, minimum 5).
pub fn draw_letter(
    state: &mut InfrastructureRecord,
    captain: &mut Captain,
    here: &str,
    source: &str,
    amount: i64,
) -> Result<(i64, i64), SimError> {
    if !has_counting_house(here) {
        return Err(SimError::NoCountingHouse);
    }
    if source == here {
        return Err(SimError::LetterOnLocalAccount);
    }
    draw(state, captain, source, amount, letter_fee(amount))
}

/// `Banked {N} silver at {Port}.`
pub fn deposit_message(amount: i64, port_id: &str) -> String {
    format!("Banked {amount} silver at {}.", port_name(port_id))
}

/// `Withdrew {N} silver - fee {F}.` (the fee clause goes at 0).
pub fn withdraw_message(amount: i64, fee: i64) -> String {
    if fee > 0 {
        format!("Withdrew {amount} silver - fee {fee}.")
    } else {
        format!("Withdrew {amount} silver.")
    }
}

/// `Drew {N} silver from {Port} by letter - fee {F}.` (the fee clause goes
/// at 0).
pub fn letter_message(amount: i64, source: &str, fee: i64) -> String {
    let from = port_name(source);
    if fee > 0 {
        format!("Drew {amount} silver from {from} by letter - fee {fee}.")
    } else {
        format!("Drew {amount} silver from {from} by letter.")
    }
}
