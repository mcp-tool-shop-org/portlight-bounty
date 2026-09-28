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
    /// A sentence copied from the Python rules engine.
    Rejected(String),
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
            Self::Rejected(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for SimError {}
