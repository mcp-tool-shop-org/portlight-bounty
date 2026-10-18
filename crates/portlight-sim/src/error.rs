//! Player-facing errors. `Display` text matches the Python strings the
//! goldens already lock, so a UI can match on the type and still show the
//! same sentence.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimError {
    NotDocked,
    NoShip,
    AlreadyAtSea,
    AlreadyAtThisPort,
    NoRoute {
        from: String,
        to: String,
    },
    /// The `BLOCKED:` suitability note `depart` returns unchanged.
    RouteBlocked(String),
    CrewMinimum {
        need: i64,
        have: i64,
    },
    NeedSilver {
        need: i64,
        have: i64,
    },
    /// A hunt needs more hull than the patched floor a tow leaves.
    HullTooLowToHunt {
        need: i64,
        have: i64,
    },
    NeedPortFee {
        fee: i64,
        have: i64,
    },
    QuantityMustBePositive,
    QuantityMustBeAPositiveNumber,
    GoodNotAvailable {
        good_id: String,
        port_name: String,
        suggestion: Option<String>,
    },
    OnlyStock {
        stock: i64,
        good_id: String,
    },
    NotEnoughCargoSpace,
    Harbormaster {
        good_id: String,
    },
    PortDoesNotTrade {
        port_name: String,
        good_id: String,
    },
    OnlyHave {
        have: i64,
        good_id: String,
    },
    NotArrivedYet,
    UnknownCaptainType(String),
    UnknownShip(String),
    NoActiveGame,
    UsageNew,
    UsageBuy,
    UsageSell,
    UsageDepart,
    UsageHire,
    UsageProvision,
    UsageWork,
    UsageDuel,
    InvalidNumber(String),
    UnknownCommand(String),
    MustBeDockedToHire,
    CrewFull,
    UnknownRole(String),
    RoleMaximum {
        name: String,
        max: i64,
    },
    NeedCrewSilver {
        cost: i64,
        count: i64,
        name: String,
        each: i64,
        have: i64,
    },
    MustBeDockedToProvision,
    MustBeDockedToWork,
    NeedProvisions {
        cost: i64,
        days: i64,
        per_day: i64,
        have: i64,
    },
    NoPendingDuel,
    InvalidStance(String),
    TooFewStances,
    UsageBuyInfrastructure,
    UsageTakeCredit,
    UsageBuyInsurance,
    UsageDeposit,
    UsageWithdraw,
    UsageRepayCredit,
    /// Python `SaveVersionError` when the file is newer than this build.
    SaveVersion {
        found: i64,
        supported: i64,
    },
    /// Python `SaveVersionError` when the migration chain cannot reach v12.
    SaveMigration {
        found: i64,
        supported: i64,
    },
    SaveIo(String),
    SaveCorrupt,
    OfferNotFound,
    TooManyContracts,
    NoActiveContract,
    ContractNotFulfilled,
    UsageAcceptContract,
    UsageCompleteContract,
    NoActiveEncounter,
    NotInNavalCombat,
    NotBoarding,
    NotInPersonalCombat,
    ChooseApproach,
    InvalidAction(String),
    NoPirateCaptain,
    UnknownPirate(String),
    UsageEncounter,
    UsageNaval,
    UsageBoard,
    UsageFight,
    UsageCapture,
    CannotCapture(String),
    MustBeDockedToTrain,
    MustBeDockedToLearnSkill,
    MustBeDockedToRecruit,
    UsageTrain,
    UsageSkill,
    UsageRecruit,
    UsageAgency,
    UsageRemember,
    /// Counting house: not docked.
    MustBeDockedToBank,
    /// Counting house: this port has none.
    NoCountingHouse,
    /// Counting house: a deposit below the minimum.
    BelowBankMinimum {
        minimum: i64,
    },
    /// Counting house: the account holds less than asked (port display name).
    InsufficientBalance {
        port: String,
        have: i64,
    },
    /// Counting house: less silver carried than the deposit.
    InsufficientSilver {
        have: i64,
    },
    /// Counting house: the fee would take the whole amount.
    BelowBankFee {
        fee: i64,
    },
    /// Counting house: a letter drawn on the account at this port.
    LetterOnLocalAccount,
    UsageBankDeposit,
    UsageBankWithdraw,
    UsageBankLetter,
    /// A sentence copied from the Python rules engine.
    Rejected(String),
    /// A Python sentence this area surfaces directly.
    Sentence(String),
}

impl fmt::Display for SimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotDocked => write!(f, "Not docked at a port"),
            Self::NoShip => write!(f, "No ship"),
            Self::AlreadyAtSea => write!(f, "Already at sea"),
            Self::AlreadyAtThisPort => write!(f, "Already at this port"),
            Self::NoRoute { from, to } => write!(f, "No route from {from} to {to}"),
            Self::RouteBlocked(note) => write!(f, "{note}"),
            Self::CrewMinimum { need, have } => {
                write!(f, "Need at least {need} crew to sail, have {have}. Hire crew first.")
            }
            Self::NeedSilver { need, have } => write!(f, "Need {need} silver, have {have}"),
            Self::HullTooLowToHunt { need, have } => {
                write!(f, "Need at least {need} hull to hunt, have {have}. Repair first.")
            }
            Self::NeedPortFee { fee, have } => {
                write!(f, "Need {fee} silver for port fee, have {have}")
            }
            Self::QuantityMustBePositive => write!(f, "Quantity must be positive"),
            Self::QuantityMustBeAPositiveNumber => write!(f, "Quantity must be a positive number."),
            Self::GoodNotAvailable {
                good_id,
                port_name,
                suggestion: Some(suggestion),
            } => write!(
                f,
                "{good_id} not available at {port_name} -- did you mean: {suggestion}"
            ),
            Self::GoodNotAvailable {
                good_id,
                port_name,
                suggestion: None,
            } => write!(f, "{good_id} not available at {port_name}"),
            Self::OnlyStock { stock, good_id } => {
                write!(f, "Only {stock} units available -- try: buy {good_id} {stock}")
            }
            Self::NotEnoughCargoSpace => write!(f, "Not enough cargo space"),
            Self::Harbormaster { good_id } => write!(
                f,
                "The harbormaster won't touch {good_id}. Try somewhere less official."
            ),
            Self::PortDoesNotTrade { port_name, good_id } => {
                write!(f, "{port_name} doesn't trade {good_id}")
            }
            Self::OnlyHave { have, good_id } => write!(f, "Only have {have} units of {good_id}"),
            Self::NotArrivedYet => write!(f, "Not arrived yet"),
            Self::UnknownCaptainType(kind) => write!(f, "Unknown captain type: {kind}"),
            Self::UnknownShip(id) => write!(f, "Unknown ship: {id}"),
            Self::NoActiveGame => write!(f, "No active game"),
            Self::UsageNew => write!(f, "Usage: new <captain_type> <name> <seed> [port]"),
            Self::UsageBuy => write!(f, "Usage: buy <good> <qty>"),
            Self::UsageSell => write!(f, "Usage: sell <good> <qty>"),
            Self::UsageDepart => write!(f, "Usage: depart <port_id>"),
            Self::UsageHire => write!(f, "Usage: hire <count> [role]"),
            Self::UsageProvision => write!(f, "Usage: provision <days>"),
            Self::UsageWork => write!(f, "Usage: work"),
            Self::UsageDuel => write!(f, "Usage: duel <stance>[,<stance>...]"),
            Self::InvalidNumber(token) => write!(f, "Invalid number: {token}"),
            Self::UnknownCommand(cmd) => write!(f, "Unknown command: {cmd}"),
            Self::MustBeDockedToHire => write!(f, "Must be docked to hire crew"),
            Self::CrewFull => write!(f, "Crew is already full"),
            Self::UnknownRole(role) => write!(
                f,
                "Unknown role: {role}. Valid: sailor, gunner, navigator, surgeon, marine, quartermaster"
            ),
            Self::RoleMaximum { name, max } => {
                write!(f, "Already at maximum {name}s ({max})")
            }
            Self::NeedCrewSilver {
                cost,
                count,
                name,
                each,
                have,
            } => write!(
                f,
                "Need {cost} silver for {count} {name}(s) ({each}/each), have {have}"
            ),
            Self::MustBeDockedToProvision => write!(f, "Must be docked to provision"),
            Self::MustBeDockedToWork => write!(f, "Must be docked to work the docks."),
            Self::NeedProvisions {
                cost,
                days,
                per_day,
                have,
            } => write!(
                f,
                "Need {cost} silver for {days} days of provisions ({per_day}/day here), have {have}"
            ),
            Self::NoPendingDuel => write!(
                f,
                "No pirate has challenged you. Duels happen during pirate encounters at sea."
            ),
            Self::InvalidStance(stance) => {
                write!(f, "Invalid stance: {stance}. Use: thrust, slash, parry")
            }
            Self::TooFewStances => write!(
                f,
                "Provide at least 3 stances (e.g. thrust,parry,slash,thrust,parry)"
            ),
            Self::SaveVersion { found, supported } => write!(
                f,
                "Save file version {found} is newer than supported version {supported}. Update Portlight to load this save."
            ),
            Self::SaveMigration { found, supported } => write!(
                f,
                "Migration chain broken: reached version {found}, expected {supported}"
            ),
            Self::SaveIo(message) => write!(f, "{message}"),
            Self::SaveCorrupt => write!(f, "Save file is corrupt"),
            Self::OfferNotFound => write!(f, "Offer not found"),
            Self::TooManyContracts => write!(f, "Too many active contracts (max 3)"),
            Self::NoActiveContract => write!(f, "No active contract with that ID"),
            Self::ContractNotFulfilled => write!(f, "Contract is not yet fulfilled"),
            Self::UsageAcceptContract => write!(f, "Usage: accept_contract <offer_id>"),
            Self::UsageCompleteContract => write!(f, "Usage: complete_contract <offer_id>"),
            Self::NoActiveEncounter => write!(
                f,
                "No active encounter. Encounters happen during pirate encounters at sea."
            ),
            Self::NotInNavalCombat => write!(f, "Not in naval combat."),
            Self::NotBoarding => write!(f, "Not boarding."),
            Self::NotInPersonalCombat => write!(f, "Not in personal combat."),
            Self::ChooseApproach => write!(f, "Choose: negotiate, flee, or fight"),
            Self::InvalidAction(actions) => {
                write!(f, "Invalid action. Available: {actions}")
            }
            Self::NoPirateCaptain => write!(f, "No pirate captain in this region."),
            Self::UnknownPirate(id) => write!(f, "Unknown pirate captain: {id}"),
            Self::UsageEncounter => write!(
                f,
                "Usage: encounter <negotiate|flee|fight> [captain_id|strength:N]"
            ),
            Self::UsageNaval => write!(f, "Usage: naval <action>"),
            Self::UsageBoard => write!(f, "Usage: board"),
            Self::UsageFight => write!(f, "Usage: fight <action>"),
            Self::UsageCapture => write!(f, "Usage: capture <crew>"),
            Self::CannotCapture(reason) => write!(f, "Cannot capture: {reason}"),
            Self::MustBeDockedToTrain => write!(f, "Must be docked at a port to train."),
            Self::MustBeDockedToLearnSkill => write!(f, "Must be docked to learn skills."),
            Self::MustBeDockedToRecruit => write!(f, "Must be docked to recruit companions."),
            Self::UsageTrain => write!(f, "Usage: train <style_id>"),
            Self::UsageSkill => write!(f, "Usage: skill <skill_id>"),
            Self::UsageRecruit => write!(f, "Usage: recruit <companion_id>"),
            Self::UsageAgency => write!(f, "Usage: agency"),
            Self::UsageRemember => write!(f, "Usage: remember <captain_id> <outcome>"),
            Self::UsageBuyInfrastructure => write!(
                f,
                "Usage: buy_infrastructure warehouse <tier> | broker <region> <tier> | license <id> | dry_dock [ship]"
            ),
            Self::UsageTakeCredit => write!(f, "Usage: take_credit <tier> <amount>"),
            Self::UsageBuyInsurance => write!(
                f,
                "Usage: buy_insurance <policy_id> [target_id] [origin] [destination]"
            ),
            Self::UsageDeposit => write!(f, "Usage: deposit <good> <qty>"),
            Self::UsageWithdraw => write!(f, "Usage: withdraw <good> <qty> [source_port]"),
            Self::UsageRepayCredit => write!(f, "Usage: repay_credit <amount>"),
            Self::MustBeDockedToBank => write!(f, "Must be docked to use a counting house."),
            Self::NoCountingHouse => write!(f, "No counting house at this port."),
            Self::BelowBankMinimum { minimum } => {
                write!(f, "Deposit at least {minimum} silver.")
            }
            Self::InsufficientBalance { port, have } => {
                write!(f, "Only {have} silver banked at {port}.")
            }
            Self::InsufficientSilver { have } => write!(f, "Only {have} silver on hand."),
            Self::BelowBankFee { fee } => {
                write!(f, "Too little to cover the {fee} silver fee.")
            }
            Self::LetterOnLocalAccount => {
                write!(f, "That account is here - withdraw instead.")
            }
            Self::UsageBankDeposit => write!(f, "Usage: bank_deposit <amount>"),
            Self::UsageBankWithdraw => write!(f, "Usage: bank_withdraw <amount>"),
            Self::UsageBankLetter => write!(f, "Usage: bank_letter <source_port> <amount>"),
            Self::Rejected(message) => write!(f, "{message}"),
            Self::Sentence(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for SimError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_sim_error_variants_format_cleanly() {
        let samples = vec![
            SimError::NotDocked,
            SimError::NoShip,
            SimError::AlreadyAtSea,
            SimError::AlreadyAtThisPort,
            SimError::NoRoute {
                from: "porto_novo".to_string(),
                to: "jade_port".to_string(),
            },
            SimError::RouteBlocked("BLOCKED".to_string()),
            SimError::CrewMinimum { need: 5, have: 3 },
            SimError::NeedSilver {
                need: 100,
                have: 50,
            },
            SimError::HullTooLowToHunt { need: 20, have: 10 },
            SimError::NeedPortFee { fee: 15, have: 5 },
            SimError::QuantityMustBePositive,
            SimError::QuantityMustBeAPositiveNumber,
            SimError::GoodNotAvailable {
                good_id: "silks".to_string(),
                port_name: "Porto Novo".to_string(),
                suggestion: Some("silk".to_string()),
            },
            SimError::GoodNotAvailable {
                good_id: "uranium".to_string(),
                port_name: "Porto Novo".to_string(),
                suggestion: None,
            },
            SimError::OnlyStock {
                stock: 4,
                good_id: "grain".to_string(),
            },
            SimError::NotEnoughCargoSpace,
            SimError::Harbormaster {
                good_id: "opium".to_string(),
            },
            SimError::PortDoesNotTrade {
                port_name: "Porto Novo".to_string(),
                good_id: "silk".to_string(),
            },
            SimError::OnlyHave {
                have: 2,
                good_id: "grain".to_string(),
            },
            SimError::NotArrivedYet,
            SimError::UnknownCaptainType("wizard".to_string()),
            SimError::UnknownShip("submarine".to_string()),
            SimError::NoActiveGame,
            SimError::UsageNew,
            SimError::UsageBuy,
            SimError::UsageSell,
            SimError::UsageDepart,
            SimError::UsageHire,
            SimError::UsageProvision,
            SimError::UsageWork,
            SimError::UsageDuel,
            SimError::InvalidNumber("abc".to_string()),
            SimError::UnknownCommand("fly".to_string()),
            SimError::MustBeDockedToHire,
            SimError::CrewFull,
            SimError::UnknownRole("cook".to_string()),
            SimError::RoleMaximum {
                name: "surgeon".to_string(),
                max: 1,
            },
            SimError::NeedCrewSilver {
                cost: 40,
                count: 2,
                name: "gunner".to_string(),
                each: 20,
                have: 10,
            },
            SimError::MustBeDockedToProvision,
            SimError::MustBeDockedToWork,
            SimError::NeedProvisions {
                cost: 20,
                days: 10,
                per_day: 2,
                have: 5,
            },
            SimError::NoPendingDuel,
            SimError::InvalidStance("jump".to_string()),
            SimError::TooFewStances,
            SimError::UsageBuyInfrastructure,
            SimError::UsageTakeCredit,
            SimError::UsageBuyInsurance,
            SimError::UsageDeposit,
            SimError::UsageWithdraw,
            SimError::UsageRepayCredit,
            SimError::SaveVersion {
                found: 99,
                supported: 13,
            },
            SimError::SaveMigration {
                found: 8,
                supported: 12,
            },
            SimError::SaveIo("disk full".to_string()),
            SimError::SaveCorrupt,
            SimError::OfferNotFound,
            SimError::TooManyContracts,
            SimError::NoActiveContract,
            SimError::ContractNotFulfilled,
            SimError::UsageAcceptContract,
            SimError::UsageCompleteContract,
            SimError::NoActiveEncounter,
            SimError::NotInNavalCombat,
            SimError::NotBoarding,
            SimError::NotInPersonalCombat,
            SimError::ChooseApproach,
            SimError::InvalidAction("dance".to_string()),
            SimError::NoPirateCaptain,
            SimError::UnknownPirate("blackbeard".to_string()),
            SimError::UsageEncounter,
            SimError::UsageNaval,
            SimError::UsageBoard,
            SimError::UsageFight,
            SimError::UsageCapture,
            SimError::CannotCapture("hull too low".to_string()),
            SimError::MustBeDockedToTrain,
            SimError::MustBeDockedToLearnSkill,
            SimError::MustBeDockedToRecruit,
            SimError::UsageTrain,
            SimError::UsageSkill,
            SimError::UsageRecruit,
            SimError::UsageAgency,
            SimError::UsageRemember,
            SimError::MustBeDockedToBank,
            SimError::NoCountingHouse,
            SimError::BelowBankMinimum { minimum: 50 },
            SimError::InsufficientBalance {
                port: "Porto Novo".to_string(),
                have: 10,
            },
            SimError::InsufficientSilver { have: 25 },
            SimError::BelowBankFee { fee: 5 },
            SimError::LetterOnLocalAccount,
            SimError::UsageBankDeposit,
            SimError::UsageBankWithdraw,
            SimError::UsageBankLetter,
            SimError::Rejected("Rejected".to_string()),
            SimError::Sentence("Sentence".to_string()),
        ];
        for err in samples {
            assert!(!err.to_string().is_empty());
        }
    }
}
