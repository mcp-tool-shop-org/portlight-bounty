//! Story beats fired once from game state.
//!
//! Ported from `engine/narrative.py`. `evaluate_narrative` is what
//! `GameSession.sell` and the sea branch of `GameSession.advance` call.
//! An in-port day does not. The beat texts are the Python catalog.
//! `sacred_cargo` and `forbidden_trade` are defined and never fired:
//! nothing in `GameSession` calls them.

use crate::campaign::HouseBooks;
use crate::content::{self, season_name};
use crate::economy::TradeReceipt;
use crate::model::{
    ContractBoard, InfrastructureRecord, NarrativeBeat, NarrativeState, VoyageStatus, World,
};
use crate::voyage::{EventType, VoyageEvent};

struct BeatDef {
    id: &'static str,
    phase: &'static str,
    title: &'static str,
    text: &'static str,
    flavor: &'static str,
    hint: &'static str,
}

const BEATS: &[BeatDef] = &[
    BeatDef {
        id: "first_trade",
        phase: "the_call",
        title: "The First Deal",
        text: "You count the silver from your first sale. It's not much, but it's yours. Every fortune in history started with a single trade.",
        flavor: "",
        hint: "Watch the market affinities — buy where goods are plentiful, sell where they're scarce.",
    },
    BeatDef {
        id: "first_voyage",
        phase: "the_call",
        title: "Into Open Water",
        text: "The harbor shrinks behind you. The wind fills your sails and the crew settles into their watches. Whatever happens next, you've left the dock.",
        flavor: "",
        hint: "Keep provisions stocked. Running out at sea is a death sentence.",
    },
    BeatDef {
        id: "first_profit",
        phase: "the_call",
        title: "Profit and Promise",
        text: "Your ledger shows a profit for the first time. The crew notices — a captain who can turn silver gets loyalty that gold can't buy.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "new_region",
        phase: "threshold",
        title: "Strange Waters",
        text: "The flags are unfamiliar. The language changes. The goods on the docks are things you've only heard about in tavern stories. You've crossed into a new world.",
        flavor: "",
        hint: "Build standing in new regions by trading consistently. Reputation opens doors.",
    },
    BeatDef {
        id: "first_contract",
        phase: "threshold",
        title: "A Binding Word",
        text: "You sign your name on a contract for the first time. The obligation weighs heavier than any cargo. Arrive on time, and doors open. Fail, and they close.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "ship_upgrade",
        phase: "threshold",
        title: "A Bigger Ship",
        text: "The new ship sits heavy in the water, her hold cavernous compared to your old sloop. Routes that were suicide runs become trade routes. The game just changed.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "survived_storm",
        phase: "tests",
        title: "Through the Tempest",
        text: "The storm broke three spars and swept a man overboard. But you held the wheel, and the ship held together. The crew will never forget this night.",
        flavor: "The sea tests every captain. Those who survive earn something money can't buy.",
        hint: "",
    },
    BeatDef {
        id: "survived_pirates",
        phase: "tests",
        title: "Blood in the Water",
        text: "Pirates spotted your cargo and gave chase. Whether by speed, guile, or luck, you kept your goods and your life. Not everyone on these waters can say the same.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "first_inspection",
        phase: "tests",
        title: "The Customs Man",
        text: "An inspector boards your vessel, ledger in hand. His eyes miss nothing. The nature of your cargo and the cleanness of your record decide what happens next.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "rival_encounter",
        phase: "tests",
        title: "A Familiar Sail",
        text: "You spot a ship you recognize — another captain working the same routes, chasing the same margins. The sea is big, but the profitable corners of it aren't.",
        flavor: "Competition sharpens instinct. Watch what others trade and find the gaps.",
        hint: "",
    },
    BeatDef {
        id: "mentor_wisdom",
        phase: "tests",
        title: "Words from an Old Hand",
        text: "An old captain shares a drink with you in a portside tavern. \"The sea doesn't care about your plans,\" he says. \"She only respects the captains who listen.\"",
        flavor: "",
        hint: "Diversify your routes. Relying on one trade lane is fragile.",
    },
    BeatDef {
        id: "cargo_seized",
        phase: "ordeal",
        title: "Seized",
        text: "They took your cargo. Every crate, inspected and confiscated. Your crew watches in silence as months of work vanish into a customs warehouse. The question isn't whether you'll recover. It's whether you'll try.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "near_bankruptcy",
        phase: "ordeal",
        title: "The Abyss",
        text: "Silver: almost nothing. Provisions: running low. The crew looks at you with eyes that ask whether this is the end. Every great merchant hit bottom once. The difference is what they did next.",
        flavor: "",
        hint: "Small trades, short routes. Rebuild from the ground up. The market always has opportunity.",
    },
    BeatDef {
        id: "contract_failed",
        phase: "ordeal",
        title: "Broken Promise",
        text: "The deadline passed. The goods never arrived. Your name is mud at the exchange, and the trust you built evaporates like morning fog. Reputation is the hardest thing to rebuild.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "first_big_contract",
        phase: "reward",
        title: "The Big Score",
        text: "A contract worth more than everything you've earned so far. The kind of deal that turns a trader into a merchant house. All those small runs were preparation for this moment.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "east_indies_arrival",
        phase: "reward",
        title: "The Spice Quarter",
        text: "The East Indies. Every merchant's dream, every navigator's test. The air smells of spice and possibility. Silk and porcelain fill warehouses that stretch to the horizon. You've arrived.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "south_seas_discovery",
        phase: "reward",
        title: "Beyond the Charts",
        text: "The South Seas. Your charts have blank spaces here. Pearls gleam in the shallows, volcanic islands smoke on the horizon, and kings you've never heard of trade in goods the Old World craves. This is the frontier.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "wealth_milestone",
        phase: "reward",
        title: "A Captain of Substance",
        text: "Your silver reserves have crossed a line that separates traders from merchants. Ships, warehouses, contracts — you're no longer surviving. You're building.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "trade_house",
        phase: "the_return",
        title: "The House You Built",
        text: "Brokers in three regions know your name. Warehouses hold your goods in ports you haven't visited in weeks. Contracts arrive without you asking. You didn't just trade — you built something that will outlast you.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "galleon_master",
        phase: "the_return",
        title: "Master of the Long Haul",
        text: "Your galleon cuts through waters that would sink lesser ships. Routes that terrified you as a sloop captain are now your daily bread. The sea hasn't changed. You have.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "five_regions",
        phase: "the_return",
        title: "The Known World",
        text: "You've traded in every region the maps can show. From the Mediterranean to the South Seas, from the North Atlantic to the East Indies. Few captains can say they've seen it all. You can.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "cultural_awakening",
        phase: "threshold",
        title: "More Than Ledgers",
        text: "The world is bigger than your ledger. Every port has a story older than your ship. Every good you carry means something to someone beyond its price.",
        flavor: "You begin to see the cultures behind the commerce.",
        hint: "Watch for cultural events at sea — they reveal the world's personality.",
    },
    BeatDef {
        id: "festival_trader",
        phase: "tests",
        title: "Festival Fortune",
        text: "The market swells with festival crowds. Prices soar, competition is fierce, and the locals remember who traded fairly during the celebration. Commerce and culture are the same thing here.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "sacred_cargo",
        phase: "tests",
        title: "What They Revere",
        text: "You carry what they revere. Handle it with care — this cargo is worth more than silver to the people who receive it. Your standing grows not because you traded well, but because you traded right.",
        flavor: "",
        hint: "Sacred goods earn standing bonuses in their home regions.",
    },
    BeatDef {
        id: "forbidden_trade",
        phase: "ordeal",
        title: "The Weight of Taboo",
        text: "They didn't say anything when you sold. But the silence was heavy. You've broken a cultural rule, and customs heat rises. Some profits cost more than silver.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "cultural_bridge",
        phase: "reward",
        title: "Bridge Between Worlds",
        text: "You belong everywhere and nowhere. The merchant who speaks every tongue and respects every custom is trusted by all. Three regions greet you as one of their own.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "festival_patron",
        phase: "reward",
        title: "Friend of the Festivals",
        text: "Word spreads along the trade routes: you are a friend of the festivals. Not just a buyer who arrives when prices rise, but a captain who respects the celebration. The ports remember.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "the_known_world_culture",
        phase: "the_return",
        title: "A Citizen of the Sea",
        text: "From the columned exchanges of the Mediterranean to the coral thrones of the South Seas, you've seen how every people makes meaning from trade. Commerce isn't just numbers. It's the story of how strangers become neighbors.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "proverb_collector",
        phase: "the_return",
        title: "Wisdom of the Ports",
        text: "Every port taught you something. You carry their wisdom like ballast — invisible, but it keeps you steady. The proverbs of twenty harbors live in your captain's log.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "first_winter",
        phase: "tests",
        title: "The Cold Season",
        text: "Winter closes in. The northern ports grow quiet and the sea turns grey. Experienced captains planned for this — they stocked medicines and tea when prices were low. The unprepared pay winter rates.",
        flavor: "",
        hint: "Watch the seasons. Buy goods when abundant, sell when scarce.",
    },
    BeatDef {
        id: "monsoon_survivor",
        phase: "tests",
        title: "Through the Monsoon",
        text: "The monsoon season tried to swallow your ship whole. Rain so heavy it felt solid, waves that blocked out the sky. But you kept the crew alive and the cargo dry. The East Indies respect a captain who dares the monsoon.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "harvest_trader",
        phase: "reward",
        title: "Riding the Harvest",
        text: "You timed it perfectly. When the harvest flooded the market with cheap grain and cotton, you were there to buy. When winter drove demand through the roof, you were there to sell. The calendar is a captain's secret weapon.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "four_seasons_captain",
        phase: "the_return",
        title: "Captain for All Seasons",
        text: "You've sailed through spring calms and winter gales, monsoon fury and autumn harvests. The sea has shown you every face it has. You trade with the rhythm of the world, not against it.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "first_contraband",
        phase: "threshold",
        title: "Crossing the Line",
        text: "You bought something the law says you shouldn't have. It sits in your hold like a secret — valuable, dangerous, and impossible to un-know. The legitimate world just got a little smaller.",
        flavor: "",
        hint: "Contraband can only be sold at BLACK_MARKET ports. Plan your route carefully.",
    },
    BeatDef {
        id: "underworld_contact",
        phase: "tests",
        title: "A Name in the Dark",
        text: "Word travels in the underworld. A pirate faction knows your name — not as prey, but as someone worth talking to. The line between trader and smuggler just blurred.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "pirate_deal",
        phase: "tests",
        title: "Trading with Wolves",
        text: "You traded with a pirate captain on the open sea. No port, no witnesses, no manifest. Just two captains, a price, and a handshake. The underworld does business differently.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "first_duel_win",
        phase: "tests",
        title: "Blood and Steel",
        text: "Your blade found its mark. The pirate captain yielded, and for a heartbeat the world narrowed to two people and a single truth: you earned what you carry. The crew looks at you differently now.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "nemesis_born",
        phase: "ordeal",
        title: "A Grudge on the Water",
        text: "A pirate captain remembers you. Not as a trade partner or a neutral ship — as an enemy. They'll be watching for your sails, and next time, the conversation starts with steel.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "faction_trusted",
        phase: "reward",
        title: "The Shadow's Trust",
        text: "A pirate faction trusts you completely. Their captains greet you as an ally, their ports treat you as family. You've earned what money alone can't buy: a place in the underworld's inner circle.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "duel_master",
        phase: "reward",
        title: "Blade of the Sea",
        text: "Five captains have felt your steel. Your name is spoken with respect in every pirate port and with fear on every patrol ship. The blade is part of who you are now.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "shadow_master",
        phase: "the_return",
        title: "Lord of the Grey",
        text: "Three factions count you as a friend. The underworld's politics flow through your hold as surely as the legitimate trade. You've built something no customs inspector can confiscate: a network that spans every shadow port in the Known World.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "faction_spillover",
        phase: "tests",
        title: "The Price of Friends",
        text: "You helped one faction, and another noticed. In the underworld, every friendship casts a shadow. The enemies of your friends are now watching you with different eyes.",
        flavor: "",
        hint: "Standing with one faction affects your reputation with their rivals and enemies.",
    },
    BeatDef {
        id: "vendetta_declared",
        phase: "ordeal",
        title: "Blood in the Ledger",
        text: "A faction has declared vendetta. Your alliance with their enemy has made you a target — not just a stranger, but a marked captain. Their ships will hunt you in their waters. Choose your routes carefully.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "political_survivor",
        phase: "reward",
        title: "Walking the Wire",
        text: "You've navigated the underworld's politics without being destroyed by them. Trade partners on one side, enemies on the other, and you in the middle — still sailing, still trading, still alive. That's an achievement few can claim.",
        flavor: "",
        hint: "",
    },
    BeatDef {
        id: "faction_diplomat",
        phase: "the_return",
        title: "The Pirate's Diplomat",
        text: "Factions that hate each other both trust you. You've done what no navy, no governor, no merchant guild has managed: earned standing on both sides of a pirate war. The sea's politics flow through you.",
        flavor: "",
        hint: "",
    },
];

#[allow(clippy::too_many_arguments)]
pub fn evaluate_narrative(
    state: &mut NarrativeState,
    world: &World,
    board: &ContractBoard,
    infra: &InfrastructureRecord,
    books: &HouseBooks,
    receipts: &[TradeReceipt],
    current_port_id: Option<&str>,
    events: &[VoyageEvent],
) -> Vec<NarrativeBeat> {
    let port = current_port_id.and_then(|id| world.port(id));
    let region = port.map(|port| port.region.as_str()).unwrap_or("");
    let mut fired = Firer {
        state,
        newly: Vec::new(),
        day: world.day,
        current_port_id: current_port_id.unwrap_or(""),
    };

    if books.total_sells > 0 {
        fired.fire("first_trade", "", region);
    }
    if world.voyage.status != VoyageStatus::InPort {
        fired.fire("first_voyage", "", "");
    }
    if books.net_profit > 0 {
        fired.fire("first_profit", "", "");
    }

    let mut regions_visited = Vec::new();
    for receipt in receipts {
        if let Some(port) = world.port(&receipt.port_id) {
            if !regions_visited.iter().any(|seen| seen == &port.region) {
                regions_visited.push(port.region.clone());
            }
        }
    }
    if regions_visited.len() >= 2 {
        fired.fire("new_region", "", region);
    }
    if board.completed.iter().any(|outcome| {
        outcome.outcome_type == "completed" || outcome.outcome_type == "completed_bonus"
    }) {
        fired.fire("first_contract", "", "");
    }
    if let Some(ship) = &world.captain.ship {
        if let Some(template) = content::content().ship(&ship.template_id) {
            if template.ship_class != "sloop" {
                fired.fire("ship_upgrade", "", "");
            }
        }
    }

    for event in events {
        match event.event_type {
            EventType::Storm => fired.fire("survived_storm", "", ""),
            EventType::Pirates => fired.fire("survived_pirates", "", ""),
            EventType::Inspection => fired.fire("first_inspection", "", ""),
            _ => {}
        }
    }
    if receipts.len() >= 20 {
        fired.fire("rival_encounter", "", "");
    }
    let mut ports_visited = Vec::new();
    for receipt in receipts {
        if !ports_visited.iter().any(|id| id == &receipt.port_id) {
            ports_visited.push(receipt.port_id.clone());
        }
    }
    if ports_visited.len() >= 3 {
        fired.fire("mentor_wisdom", "", "");
    }

    for incident in &world.captain.standing.incidents {
        if incident.incident_type.to_lowercase().contains("seizure")
            || incident.description.to_lowercase().contains("seize")
        {
            fired.fire("cargo_seized", "", "");
            break;
        }
    }
    if world.captain.silver < 50 && world.day > 10 {
        fired.fire("near_bankruptcy", "", "");
    }
    if board
        .completed
        .iter()
        .any(|outcome| outcome.outcome_type == "expired" || outcome.outcome_type == "abandoned")
    {
        fired.fire("contract_failed", "", "");
    }

    let successful = board
        .completed
        .iter()
        .any(|outcome| outcome.outcome_type.contains("completed") && outcome.silver_delta >= 500);
    if successful {
        fired.fire("first_big_contract", "", "");
    }
    if region == "East Indies" {
        fired.fire("east_indies_arrival", "", "East Indies");
    }
    if region == "South Seas" {
        fired.fire("south_seas_discovery", "", "South Seas");
    }
    if world.captain.silver >= 2000 {
        fired.fire("wealth_milestone", "", "");
    }

    let mut broker_regions = Vec::new();
    for broker in &infra.brokers {
        if broker.active && !broker_regions.iter().any(|seen| seen == &broker.region) {
            broker_regions.push(broker.region.clone());
        }
    }
    if broker_regions.len() >= 3 {
        fired.fire("trade_house", "", "");
    }
    if let Some(ship) = &world.captain.ship {
        if let Some(template) = content::content().ship(&ship.template_id) {
            if template.ship_class == "galleon" || template.ship_class == "man_of_war" {
                fired.fire("galleon_master", "", "");
            }
        }
    }
    let standing_regions = world
        .captain
        .standing
        .regional
        .iter()
        .filter(|value| **value >= 5)
        .count();
    if standing_regions >= 5 {
        fired.fire("five_regions", "", "");
    }

    if world.culture.cultural_encounters >= 1 {
        fired.fire("cultural_awakening", "", "");
    }
    if current_port_id.is_some() && books.total_sells > 0 {
        let here = current_port_id.unwrap_or("");
        if world
            .culture
            .active_festivals
            .iter()
            .any(|fest| fest.port_id == here)
        {
            fired.fire("festival_trader", "", "");
        }
    }
    let high_standing = world
        .captain
        .standing
        .regional
        .iter()
        .filter(|value| **value >= 15)
        .count();
    if high_standing >= 3 {
        fired.fire("cultural_bridge", "", "");
    }
    if world.culture.festivals_visited >= 3 {
        fired.fire("festival_patron", "", "");
    }
    if world.culture.regions_entered.len() >= 5 && world.culture.cultural_encounters >= 5 {
        fired.fire("the_known_world_culture", "", "");
    }
    if world.culture.port_visits.len() >= 15 {
        fired.fire("proverb_collector", "", "");
    }

    let season = season_name(world.day);
    if season == "winter" && world.day >= 271 {
        fired.fire("first_winter", "", "");
    }
    if season == "summer" && region == "East Indies" {
        fired.fire("monsoon_survivor", "", "");
    }
    if season == "autumn" && books.net_profit >= 500 {
        fired.fire("harvest_trader", "", "");
    }
    if world.day > 360 {
        fired.fire("four_seasons_captain", "", "");
    }

    let contraband = ["opium", "black_powder", "stolen_cargo"];
    if world
        .captain
        .cargo
        .iter()
        .any(|item| contraband.contains(&item.good_id.as_str()))
    {
        fired.fire("first_contraband", "", "");
    }
    if world
        .captain
        .standing
        .underworld
        .iter()
        .any(|(_, value)| *value >= 10)
    {
        fired.fire("underworld_contact", "", "");
    }
    if world
        .captain
        .encounters
        .iter()
        .any(|encounter| encounter.outcome == "trade")
    {
        fired.fire("pirate_deal", "", "");
    }
    if world.captain.duels_won >= 1 {
        fired.fire("first_duel_win", "", "");
    }
    if world.nemesis_id.is_some() {
        fired.fire("nemesis_born", "", "");
    }
    if world
        .captain
        .standing
        .underworld
        .iter()
        .any(|(_, value)| *value >= 50)
    {
        fired.fire("faction_trusted", "", "");
    }
    if world.captain.duels_won >= 5 {
        fired.fire("duel_master", "", "");
    }
    let trusted_factions = world
        .captain
        .standing
        .underworld
        .iter()
        .filter(|(_, value)| *value >= 25)
        .count();
    if trusted_factions >= 3 {
        fired.fire("shadow_master", "", "");
    }
    if world
        .captain
        .standing
        .underworld
        .iter()
        .any(|(_, value)| *value >= 15)
    {
        fired.fire("faction_spillover", "", "");
    }
    for (faction_id, _) in &world.captain.standing.underworld {
        if !vendetta(&world.captain.standing.underworld, faction_id).is_empty() {
            fired.fire("vendetta_declared", "", "");
            break;
        }
    }
    let above_10: Vec<&str> = world
        .captain
        .standing
        .underworld
        .iter()
        .filter(|(_, value)| *value >= 10)
        .map(|(id, _)| id.as_str())
        .collect();
    'pairs: for (index, faction) in above_10.iter().enumerate() {
        let enemies = content::content().faction_enemies(faction);
        for other in &above_10[index + 1..] {
            if enemies.iter().any(|enemy| enemy == other) {
                fired.fire("political_survivor", "", "");
                break 'pairs;
            }
        }
    }
    let above_25: Vec<&str> = world
        .captain
        .standing
        .underworld
        .iter()
        .filter(|(_, value)| *value >= 25)
        .map(|(id, _)| id.as_str())
        .collect();
    'diplomats: for (index, faction) in above_25.iter().enumerate() {
        let enemies = content::content().faction_enemies(faction);
        for other in &above_25[index + 1..] {
            if enemies.iter().any(|enemy| enemy == other) {
                fired.fire("faction_diplomat", "", "");
                break 'diplomats;
            }
        }
    }
    fired.newly
}

struct Firer<'a> {
    state: &'a mut NarrativeState,
    newly: Vec<NarrativeBeat>,
    day: i64,
    current_port_id: &'a str,
}

impl Firer<'_> {
    fn fire(&mut self, beat_id: &str, port_id: &str, beat_region: &str) {
        if self.state.fired.iter().any(|id| id == beat_id) {
            return;
        }
        let Some(beat) = BEATS.iter().find(|beat| beat.id == beat_id) else {
            return;
        };
        self.newly.push(owned(beat));
        self.state.fired.push(beat_id.to_string());
        let journal_port = if port_id.is_empty() {
            self.current_port_id.to_string()
        } else {
            port_id.to_string()
        };
        self.state.journal.push(crate::model::JournalEntry {
            beat_id: beat_id.to_string(),
            day: self.day,
            port_id: journal_port,
            region: beat_region.to_string(),
        });
    }
}

fn owned(beat: &BeatDef) -> NarrativeBeat {
    NarrativeBeat {
        id: beat.id.to_string(),
        phase: beat.phase.to_string(),
        title: beat.title.to_string(),
        text: beat.text.to_string(),
        flavor: beat.flavor.to_string(),
        hint: beat.hint.to_string(),
    }
}

/// `check_vendetta`: standing 25+ with `source`, and a hostile enemy below 10.
fn vendetta(underworld: &[(String, i64)], source: &str) -> Vec<String> {
    let source_standing = underworld
        .iter()
        .find(|(id, _)| id == source)
        .map(|(_, value)| *value)
        .unwrap_or(0);
    if source_standing < 25 {
        return Vec::new();
    }
    content::content()
        .faction_enemies(source)
        .into_iter()
        .filter(|enemy| {
            underworld
                .iter()
                .find(|(id, _)| id == enemy)
                .map(|(_, value)| *value)
                .unwrap_or(0)
                < 10
        })
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::new_game;

    #[test]
    fn first_sale_fires_the_call_and_only_once() {
        let world = new_game("Ada", "merchant", 1, Some("porto_novo")).unwrap();
        let mut state = NarrativeState::default();
        let books = HouseBooks {
            total_sells: 12,
            net_profit: 4,
            ..HouseBooks::default()
        };
        let board = ContractBoard::default();
        let infra = InfrastructureRecord::default();
        let beats = evaluate_narrative(
            &mut state,
            &world,
            &board,
            &infra,
            &books,
            &[],
            Some("porto_novo"),
            &[],
        );
        let ids: Vec<_> = beats.iter().map(|beat| beat.id.as_str()).collect();
        assert_eq!(ids, ["first_trade", "first_profit"]);
        assert_eq!(state.journal[0].port_id, "porto_novo");
        assert_eq!(state.journal[0].region, "Mediterranean");
        let again = evaluate_narrative(
            &mut state,
            &world,
            &board,
            &infra,
            &books,
            &[],
            Some("porto_novo"),
            &[],
        );
        assert!(again.is_empty());
    }

    #[test]
    fn catalog_matches_python_count() {
        assert_eq!(BEATS.len(), 45);
        assert!(BEATS.iter().any(|beat| beat.id == "faction_diplomat"));
    }
}
