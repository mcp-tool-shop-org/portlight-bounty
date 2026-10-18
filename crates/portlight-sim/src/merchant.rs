//! Merchant price markup and regional stock from `engine/merchant.py`.
//!
//! Buying an item uses the weapon, armor, and ammo catalogs filtered by the
//! port's region. The markup itself is `max(1, round(base * markup))`.

use crate::content;
use crate::util::py_round;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopEntry {
    pub item_type: String,
    pub item_id: String,
    pub silver_cost: i64,
}

pub fn marked_price(base_cost: i64, markup: f64) -> i64 {
    1.max(py_round(base_cost as f64 * markup))
}

fn merchant_sells(types: &[String], kind: &str) -> bool {
    types.iter().any(|item| item == kind)
}

fn in_region(regions: &[String], region: &str) -> bool {
    regions.iter().any(|item| item == region)
}

/// `get_merchant_inventory` for the port's region.
pub fn merchant_inventory(merchant: &content::MerchantDef, region: &str) -> Vec<ShopEntry> {
    let catalog = content::content();
    let mut items = Vec::new();
    if merchant_sells(&merchant.inventory_types, "melee") {
        for weapon in catalog
            .melee_weapons
            .iter()
            .filter(|weapon| in_region(&weapon.available_regions, region))
        {
            items.push(ShopEntry {
                item_type: "melee".into(),
                item_id: weapon.id.clone(),
                silver_cost: marked_price(weapon.silver_cost, merchant.price_markup),
            });
        }
    }
    if merchant_sells(&merchant.inventory_types, "armor") {
        for armor in catalog
            .armor
            .iter()
            .filter(|armor| in_region(&armor.available_regions, region))
        {
            items.push(ShopEntry {
                item_type: "armor".into(),
                item_id: armor.id.clone(),
                silver_cost: marked_price(armor.silver_cost, merchant.price_markup),
            });
        }
    }
    if merchant_sells(&merchant.inventory_types, "ranged") {
        for weapon in catalog
            .ranged_weapons
            .iter()
            .filter(|weapon| in_region(&weapon.available_regions, region))
        {
            items.push(ShopEntry {
                item_type: "ranged".into(),
                item_id: weapon.id.clone(),
                silver_cost: marked_price(weapon.silver_cost, merchant.price_markup),
            });
        }
    }
    if merchant_sells(&merchant.inventory_types, "ammo") {
        for ammo in catalog
            .ammo
            .iter()
            .filter(|ammo| in_region(&ammo.available_regions, region))
        {
            items.push(ShopEntry {
                item_type: "ammo".into(),
                item_id: ammo.id.clone(),
                silver_cost: marked_price(ammo.silver_cost, merchant.price_markup),
            });
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content;

    #[test]
    fn marco_marks_a_blade_up_by_ten_percent() {
        let marco = content::content().merchant("marco_the_blade").unwrap();
        assert!((marco.price_markup - 1.10).abs() < 1e-9);
        assert_eq!(marked_price(15, marco.price_markup), py_round_expected());
        assert_eq!(marked_price(0, 1.1), 1);
    }

    #[test]
    fn marco_inventory_filters_by_mediterranean_and_types() {
        let marco = content::content().merchant("marco_the_blade").unwrap();
        let stock = merchant_inventory(marco, "Mediterranean");
        assert!(stock
            .iter()
            .any(|item| item.item_id == "cutlass" && item.silver_cost == 28));
        assert!(stock
            .iter()
            .any(|item| item.item_id == "matchlock_pistol" && item.silver_cost == 55));
        assert!(stock
            .iter()
            .any(|item| item.item_id == "pistol_shot" && item.silver_cost == 6));
        assert!(!stock.iter().any(|item| item.item_type == "armor"));
    }

    fn py_round_expected() -> i64 {
        // 15 * 1.10 = 16.5, and Python 3 rounds halves to even → 16.
        16
    }
}
