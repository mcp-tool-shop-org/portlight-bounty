//! Docked counting-house. Warehouse, broker, license, credit, and insurance.
//!
//! Session already owns the verbs. This screen does not call `dry_dock`, and
//! it does not offer claim or cancel-lease buttons. Closures are unpaid
//! upkeep inside `Session::advance`.

use std::collections::BTreeMap;

use godot::classes::control::{MouseFilter, SizeFlags};
use godot::classes::text_server::AutowrapMode;
use godot::classes::{
    Button, HBoxContainer, Label, PanelContainer, ScrollContainer, StyleBoxFlat, TextureRect,
    VBoxContainer,
};
use godot::prelude::*;
use portlight_sim::content::{self, BrokerOfficeDef};
use portlight_sim::economy::item_weight;
use portlight_sim::infrastructure::{self, check_license_eligibility};
use portlight_sim::model::{InfrastructureRecord, VoyageStatus};
use portlight_sim::Session;

use crate::encounter_screen;
use crate::logic::humanize_id;

const INK: Color = Color::from_rgb(0.08, 0.11, 0.16);
const CREAM: Color = Color::from_rgb(0.94, 0.91, 0.84);
const GOLD: Color = Color::from_rgb(0.96, 0.84, 0.45);
const MUTED: Color = Color::from_rgb(0.7, 0.74, 0.78);

pub(crate) const WAREHOUSE_LAPSE: &str =
    "Unpaid upkeep of 3 days closes the lease and can seize the goods. There is no cancel.";
pub(crate) const OFFICE_LAPSE: &str =
    "Unpaid upkeep of 5 days closes the office. There is no cancel.";
pub(crate) const INSURANCE_NOTE: &str = "Payouts arrive on the next day. There is no claim button.";

pub(crate) const ANCHOR_WAREHOUSE: &str = "HarbourWarehouse";
pub(crate) const ANCHOR_BROKER: &str = "HarbourBroker";
pub(crate) const ANCHOR_FINANCE: &str = "HarbourCredit";
pub(crate) const ANCHOR_INSURANCE: &str = "HarbourInsurance";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HarbourIntent {
    LeaseWarehouse(String),
    OpenBroker {
        region: String,
        tier: String,
    },
    BuyLicense(String),
    OpenCredit(String),
    Draw {
        tier: String,
        amount: i64,
    },
    Emergency(i64),
    BuyInsurance {
        policy_id: String,
        target_id: String,
        origin: String,
        destination: String,
    },
}

/// `buy_infrastructure` kind for this intent. Deposit, credit, and insurance
/// are other Session calls. `dry_dock` is not one of them.
pub(crate) fn infrastructure_kind(intent: &HarbourIntent) -> Option<&'static str> {
    match intent {
        HarbourIntent::LeaseWarehouse(_) => Some("warehouse"),
        HarbourIntent::OpenBroker { .. } => Some("broker"),
        HarbourIntent::BuyLicense(_) => Some("license"),
        _ => None,
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ActionRow {
    pub title: String,
    pub detail: String,
    pub button: String,
    pub intent: Option<HarbourIntent>,
    pub block: String,
}

#[derive(Clone, Debug)]
pub(crate) struct GoodPick {
    pub good_id: String,
    pub label: String,
}

#[derive(Clone, Debug)]
pub(crate) struct HarbourModel {
    pub port_name: String,
    pub ship_template: String,
    pub status: String,
    pub warehouse_lines: Vec<String>,
    pub deposit_goods: Vec<GoodPick>,
    pub withdraw_goods: Vec<GoodPick>,
    pub warehouse_offers: Vec<ActionRow>,
    pub broker_lines: Vec<String>,
    pub broker_offers: Vec<ActionRow>,
    pub license_lines: Vec<String>,
    pub license_offers: Vec<ActionRow>,
    pub credit_lines: Vec<String>,
    pub credit_offers: Vec<ActionRow>,
    pub draw_tier: String,
    pub draw_block: String,
    pub repay_all: i64,
    pub repay_block: String,
    pub policy_lines: Vec<String>,
    pub claim_lines: Vec<String>,
    pub insurance_offers: Vec<ActionRow>,
}

#[derive(Clone)]
pub(crate) struct HarbourNodes {
    pub root: Gd<PanelContainer>,
    pub title: Gd<Label>,
    pub notice: Gd<Label>,
    pub confirm_row: Gd<HBoxContainer>,
    pub scroll: Gd<ScrollContainer>,
    pub body: Gd<VBoxContainer>,
    pub close_host: Gd<HBoxContainer>,
    pub plate: Gd<TextureRect>,
    pub plate_panel: Gd<PanelContainer>,
    pub plate_caption: Gd<Label>,
    pub placeholder: Gd<PanelContainer>,
}

pub(crate) fn build_harbour_screen() -> HarbourNodes {
    let mut root = PanelContainer::new_alloc();
    root.set_name("HarbourScreen");
    root.set_mouse_filter(MouseFilter::STOP);
    root.set_visible(false);
    let mut style = StyleBoxFlat::new_gd();
    style.set_bg_color(INK);
    style.set_content_margin_all(28.0);
    root.add_theme_stylebox_override("panel", &style);

    let mut row = HBoxContainer::new_alloc();
    row.set_h_size_flags(SizeFlags::EXPAND_FILL);
    row.set_v_size_flags(SizeFlags::EXPAND_FILL);
    row.add_theme_constant_override("separation", 20);
    root.add_child(&row);

    let side = encounter_screen::side_column();
    let plate = side.plate.clone();
    let plate_panel = side.panel.clone();
    let plate_caption = side.caption.clone();
    let placeholder = side.placeholder.clone();
    row.add_child(&side.column);

    let mut column = VBoxContainer::new_alloc();
    column.set_h_size_flags(SizeFlags::EXPAND_FILL);
    column.set_v_size_flags(SizeFlags::EXPAND_FILL);
    column.add_theme_constant_override("separation", 10);
    row.add_child(&column);

    column.add_child(&text_label("Harbour", 14, MUTED));
    let title = text_label("", 28, GOLD);
    column.add_child(&title);
    let mut notice = text_label("", 16, CREAM);
    notice.set_autowrap_mode(AutowrapMode::WORD_SMART);
    column.add_child(&notice);

    let mut confirm_row = HBoxContainer::new_alloc();
    confirm_row.set_name("HarbourConfirm");
    confirm_row.set_visible(false);
    confirm_row.add_theme_constant_override("separation", 8);
    column.add_child(&confirm_row);

    let mut scroll = ScrollContainer::new_alloc();
    scroll.set_h_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_v_size_flags(SizeFlags::EXPAND_FILL);
    scroll.set_horizontal_scroll_mode(godot::classes::scroll_container::ScrollMode::DISABLED);
    let mut body = VBoxContainer::new_alloc();
    body.set_name("HarbourBody");
    body.set_h_size_flags(SizeFlags::EXPAND_FILL);
    body.add_theme_constant_override("separation", 8);
    scroll.add_child(&body);
    column.add_child(&scroll);

    let mut close_host = HBoxContainer::new_alloc();
    close_host.set_name("HarbourClose");
    close_host.add_theme_constant_override("separation", 8);
    column.add_child(&close_host);

    HarbourNodes {
        root,
        title,
        notice,
        confirm_row,
        scroll,
        body,
        close_host,
        plate,
        plate_panel,
        plate_caption,
        placeholder,
    }
}

/// Catalog and infra for the docked port. `None` at sea.
pub(crate) fn harbour_model(session: &Session) -> Option<HarbourModel> {
    let world = session.world();
    if world.voyage.status != VoyageStatus::InPort {
        return None;
    }
    let port_id = world.voyage.destination_id.as_str();
    let port = world.port(port_id)?;
    let catalog = content::content();
    let infra = session.infrastructure();
    let silver = world.captain.silver;
    let standing = &world.captain.standing;
    let region = port.region.clone();
    let heat = standing.heat_of(&region);

    let lease = infra
        .warehouses
        .iter()
        .find(|lease| lease.port_id == port_id && lease.active);
    let mut warehouse_lines = Vec::new();
    if let Some(lease) = lease {
        let name = warehouse_name(&lease.tier);
        let used: f64 = lease
            .inventory
            .iter()
            .map(|lot| item_weight(&lot.good_id, lot.quantity))
            .sum();
        warehouse_lines.push(format!(
            "{name}. {used}/{capacity}. Active {active}. Upkeep {upkeep}/day.",
            used = format_units(used),
            capacity = lease.capacity,
            active = yes_no(lease.active),
            upkeep = lease.upkeep_per_day
        ));
        let totals = totals_by_good(
            lease
                .inventory
                .iter()
                .map(|lot| (lot.good_id.as_str(), lot.quantity)),
        );
        if totals.is_empty() {
            warehouse_lines.push("Warehouse is empty.".to_string());
        } else {
            for (good_id, qty) in &totals {
                warehouse_lines.push(format!("{} x {qty}", good_name(good_id)));
            }
        }
    } else {
        warehouse_lines.push("No warehouse leased here.".to_string());
    }
    warehouse_lines.push(WAREHOUSE_LAPSE.to_string());

    let deposit_goods = good_picks(
        world
            .captain
            .cargo
            .iter()
            .map(|item| (item.good_id.as_str(), item.quantity)),
    );
    let withdraw_goods = lease
        .map(|lease| {
            good_picks(
                lease
                    .inventory
                    .iter()
                    .map(|lot| (lot.good_id.as_str(), lot.quantity)),
            )
        })
        .unwrap_or_default();

    let offered = catalog.port_warehouse_tiers(port_id);
    let mut warehouse_offers = Vec::new();
    if offered.is_empty() {
        warehouse_lines.push("No warehouse tiers offered at this port.".to_string());
    }
    for tier in offered {
        let Some(spec) = catalog.warehouse_tier(tier) else {
            continue;
        };
        let name = ascii_copy(&spec.name);
        let mut block = String::new();
        if lease.is_some_and(|lease| lease.tier == spec.tier) {
            block = format!("Already have a {name} at this port");
        } else if spec.lease_cost > silver {
            block = format!(
                "Need {} silver to lease {name}, have {silver}",
                spec.lease_cost
            );
        }
        warehouse_offers.push(ActionRow {
            title: name,
            detail: format!(
                "{} silver. Capacity {}. Upkeep {}/day. {}",
                spec.lease_cost,
                spec.capacity,
                spec.upkeep_per_day,
                ascii_copy(&spec.description)
            ),
            button: "Lease".to_string(),
            intent: block
                .is_empty()
                .then(|| HarbourIntent::LeaseWarehouse(spec.tier.clone())),
            block,
        });
    }

    let mut broker_lines = Vec::new();
    let active_brokers: Vec<_> = infra
        .brokers
        .iter()
        .filter(|broker| broker.active && broker.tier != "none")
        .collect();
    if active_brokers.is_empty() {
        broker_lines.push("No broker office open.".to_string());
    }
    for office in active_brokers {
        let spec = catalog.broker(&office.region, &office.tier);
        let name = broker_name(&office.region, &office.tier);
        let upkeep = spec.map(|spec| spec.upkeep_per_day).unwrap_or(0);
        broker_lines.push(format!(
            "{name}. Active {active}. Upkeep {upkeep}/day.",
            active = yes_no(office.active)
        ));
    }
    broker_lines.push(OFFICE_LAPSE.to_string());

    let mut broker_offers = Vec::new();
    for spec in catalog
        .infrastructure
        .brokers
        .iter()
        .filter(|spec| spec.region == region)
    {
        broker_offers.push(broker_row(infra, silver, spec));
    }

    let mut license_lines = Vec::new();
    let held: Vec<_> = infra
        .licenses
        .iter()
        .filter(|license| license.active)
        .collect();
    if held.is_empty() {
        license_lines.push("No license held.".to_string());
    }
    for owned in held {
        let spec = catalog.license(&owned.license_id);
        let name = license_name(&owned.license_id);
        let upkeep = spec.map(|spec| spec.upkeep_per_day).unwrap_or(0);
        license_lines.push(format!("{name}. Upkeep {upkeep}/day."));
    }
    license_lines.push(OFFICE_LAPSE.to_string());

    let mut license_offers = Vec::new();
    for spec in &catalog.infrastructure.licenses {
        let name = ascii_copy(&spec.name);
        let mut block = check_license_eligibility(infra, spec, standing)
            .err()
            .map(|err| trust_block(&ascii_copy(&err.to_string()), &spec.required_trust_tier))
            .unwrap_or_default();
        if block.is_empty() && spec.purchase_cost > silver {
            block = format!("Need {} silver, have {silver}", spec.purchase_cost);
        }
        let scope = spec
            .region_scope
            .as_deref()
            .map(ascii_copy)
            .unwrap_or_else(|| "every region".to_string());
        license_offers.push(ActionRow {
            title: name,
            detail: format!(
                "{} silver. Upkeep {}/day. {}Scope {scope}. {}",
                spec.purchase_cost,
                spec.upkeep_per_day,
                requirement_terms(&spec.required_trust_tier, spec.required_standing),
                ascii_copy(&spec.description)
            ),
            button: "Buy".to_string(),
            intent: block
                .is_empty()
                .then(|| HarbourIntent::BuyLicense(spec.id.clone())),
            block,
        });
    }

    let credit = infra.credit.as_ref().filter(|credit| {
        credit.active
            || credit.outstanding != 0
            || credit.interest_accrued != 0
            || credit.defaults != 0
    });
    let mut credit_lines = Vec::new();
    if let Some(credit) = credit {
        let name = credit_tier_name(&credit.tier);
        credit_lines.push(format!(
            "{name}. Limit {}. Outstanding {}. Interest {}. Next due day {}.",
            credit.credit_limit, credit.outstanding, credit.interest_accrued, credit.next_due_day
        ));
        if credit.defaults > 0 {
            credit_lines.push(format!("Defaults {}.", credit.defaults));
        }
    } else {
        credit_lines.push("No credit line.".to_string());
    }
    credit_lines.push(
        "Emergency loans are capped at 200 silver and add a deferred fee of about 15%.".to_string(),
    );

    let active_tier = infra
        .credit
        .as_ref()
        .filter(|credit| credit.active)
        .map(|credit| credit.tier.clone());
    let active_rank = active_tier.as_deref().map(credit_rank).unwrap_or(0);
    let mut credit_offers = Vec::new();
    for spec in &catalog.infrastructure.credit_tiers {
        let name = ascii_copy(&spec.name);
        let mut owned = infra.clone();
        let mut block = infrastructure::check_credit_eligibility(&mut owned, spec, standing)
            .err()
            .map(|err| trust_block(&ascii_copy(&err.to_string()), &spec.required_trust_tier))
            .unwrap_or_default();
        if block.is_empty() && active_tier.is_some() && active_rank >= credit_rank(&spec.tier) {
            let current = active_tier
                .as_deref()
                .map_or_else(|| "none".to_string(), credit_tier_name);
            block = format!("Already have {current} or better");
        }
        credit_offers.push(ActionRow {
            title: name.clone(),
            detail: format!(
                "Limit {}. Interest {:.0}% each {} days. {}{}",
                spec.credit_limit,
                spec.interest_rate * 100.0,
                spec.interest_period,
                requirement_terms(&spec.required_trust_tier, spec.required_standing),
                ascii_copy(&spec.description)
            ),
            button: "Open".to_string(),
            intent: block
                .is_empty()
                .then(|| HarbourIntent::OpenCredit(spec.tier.clone())),
            block,
        });
    }

    let draw_tier = active_tier
        .clone()
        .unwrap_or_else(|| "merchant_line".to_string());
    let draw_block = if active_tier.is_some() {
        String::new()
    } else if let Some(spec) = catalog.credit_tier(&draw_tier) {
        let mut owned = infra.clone();
        infrastructure::check_credit_eligibility(&mut owned, spec, standing)
            .err()
            .map(|err| trust_block(&ascii_copy(&err.to_string()), &spec.required_trust_tier))
            .unwrap_or_default()
    } else {
        format!("Unknown credit tier: {draw_tier}")
    };

    let repay_all = infra
        .credit
        .as_ref()
        .filter(|credit| credit.active)
        .map(|credit| credit.outstanding + credit.interest_accrued)
        .unwrap_or(0);
    let repay_block = if infra.credit.as_ref().is_none_or(|credit| !credit.active) {
        "No credit line".to_string()
    } else if repay_all <= 0 {
        "No outstanding debt".to_string()
    } else {
        String::new()
    };

    let mut policy_lines = Vec::new();
    let active_policies: Vec<_> = infra
        .policies
        .iter()
        .filter(|policy| policy.active)
        .collect();
    if active_policies.is_empty() {
        policy_lines.push("No active policy.".to_string());
    }
    for policy in &active_policies {
        let name = policy_name(&policy.spec_id);
        let mut line = format!(
            "{name}. Premium {}. Claims {}.",
            policy.premium_paid, policy.claims_made
        );
        if !policy.target_id.is_empty() {
            line.push_str(&format!(
                " Target {}.",
                contract_target_name(session, &policy.target_id)
            ));
        }
        if !policy.voyage_origin.is_empty() || !policy.voyage_destination.is_empty() {
            line.push_str(&format!(
                " Voyage {} to {}.",
                port_name(session, &policy.voyage_origin),
                port_name(session, &policy.voyage_destination)
            ));
        }
        policy_lines.push(line);
    }
    policy_lines.push(INSURANCE_NOTE.to_string());

    let claim_lines = infra
        .claims
        .iter()
        .map(|claim| {
            claim_line(
                claim.day,
                &claim.incident_type,
                claim.denied,
                &claim.denial_reason,
                claim.payout,
            )
        })
        .collect();

    let contracts: Vec<_> = session.board().active.iter().collect();
    let mut insurance_offers = Vec::new();
    for spec in &catalog.infrastructure.policies {
        let name = ascii_copy(&spec.name);
        let premium = adjusted_premium(spec.premium, heat, spec.heat_premium_mult);
        let family = family_label(&spec.family);
        if spec.scope == "named_contract" {
            if contracts.is_empty() {
                insurance_offers.push(ActionRow {
                    title: format!("{family}: {name}"),
                    detail: format!("{premium} silver. {}", ascii_copy(&spec.description)),
                    button: "Buy".to_string(),
                    intent: None,
                    block: "No contract to guarantee.".to_string(),
                });
                continue;
            }
            for contract in &contracts {
                let mut block = String::new();
                if spec.heat_max.is_some_and(|max| heat > max) {
                    block = format!(
                        "Heat too high ({heat}) for {name} - max {}",
                        spec.heat_max.unwrap_or(0)
                    );
                } else if premium > silver {
                    block = format!("Need {premium} silver for {name}, have {silver}");
                } else if infra.policies.iter().any(|policy| {
                    policy.active
                        && policy.spec_id == spec.id
                        && policy.target_id == contract.offer_id
                }) {
                    block = format!("Already have active {name}");
                }
                let title_name = ascii_copy(&contract.title);
                insurance_offers.push(ActionRow {
                    title: format!("{family}: {name}"),
                    detail: format!(
                        "{premium} silver. Contract {title_name}. {}",
                        ascii_copy(&spec.description)
                    ),
                    button: "Buy".to_string(),
                    intent: block.is_empty().then(|| HarbourIntent::BuyInsurance {
                        policy_id: spec.id.clone(),
                        target_id: contract.offer_id.clone(),
                        origin: String::new(),
                        destination: String::new(),
                    }),
                    block,
                });
            }
            continue;
        }
        let mut block = String::new();
        if spec.heat_max.is_some_and(|max| heat > max) {
            block = format!(
                "Heat too high ({heat}) for {name} - max {}",
                spec.heat_max.unwrap_or(0)
            );
        } else if premium > silver {
            block = format!("Need {premium} silver for {name}, have {silver}");
        } else if infra
            .policies
            .iter()
            .any(|policy| policy.active && policy.spec_id == spec.id && policy.target_id.is_empty())
        {
            block = format!("Already have active {name}");
        }
        insurance_offers.push(ActionRow {
            title: format!("{family}: {name}"),
            detail: format!("{premium} silver. {}", ascii_copy(&spec.description)),
            button: "Buy".to_string(),
            intent: block.is_empty().then(|| HarbourIntent::BuyInsurance {
                policy_id: spec.id.clone(),
                target_id: String::new(),
                origin: String::new(),
                destination: String::new(),
            }),
            block,
        });
    }

    let warehouse_bit = lease
        .map(|lease| {
            let name = catalog
                .warehouse_tier(&lease.tier)
                .map(|spec| ascii_copy(&spec.name))
                .unwrap_or_else(|| "Warehouse".to_string());
            format!("{name} active")
        })
        .unwrap_or_else(|| "no warehouse".to_string());
    let credit_bit = active_tier
        .as_deref()
        .map(|tier| {
            let owed = infra
                .credit
                .as_ref()
                .map(|credit| credit.outstanding)
                .unwrap_or(0);
            credit_status(tier, owed)
        })
        .unwrap_or_else(|| "no credit line".to_string());
    let ship_template = world
        .captain
        .ship
        .as_ref()
        .map(|ship| ship.template_id.clone())
        .unwrap_or_default();

    Some(HarbourModel {
        port_name: ascii_copy(&port.name),
        ship_template,
        status: format!("Silver {silver}. {warehouse_bit}. {credit_bit}."),
        warehouse_lines,
        deposit_goods,
        withdraw_goods,
        warehouse_offers,
        broker_lines,
        broker_offers,
        license_lines,
        license_offers,
        credit_lines,
        credit_offers,
        draw_tier,
        draw_block,
        repay_all,
        repay_block,
        policy_lines,
        claim_lines,
        insurance_offers,
    })
}

pub(crate) fn confirm_prompt(session: &Session, intent: &HarbourIntent) -> String {
    let catalog = content::content();
    match intent {
        HarbourIntent::LeaseWarehouse(tier) => {
            let spec = catalog.warehouse_tier(tier);
            let name = warehouse_name(tier);
            let cost = spec.map(|spec| spec.lease_cost).unwrap_or(0);
            let upkeep = spec.map(|spec| spec.upkeep_per_day).unwrap_or(0);
            format!(
                "Lease {name} for {cost} silver? Upkeep {upkeep}/day. The lease closes only if upkeep fails."
            )
        }
        HarbourIntent::OpenBroker { region, tier } => {
            let spec = catalog.broker(region, tier);
            let name = broker_name(region, tier);
            let cost = spec.map(|spec| spec.purchase_cost).unwrap_or(0);
            let upkeep = spec.map(|spec| spec.upkeep_per_day).unwrap_or(0);
            format!(
                "Open {name} for {cost} silver? Upkeep {upkeep}/day. The office closes only if upkeep fails."
            )
        }
        HarbourIntent::BuyLicense(id) => {
            let spec = catalog.license(id);
            let name = license_name(id);
            let cost = spec.map(|spec| spec.purchase_cost).unwrap_or(0);
            let upkeep = spec.map(|spec| spec.upkeep_per_day).unwrap_or(0);
            format!(
                "Buy {name} for {cost} silver? Upkeep {upkeep}/day. The license lapses only if upkeep fails."
            )
        }
        HarbourIntent::OpenCredit(tier) => {
            let spec = catalog.credit_tier(tier);
            let name = credit_tier_name(tier);
            let limit = spec.map(|spec| spec.credit_limit).unwrap_or(0);
            format!("Open {name}? Limit {limit}. No silver is drawn yet.")
        }
        HarbourIntent::Draw { tier, amount } => {
            let name = credit_tier_name(tier);
            format!("Draw {amount} silver on {name}?")
        }
        HarbourIntent::Emergency(amount) => format!(
            "Emergency loan of {amount} silver? A deferred fee of about 15% is added. The cap is 200."
        ),
        HarbourIntent::BuyInsurance {
            policy_id,
            target_id,
            ..
        } => {
            let spec = catalog.policy(policy_id);
            let name = policy_name(policy_id);
            let region = insurance_region(session);
            let heat = session.world().captain.standing.heat_of(&region);
            let premium = spec
                .map(|spec| adjusted_premium(spec.premium, heat, spec.heat_premium_mult))
                .unwrap_or(0);
            if target_id.is_empty() {
                format!("Buy {name} for {premium} silver?")
            } else {
                // The desk only offers guarantees for active contracts, whose
                // title reads after "contract". Any other target (an offer, a
                // settled contract) already names itself, so the word is dropped.
                let target = contract_target_name(session, target_id);
                let active = session
                    .board()
                    .active
                    .iter()
                    .any(|contract| contract.offer_id == *target_id);
                if active {
                    format!("Buy {name} for contract {target} at {premium} silver?")
                } else {
                    format!("Buy {name} for {target} at {premium} silver?")
                }
            }
        }
    }
}

/// `purchase_policy`: `int(premium * (1 + heat * heat_premium_mult))`.
fn adjusted_premium(premium: i64, heat: i64, heat_premium_mult: f64) -> i64 {
    let surcharge = (heat.max(0) as f64) * heat_premium_mult;
    portlight_sim::util::py_trunc(premium as f64 * (1.0 + surcharge))
}

fn insurance_region(session: &Session) -> String {
    let world = session.world();
    if world.voyage.status == VoyageStatus::AtSea {
        return world
            .port(&world.voyage.destination_id)
            .map(|port| port.region.clone())
            .unwrap_or_else(|| "Mediterranean".to_string());
    }
    let port_id = world.voyage.destination_id.as_str();
    world
        .port(port_id)
        .map(|port| port.region.clone())
        .unwrap_or_else(|| "Mediterranean".to_string())
}

fn broker_row(infra: &InfrastructureRecord, silver: i64, spec: &BrokerOfficeDef) -> ActionRow {
    let name = ascii_copy(&spec.name);
    let current = infra
        .brokers
        .iter()
        .find(|office| office.region == spec.region && office.active && office.tier != "none");
    let mut block = String::new();
    if current.is_some_and(|office| office.tier == spec.tier) {
        block = format!("Already have a {name} in {}", ascii_copy(&spec.region));
    } else if current.is_some_and(|office| office.tier == "established" && spec.tier == "local") {
        block = "Cannot downgrade a broker office".to_string();
    } else if spec.purchase_cost > silver {
        block = format!(
            "Need {} silver to open {name}, have {silver}",
            spec.purchase_cost
        );
    }
    ActionRow {
        title: name,
        detail: format!(
            "{} silver. Upkeep {}/day. {}",
            spec.purchase_cost,
            spec.upkeep_per_day,
            broker_bonus(spec)
        ),
        button: "Open".to_string(),
        intent: block.is_empty().then(|| HarbourIntent::OpenBroker {
            region: spec.region.clone(),
            tier: spec.tier.clone(),
        }),
        block,
    }
}

fn broker_bonus(spec: &BrokerOfficeDef) -> String {
    format!(
        "Board x{:.2}, signals +{:.2}, terms x{:.2}. {}",
        spec.board_quality_bonus,
        spec.market_signal_bonus,
        spec.trade_term_modifier,
        ascii_copy(&spec.description)
    )
}

fn family_label(family: &str) -> &str {
    match family {
        "hull" => "Hull",
        "premium_cargo" => "Cargo",
        "contract_guarantee" => "Contract",
        _ => "Policy",
    }
}

fn credit_rank(tier: &str) -> i64 {
    match tier {
        "merchant_line" => 1,
        "house_credit" => 2,
        "premier_commercial" => 3,
        _ => 0,
    }
}

fn good_picks<'a>(lots: impl Iterator<Item = (&'a str, i64)>) -> Vec<GoodPick> {
    totals_by_good(lots)
        .into_iter()
        .map(|(good_id, qty)| GoodPick {
            label: format!("{} x {qty}", good_name(&good_id)),
            good_id,
        })
        .collect()
}

fn totals_by_good<'a>(lots: impl Iterator<Item = (&'a str, i64)>) -> BTreeMap<String, i64> {
    let mut totals = BTreeMap::new();
    for (good_id, qty) in lots {
        *totals.entry(good_id.to_string()).or_default() += qty;
    }
    totals.retain(|_, qty| *qty > 0);
    totals
}

fn good_name(id: &str) -> String {
    content::content()
        .good(id)
        .map(|good| ascii_copy(&good.name))
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| humanize_id(id))
}

fn yes_no(active: bool) -> &'static str {
    if active {
        "yes"
    } else {
        "no"
    }
}

fn format_units(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if (rounded - rounded.round()).abs() < 0.001 {
        format!("{}", rounded.round() as i64)
    } else {
        format!("{rounded:.1}")
    }
}

/// Player copy for a credit tier id: the catalog name (`premier_commercial`
/// reads `Premier Commercial Line`), else [`humanize_id`].
pub(crate) fn credit_tier_name(tier: &str) -> String {
    catalog_or_humanized(
        content::content()
            .credit_tier(tier)
            .map(|spec| spec.name.as_str()),
        tier,
    )
}

/// A catalog name as ASCII copy (an em dash reads `-`), else the humanized
/// id. Never the raw id.
fn catalog_or_humanized(name: Option<&str>, id: &str) -> String {
    name.map(ascii_copy)
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| humanize_id(&ascii_copy(id)))
}

/// License name (`med_trade_charter` reads `Mediterranean Trade Charter`).
pub(crate) fn license_name(id: &str) -> String {
    catalog_or_humanized(
        content::content()
            .license(id)
            .map(|spec| spec.name.as_str()),
        id,
    )
}

/// Insurance policy name (`hull_basic` reads `Basic Hull Insurance`).
pub(crate) fn policy_name(id: &str) -> String {
    catalog_or_humanized(
        content::content().policy(id).map(|spec| spec.name.as_str()),
        id,
    )
}

/// Warehouse tier name (`depot` reads `Small Depot`).
pub(crate) fn warehouse_name(tier: &str) -> String {
    catalog_or_humanized(
        content::content()
            .warehouse_tier(tier)
            .map(|spec| spec.name.as_str()),
        tier,
    )
}

/// Broker office name for a region and tier, else `{Region} {Tier}`.
pub(crate) fn broker_name(region: &str, tier: &str) -> String {
    match content::content().broker(region, tier) {
        Some(spec) => catalog_or_humanized(Some(&spec.name), tier),
        None => format!(
            "{} {}",
            humanize_id(&ascii_copy(region)),
            humanize_id(&ascii_copy(tier))
        ),
    }
}

/// Neutral name for an insured contract that is on no board list.
pub(crate) const SETTLED_CONTRACT: &str = "a settled contract";

/// The contract an insurance policy targets, by the id the policy holds
/// (the offer id):
/// - an active or offered contract: its board title;
/// - a resolved one (delivered, expired, abandoned) in `board.completed`:
///   `{Good} for {Port}` from catalog names;
/// - none of these: [`SETTLED_CONTRACT`]. Contract ids are hashes, so the id
///   is never shown, not even humanized.
pub(crate) fn contract_target_name(session: &Session, id: &str) -> String {
    let board = session.board();
    let title = board
        .active
        .iter()
        .find(|contract| contract.offer_id == id)
        .map(|contract| contract.title.as_str())
        .or_else(|| {
            board
                .offers
                .iter()
                .find(|offer| offer.id == id)
                .map(|offer| offer.title.as_str())
        })
        .map(|title| ascii_copy(title).trim().to_string())
        .filter(|title| !title.is_empty());
    if let Some(title) = title {
        return title;
    }
    board
        .completed
        .iter()
        .find(|outcome| outcome.contract_id == id)
        .map(|outcome| {
            format!(
                "{} for {}",
                good_name(&outcome.good_id),
                port_name(session, &outcome.destination_port_id)
            )
        })
        .unwrap_or_else(|| SETTLED_CONTRACT.to_string())
}

/// Insurance-frame stage, built from `Session` verbs only: a merchant at the
/// start port holds a voyage policy (start port to Corsair's Rest) and a
/// contract guarantee whose contract has since expired undelivered, so the
/// desk shows a `Voyage A to B` row, a settled-contract `Target` row and the
/// guarantee's claim.
pub(crate) fn insurance_stage_session() -> Result<Session, String> {
    let mut session = Session::new("Ada", "merchant", 1, None).map_err(|err| err.to_string())?;
    let here = session.world().voyage.destination_id.clone();
    session
        .buy_insurance("hull_basic", "", &here, "corsairs_rest")
        .map_err(|err| format!("voyage policy: {err}"))?;
    let offer = session
        .board()
        .offers
        .first()
        .cloned()
        .ok_or_else(|| "no contract offer".to_string())?;
    session
        .accept_contract(&offer.id)
        .map_err(|err| format!("accept: {err}"))?;
    session
        .buy_insurance("contract_basic", &offer.id, "", "")
        .map_err(|err| format!("guarantee: {err}"))?;
    for _ in 0..80 {
        session.advance().map_err(|err| format!("advance: {err}"))?;
        let settled = session
            .board()
            .completed
            .iter()
            .any(|outcome| outcome.contract_id == offer.id);
        if settled {
            return Ok(session);
        }
    }
    Err("the guaranteed contract never settled".to_string())
}

/// Port name for a policy voyage end, else the humanized id.
fn port_name(session: &Session, id: &str) -> String {
    catalog_or_humanized(session.world().port(id).map(|port| port.name.as_str()), id)
}

/// One Harbour claims row. A paid contract guarantee reads
/// `Day 21: contract guarantee paid 105.` (no currency word); any other
/// incident is humanised, never a raw id (`Day 5: Storm payout 40.`).
fn claim_line(day: i64, incident: &str, denied: bool, reason: &str, payout: i64) -> String {
    let name = humanize_id(&ascii_copy(incident));
    if denied {
        format!("Day {day}: {name} denied. {}", ascii_copy(reason))
    } else if incident == "contract_failure" {
        format!("Day {day}: contract guarantee paid {payout}.")
    } else {
        format!("Day {day}: {name} payout {payout}.")
    }
}

/// `Trust Trusted. Standing 3. ` for an offer's detail line. The tier goes
/// through the humaniser; a zero standing is omitted (R4).
fn requirement_terms(tier: &str, standing: i64) -> String {
    let mut out = format!("Trust {}. ", humanize_id(&ascii_copy(tier)));
    if standing != 0 {
        out.push_str(&format!("Standing {standing}. "));
    }
    out
}

/// The sim refusal `Requires trusted trust (currently credible)` reads
/// `Requires trust tier Trusted - you are Credible`. Guarded format of the one
/// fixed sim template: only when the text starts with the exact prefix built
/// from the same spec and ends with `)`. Any other refusal prints unchanged.
pub(crate) fn trust_block(sim_text: &str, required_tier: &str) -> String {
    let prefix = format!("Requires {required_tier} trust (currently ");
    let current = sim_text
        .strip_prefix(prefix.as_str())
        .and_then(|rest| rest.strip_suffix(')'));
    match current {
        Some(current) if !current.is_empty() && !current.contains(['(', ')']) => format!(
            "Requires trust tier {} - you are {}",
            humanize_id(required_tier),
            humanize_id(current)
        ),
        _ => sim_text.to_string(),
    }
}

/// The credit half of the desk status line: `Merchant Line outstanding 40`.
fn credit_status(tier: &str, owed: impl std::fmt::Display) -> String {
    format!("{} outstanding {owed}", credit_tier_name(tier))
}

/// Catalog text for the desk. An em dash becomes a hyphen. Other non-ASCII
/// becomes a space so the overlay stays ASCII.
pub(crate) fn ascii_copy(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\u{2014}' | '\u{2013}' | '\u{2212}' => out.push('-'),
            c if c.is_ascii() => out.push(c),
            _ => out.push(' '),
        }
    }
    out
}

/// A positive whole number. Empty input is rejected by the caller.
pub(crate) fn parse_positive(text: &str) -> Result<i64, &'static str> {
    let text = text.trim();
    match text.parse::<i64>() {
        Ok(value) if value > 0 => Ok(value),
        _ => Err("Amount must be positive"),
    }
}

/// Gold subhead, same as Contracts, Shipyard, Journal, Crew, and Hunt:
/// section heads are 16 px GOLD on every desk.
pub(crate) fn section_label(text: &str) -> Gd<Label> {
    let mut label = text_label(text, 16, GOLD);
    label.set_autowrap_mode(AutowrapMode::WORD_SMART);
    label
}

pub(crate) fn muted_label(text: &str) -> Gd<Label> {
    let mut label = text_label(text, 14, MUTED);
    label.set_autowrap_mode(AutowrapMode::WORD_SMART);
    label
}

pub(crate) fn cream_label(text: &str) -> Gd<Label> {
    let mut label = text_label(text, 15, CREAM);
    label.set_autowrap_mode(AutowrapMode::WORD_SMART);
    label
}

/// Red fill, gold border. Emergency loan only. The other desk buttons keep
/// the encounter fill.
pub(crate) fn style_danger_button(button: &mut Gd<Button>) {
    let normal = danger_fill(Color::from_rgb(0.45, 0.16, 0.14));
    let hover = danger_fill(Color::from_rgb(0.62, 0.24, 0.20));
    let pressed = danger_fill(Color::from_rgb(0.32, 0.10, 0.10));
    button.add_theme_stylebox_override("normal", &normal);
    button.add_theme_stylebox_override("hover", &hover);
    button.add_theme_stylebox_override("pressed", &pressed);
    button.add_theme_stylebox_override("focus", &hover);
    button.add_theme_color_override("font_color", CREAM);
    button.add_theme_color_override("font_hover_color", INK);
    button.add_theme_color_override("font_pressed_color", GOLD);
    button.add_theme_color_override("font_focus_color", INK);
    button.add_theme_stylebox_override("disabled", &danger_fill(Color::from_rgb(0.28, 0.16, 0.16)));
}

fn danger_fill(fill: Color) -> Gd<StyleBoxFlat> {
    let mut style = StyleBoxFlat::new_gd();
    style.set_bg_color(fill);
    style.set_border_color(GOLD);
    style.set_border_width_all(2);
    style.set_content_margin_all(8.0);
    style.set_corner_radius_all(2);
    style
}

fn text_label(text: &str, size: i32, color: Color) -> Gd<Label> {
    let mut label = Label::new_alloc();
    label.set_text(text);
    label.add_theme_font_size_override("font_size", size);
    label.add_theme_color_override("font_color", color);
    label
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exact copy, both tiers humanized; any other
    /// refusal prints unchanged.
    #[test]
    fn trust_block_rewrites_only_the_fixed_sim_template() {
        assert_eq!(
            trust_block("Requires trusted trust (currently credible)", "trusted"),
            "Requires trust tier Trusted - you are Credible"
        );
        for other in [
            "Requires 10 standing in Mediterranean",
            "Requires trusted trust",
            "Requires trusted trust (currently )",
            "Requires credible trust (currently unknown)",
        ] {
            assert_eq!(trust_block(other, "trusted"), other);
        }
        let text = trust_block("Requires trusted trust (currently credible)", "trusted");
        assert!(!text.contains("trusted trust") && text.is_ascii());
    }

    /// `Standing 0.` is omitted (R4); the tier is humanized.
    #[test]
    fn requirement_terms_omit_zero_standing() {
        assert_eq!(requirement_terms("trusted", 0), "Trust Trusted. ");
        assert_eq!(
            requirement_terms("credible", 3),
            "Trust Credible. Standing 3. "
        );
    }

    /// Claims ledger rows: the contract guarantee reads in the agreed words; other
    /// incidents are humanized, never raw ids.
    #[test]
    fn claim_lines_name_the_guarantee() {
        assert_eq!(
            claim_line(21, "contract_failure", false, "", 105),
            "Day 21: contract guarantee paid 105."
        );
        assert_eq!(
            claim_line(4, "storm_damage", false, "", 30),
            "Day 4: Storm Damage payout 30."
        );
        assert_eq!(
            claim_line(4, "storm_damage", true, "Policy lapsed.", 0),
            "Day 4: Storm Damage denied. Policy lapsed."
        );
        for line in [
            claim_line(21, "contract_failure", false, "", 105),
            claim_line(4, "storm_damage", true, "x", 0),
        ] {
            assert!(!line.contains('_') && line.is_ascii(), "{line}");
        }
    }

    #[test]
    fn credit_status_uses_the_catalog_tier_name() {
        assert_eq!(
            credit_status("premier_commercial", 0),
            "Premier Commercial Line outstanding 0"
        );
        assert_eq!(
            credit_status("merchant_line", 40),
            "Merchant Line outstanding 40"
        );
        assert_eq!(credit_tier_name("house_credit"), "House Credit");
        assert_eq!(credit_tier_name("no_such_tier"), "No Such Tier");
    }

    fn merchant() -> Session {
        Session::new("Ada", "merchant", 1, None).unwrap()
    }

    fn intents(model: &HarbourModel) -> Vec<HarbourIntent> {
        model
            .warehouse_offers
            .iter()
            .chain(model.broker_offers.iter())
            .chain(model.license_offers.iter())
            .chain(model.credit_offers.iter())
            .chain(model.insurance_offers.iter())
            .filter_map(|row| row.intent.clone())
            .collect()
    }

    #[test]
    fn porto_novo_desk_lists_the_catalog_and_skips_dry_dock() {
        let session = merchant();
        let model = harbour_model(&session).unwrap();
        assert_eq!(model.port_name, "Porto Novo");
        assert!(model
            .warehouse_lines
            .iter()
            .any(|line| line.contains("No warehouse leased here.")));
        assert_eq!(model.warehouse_offers.len(), 3);
        assert_eq!(model.broker_offers.len(), 2);
        assert_eq!(model.license_offers.len(), 7);
        assert_eq!(model.credit_offers.len(), 3);
        assert!(model.insurance_offers.len() >= 6);
        assert!(model.status.contains("Silver"));
        assert!(model
            .warehouse_lines
            .iter()
            .any(|line| line.contains("There is no cancel")));
        for intent in intents(&model) {
            assert_ne!(infrastructure_kind(&intent), Some("dry_dock"));
            if let Some(kind) = infrastructure_kind(&intent) {
                assert!(matches!(kind, "warehouse" | "broker" | "license"));
            }
        }
    }

    #[test]
    fn license_and_contract_rows_use_session_shaped_blocks() {
        let session = merchant();
        let model = harbour_model(&session).unwrap();
        let charter = model
            .license_offers
            .iter()
            .find(|row| row.intent.is_none() && row.block.to_lowercase().contains("broker"))
            .expect("charter needs a broker");
        assert!(charter.block.contains("Mediterranean"));
        assert!(model.insurance_offers.iter().any(|row| {
            row.title.starts_with("Contract:") && row.block == "No contract to guarantee."
        }));
        assert!(model.credit_offers.iter().any(|row| {
            matches!(row.intent, Some(HarbourIntent::OpenCredit(ref tier)) if tier == "merchant_line")
        }));
    }

    #[test]
    fn confirm_copy_is_ascii_and_names_the_fee() {
        let session = merchant();
        let lease = confirm_prompt(&session, &HarbourIntent::LeaseWarehouse("depot".into()));
        assert!(lease.is_ascii());
        assert!(lease.contains("50 silver"));
        assert!(lease.contains("upkeep fails"));
        let loan = confirm_prompt(&session, &HarbourIntent::Emergency(50));
        assert!(loan.is_ascii());
        assert!(loan.contains("15%"));
        assert!(loan.contains("200"));
        let policy = confirm_prompt(
            &session,
            &HarbourIntent::BuyInsurance {
                policy_id: "contract_basic".into(),
                target_id: "offer-1".into(),
                origin: String::new(),
                destination: String::new(),
            },
        );
        assert!(policy.is_ascii());
        assert!(!policy.contains('\u{2014}'));
    }

    #[test]
    fn at_sea_has_no_desk() {
        let mut session = merchant();
        session.depart("al_manar").unwrap();
        assert!(harbour_model(&session).is_none());
    }

    #[test]
    fn advance_notes_survive_the_log_helper() {
        let mut session = merchant();
        session.buy_infrastructure("warehouse", &["depot"]).unwrap();
        session.buy("grain", 1).unwrap();
        assert_eq!(session.deposit_cargo("grain", 1).unwrap(), 1);
        session.take_credit("merchant_line", 40).unwrap();
        session.buy_insurance("hull_basic", "", "", "").unwrap();
        let mut saw = false;
        for _ in 0..15 {
            let turn = session.advance().unwrap();
            if turn.notes.iter().any(|note| {
                note.contains("Interest accrued")
                    || note.contains("Credit payment")
                    || note.contains("DEFAULT")
                    || note.contains("seized")
            }) {
                let lines = crate::logic::day_log_lines(&[], &[], &turn.notes, Some("Docked"));
                assert!(lines
                    .iter()
                    .any(|line| turn.notes.iter().any(|note| note == line)));
                saw = true;
                break;
            }
        }
        assert!(saw, "expected a credit or upkeep note from advance");
    }

    /// A catalog miss reads as humanized copy, never the raw id; a known id
    /// keeps its catalog name (ASCII).
    #[test]
    fn catalog_misses_never_print_raw_ids() {
        let session = merchant();
        let miss = "salt_spit_cove";
        let human = "Salt Spit Cove";
        assert_eq!(license_name(miss), human);
        assert_eq!(policy_name(miss), human);
        assert_eq!(warehouse_name(miss), human);
        assert_eq!(credit_tier_name(miss), human);
        assert_eq!(broker_name("salt_spit", "cove"), human);
        assert_eq!(contract_target_name(&session, miss), SETTLED_CONTRACT);
        assert_eq!(
            contract_target_name(&session, "b21e3e594ab0"),
            SETTLED_CONTRACT
        );
        assert_eq!(port_name(&session, miss), human);

        assert_eq!(
            license_name("med_trade_charter"),
            "Mediterranean Trade Charter"
        );
        assert_eq!(policy_name("contract_basic"), "Contract Guarantee - Basic");
        assert_eq!(warehouse_name("depot"), "Small Depot");
        assert_eq!(
            broker_name("Mediterranean", "local"),
            "Mediterranean Local Broker"
        );
        assert_eq!(port_name(&session, "porto_novo"), "Porto Novo");

        for intent in [
            HarbourIntent::LeaseWarehouse(miss.into()),
            HarbourIntent::OpenBroker {
                region: "salt_spit".into(),
                tier: "cove".into(),
            },
            HarbourIntent::BuyLicense(miss.into()),
            HarbourIntent::OpenCredit(miss.into()),
            HarbourIntent::Draw {
                tier: miss.into(),
                amount: 10,
            },
            HarbourIntent::BuyInsurance {
                policy_id: miss.into(),
                target_id: String::new(),
                origin: String::new(),
                destination: String::new(),
            },
        ] {
            let prompt = confirm_prompt(&session, &intent);
            assert!(!prompt.contains('_'), "{prompt}");
            assert!(
                prompt.contains(human) || prompt.contains("Salt Spit"),
                "{prompt}"
            );
            assert!(prompt.is_ascii(), "{prompt}");
        }
    }

    /// A voyage policy row names both ports in words, no arrow.
    #[test]
    fn voyage_policy_row_reads_port_to_port() {
        let mut session = merchant();
        let here = session.world().voyage.destination_id.clone();
        session
            .buy_insurance("hull_basic", "", &here, "corsairs_rest")
            .expect("voyage policy");
        let model = harbour_model(&session).expect("docked desk");
        let here_name = port_name(&session, &here);
        let line = model
            .policy_lines
            .iter()
            .find(|line| line.contains(" Voyage "))
            .expect("voyage row");
        assert!(
            line.ends_with(&format!(" Voyage {here_name} to Corsair's Rest.")),
            "{line}"
        );
        assert!(!line.contains("->"), "{line}");
    }

    /// A guarantee whose contract has expired names it from the settled
    /// outcome (`{Good} for {Port}`), never the hash id.
    #[test]
    fn settled_insured_contract_reads_good_for_port() {
        let mut session = merchant();
        let offer = session.board().offers.first().cloned().expect("an offer");
        session.accept_contract(&offer.id).expect("accept");
        session
            .buy_insurance("contract_basic", &offer.id, "", "")
            .expect("guarantee");
        // Stay docked past the deadline: the contract expires undelivered.
        let mut outcome = None;
        for _ in 0..80 {
            session.advance().expect("advance");
            outcome = session
                .board()
                .completed
                .iter()
                .find(|outcome| outcome.contract_id == offer.id)
                .cloned();
            if outcome.is_some() {
                break;
            }
        }
        let outcome = outcome.expect("settled outcome");
        assert_eq!(outcome.outcome_type, "expired");
        let want = format!(
            "{} for {}",
            good_name(&outcome.good_id),
            port_name(&session, &outcome.destination_port_id)
        );
        assert_eq!(contract_target_name(&session, &offer.id), want);
        let model = harbour_model(&session).expect("docked desk");
        let row = model
            .policy_lines
            .iter()
            .find(|line| line.contains(" Target "))
            .expect("guarantee row");
        assert!(row.contains(&format!(" Target {want}.")), "{row}");
        assert!(!row.contains(&offer.id), "{row}");
        assert!(
            !row.to_lowercase().contains(&offer.id.to_lowercase()),
            "{row}"
        );
    }

    /// The insurance prompt names the target contract by its title.
    #[test]
    fn insurance_prompt_names_the_contract_title() {
        let mut session = merchant();
        let offer = session.board().offers.first().cloned().expect("an offer");
        session.accept_contract(&offer.id).expect("accept");
        let prompt = confirm_prompt(
            &session,
            &HarbourIntent::BuyInsurance {
                policy_id: "contract_basic".into(),
                target_id: offer.id.clone(),
                origin: String::new(),
                destination: String::new(),
            },
        );
        assert!(
            prompt.contains(&format!("for contract {}", ascii_copy(&offer.title))),
            "{prompt}"
        );
        assert!(!prompt.contains(&offer.id), "{prompt}");
        assert_eq!(
            contract_target_name(&session, &offer.id),
            ascii_copy(&offer.title)
        );
    }

    /// An offered contract not yet accepted reads by its offer title.
    #[test]
    fn offered_contract_reads_its_offer_title() {
        let session = merchant();
        let offer = session.board().offers.first().cloned().expect("an offer");
        assert!(session
            .board()
            .active
            .iter()
            .all(|contract| contract.offer_id != offer.id));
        let title = ascii_copy(&offer.title).trim().to_string();
        assert!(!title.is_empty());
        assert_eq!(contract_target_name(&session, &offer.id), title);
    }

    /// A guarantee target that is not an active contract (an offer, a settled
    /// contract) drops the word "contract": `Buy {name} for {target} at N silver?`.
    #[test]
    fn insurance_prompt_without_an_active_target() {
        let session = merchant();
        let offer = session.board().offers.first().cloned().expect("an offer");
        let intent = |target: &str| HarbourIntent::BuyInsurance {
            policy_id: "contract_basic".into(),
            target_id: target.into(),
            origin: String::new(),
            destination: String::new(),
        };
        let name = policy_name("contract_basic");
        let offered = confirm_prompt(&session, &intent(&offer.id));
        let title = ascii_copy(&offer.title).trim().to_string();
        assert!(
            offered.starts_with(&format!("Buy {name} for {title} at ")),
            "{offered}"
        );
        assert!(offered.ends_with(" silver?"), "{offered}");
        assert!(!offered.contains("for contract "), "{offered}");
        let settled = confirm_prompt(&session, &intent("b21e3e594ab0"));
        assert!(
            settled.starts_with(&format!("Buy {name} for {SETTLED_CONTRACT} at ")),
            "{settled}"
        );
        assert!(!settled.contains("b21e3e594ab0"), "{settled}");
    }

    /// The insurance-frame stage shows both rows the frame exists for: the
    /// voyage policy in words and the settled contract by good and port.
    #[test]
    fn insurance_stage_shows_voyage_and_settled_rows() {
        let session = insurance_stage_session().expect("stage");
        let model = harbour_model(&session).expect("docked desk");
        let lines = model.policy_lines.join("\n");
        assert!(lines.contains(" to Corsair's Rest."), "{lines}");
        let target = model
            .policy_lines
            .iter()
            .find(|line| line.contains(" Target "))
            .expect("guarantee row");
        assert!(target.contains(" for "), "{target}");
        assert!(!target.contains(SETTLED_CONTRACT), "{target}");
        assert!(lines.is_ascii(), "{lines}");
    }
}
