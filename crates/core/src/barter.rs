use game_tags::{TagRegistry, Tags};

use crate::reputation::{FactionReputation, ReputationRank};

/// Apply price modifier to a base price based on faction standing.
/// - Hostile / Unfriendly: 200% markup (double price)
/// - Neutral: no change  
/// - Friendly: 15% discount
/// - Honored / Exalted: 30% discount
pub fn apply_faction_price_modifier(
    base_price: u32,
    faction_name: &str,
    reputation: Option<&FactionReputation>,
) -> u32 {
    if let Some(rep) = reputation {
        match rep.rank(faction_name) {
            ReputationRank::Hostile | ReputationRank::Unfriendly => {
                // 2x markup for hostile/unfriendly
                base_price.saturating_mul(2)
            }
            ReputationRank::Friendly => {
                // 15% discount
                (base_price as f64 * 0.85) as u32
            }
            ReputationRank::Honored | ReputationRank::Exalted => {
                // 30% discount
                (base_price as f64 * 0.70) as u32
            }
            ReputationRank::Neutral => base_price,
        }
    } else {
        base_price
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BarterItem {
    pub name: String,
    pub quantity: u32,
    pub base_value: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BarterOffer {
    pub offered: Vec<BarterItem>,
    pub requested: Vec<BarterItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BarterResult {
    pub accepted: bool,
    pub exchanged_items: Vec<BarterItem>,
    pub received_items: Vec<BarterItem>,
    pub value_delta: i64,
}

pub fn value_of_item(
    item: &BarterItem,
    entity_tags: Option<&Tags>,
    tag_registry: Option<&TagRegistry>,
) -> u32 {
    let quality_mult = match (entity_tags, tag_registry) {
        (Some(tags), Some(reg)) => {
            let quality_ids = ["COMMON", "UNCOMMON", "RARE", "EPIC", "LEGENDARY"];
            let mut mult: u32 = 1;
            for qname in &quality_ids {
                if let Some(qid) = reg.tag_id(qname)
                    && tags.has(qid)
                        && let Some(m) = reg.tag_by_id(qid).multiplier {
                            mult = m as u32;
                        }
            }
            mult
        }
        _ => 1,
    };
    item.base_value
        .saturating_mul(quality_mult)
        .saturating_mul(item.quantity)
}

pub fn evaluate_offer(offer: &BarterOffer) -> i64 {
    let offered_value: u64 = offer
        .offered
        .iter()
        .map(|i| i.base_value as u64 * i.quantity as u64)
        .sum();
    let requested_value: u64 = offer
        .requested
        .iter()
        .map(|i| i.base_value as u64 * i.quantity as u64)
        .sum();
    offered_value as i64 - requested_value as i64
}

pub fn resolve_barter(
    offer: &BarterOffer,
    _tags_a: Option<&Tags>,
    _tags_b: Option<&Tags>,
    _registry: Option<&TagRegistry>,
) -> BarterResult {
    let value_delta = evaluate_offer(offer);
    let accepted = value_delta >= 0;
    BarterResult {
        accepted,
        exchanged_items: if accepted {
            offer.offered.clone()
        } else {
            vec![]
        },
        received_items: if accepted {
            offer.requested.clone()
        } else {
            vec![]
        },
        value_delta,
    }
}

pub fn resolve_barter_with_haggle(
    offer: &BarterOffer,
    tags_a: Option<&Tags>,
    tags_b: Option<&Tags>,
    tag_registry: Option<&TagRegistry>,
    rng: &mut impl rand::Rng,
) -> BarterResult {
    let raw_delta = evaluate_offer(offer);

    let npc_greed = match (tags_b, tag_registry) {
        (Some(tags), Some(reg))
            if reg.tag_id("GREEDY").is_some_and(|id| tags.has(id)) => {
                1.3
            }
        _ => 1.0,
    };

    let npc_trust = match (tags_a, tag_registry) {
        (Some(tags), Some(reg))
            if reg.tag_id("CURIOUS").is_some_and(|id| tags.has(id)) => {
                0.9
            }
        _ => 1.0,
    };

    let noise = rng.random_range(-0.05f32..0.05f32);
    let acceptance_ratio = npc_greed * npc_trust * (1.0 + noise);

    let accepted = (raw_delta as f32) * acceptance_ratio >= 0.0;

    BarterResult {
        accepted,
        exchanged_items: if accepted {
            offer.offered.clone()
        } else {
            vec![]
        },
        received_items: if accepted {
            offer.requested.clone()
        } else {
            vec![]
        },
        value_delta: raw_delta,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn make_item(name: &str, base_value: u32, quantity: u32) -> BarterItem {
        BarterItem {
            name: name.to_string(),
            quantity,
            base_value,
        }
    }

    #[test]
    fn test_fair_trade_accepted() {
        let offer = BarterOffer {
            offered: vec![make_item("Chip", 10, 5)],
            requested: vec![make_item("Metal Blade", 50, 1)],
        };
        let result = resolve_barter(&offer, None, None, None);
        assert!(result.accepted);
        assert_eq!(result.exchanged_items.len(), 1);
        assert_eq!(result.received_items.len(), 1);
        assert_eq!(result.value_delta, 0);
    }

    #[test]
    fn test_unfair_trade_rejected() {
        let offer = BarterOffer {
            offered: vec![make_item("Chip", 10, 1)],
            requested: vec![make_item("Metal Blade", 50, 1)],
        };
        let result = resolve_barter(&offer, None, None, None);
        assert!(!result.accepted);
        assert_eq!(result.value_delta, -40);
    }

    #[test]
    fn test_generous_trade_accepted() {
        let offer = BarterOffer {
            offered: vec![make_item("Chip", 10, 10)],
            requested: vec![make_item("Metal Blade", 50, 1)],
        };
        let result = resolve_barter(&offer, None, None, None);
        assert!(result.accepted);
        assert_eq!(result.value_delta, 50);
    }

    #[test]
    fn test_value_of_item_with_quality_mult() {
        let mut builder = game_tags::TagRegistryBuilder::new();
        let quality = builder.add_archetype("quality", "Quality", game_tags::Exclusivity::Mutual);
        builder
            .add_tag(
                quality,
                "RARE",
                vec![],
                vec![],
                None,
                None,
                Some(3.0),
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
        let builder_registry = builder.build().unwrap();

        let mut tags = Tags::new(builder_registry.tag_count());
        let rare_id = builder_registry.tag_id("RARE").unwrap();
        tags.add_tag(rare_id, game_tags::TagValue::None, &builder_registry);

        let item = make_item("Gem", 100, 1);
        let value = value_of_item(&item, Some(&tags), Some(&builder_registry));
        assert_eq!(value, 300, "RARE quality should triple base value");
    }

    #[test]
    fn test_haggle_with_greedy_npc() {
        let mut builder = game_tags::TagRegistryBuilder::new();
        let arch = builder.add_archetype("trait", "Trait", game_tags::Exclusivity::Any);
        builder
            .add_tag(
                arch,
                "GREEDY",
                vec![],
                vec![],
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
        let registry = builder.build().unwrap();

        let mut npc_tags = Tags::new(registry.tag_count());
        let greedy_id = registry.tag_id("GREEDY").unwrap();
        npc_tags.add_tag(greedy_id, game_tags::TagValue::None, &registry);

        let generous_offer = BarterOffer {
            offered: vec![make_item("Chip", 10, 10)],
            requested: vec![make_item("Blade", 50, 1)],
        };

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let result = resolve_barter_with_haggle(
            &generous_offer,
            None,
            Some(&npc_tags),
            Some(&registry),
            &mut rng,
        );

        assert!(
            result.accepted,
            "generous offer (100 for 50) should be accepted even by greedy NPC"
        );
        assert_eq!(result.exchanged_items.len(), 1);
    }

    #[test]
    fn test_greedy_npc_rejects_underpay() {
        let mut builder = game_tags::TagRegistryBuilder::new();
        let arch = builder.add_archetype("trait", "Trait", game_tags::Exclusivity::Any);
        builder
            .add_tag(
                arch,
                "GREEDY",
                vec![],
                vec![],
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
        let registry = builder.build().unwrap();

        let mut npc_tags = Tags::new(registry.tag_count());
        npc_tags.add_tag(
            registry.tag_id("GREEDY").unwrap(),
            game_tags::TagValue::None,
            &registry,
        );

        let underpay = BarterOffer {
            offered: vec![make_item("Chip", 10, 4)],
            requested: vec![make_item("Blade", 50, 1)],
        };

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let result =
            resolve_barter_with_haggle(&underpay, None, Some(&npc_tags), Some(&registry), &mut rng);

        assert!(
            !result.accepted,
            "greedy NPC should reject underpay (40 for 50)"
        );
    }

    #[test]
    fn test_haggle_deterministic_seed() {
        let mut builder = game_tags::TagRegistryBuilder::new();
        let arch = builder.add_archetype("trait", "Trait", game_tags::Exclusivity::Any);
        builder
            .add_tag(
                arch,
                "GREEDY",
                vec![],
                vec![],
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
        let registry = builder.build().unwrap();

        let mut npc_tags = Tags::new(registry.tag_count());
        npc_tags.add_tag(
            registry.tag_id("GREEDY").unwrap(),
            game_tags::TagValue::None,
            &registry,
        );

        let offer = BarterOffer {
            offered: vec![make_item("Chip", 10, 4)],
            requested: vec![make_item("Blade", 50, 1)],
        };

        let run = || -> bool {
            let mut rng = rand::rngs::StdRng::seed_from_u64(42);
            resolve_barter_with_haggle(&offer, None, Some(&npc_tags), Some(&registry), &mut rng)
                .accepted
        };

        assert_eq!(run(), run());
    }
}
