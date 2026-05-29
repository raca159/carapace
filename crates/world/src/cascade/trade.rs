use std::collections::HashMap;
use super::PlacedLocation;
use crate::cascade::economy::PricingContext;

/// A trade connection between two economy locations
#[derive(Debug, Clone)]
pub struct TradeRoute {
    pub from: usize,
    pub to: usize,
    pub distance: f32,
}

/// Resource storing all trade routes for runtime use
#[derive(Debug, Clone, Default, bevy_ecs::prelude::Resource)]
pub struct TradeRoutes(pub Vec<TradeRoute>);

/// Generate trade routes between economy locations
pub fn generate_trade_routes(
    locations: &[PlacedLocation],
    max_routes_per_location: usize,
) -> Vec<TradeRoute> {
    let economy_locations: Vec<&PlacedLocation> = locations.iter()
        .filter(|l| l.tags.iter().any(|t| t == "HAS_ECONOMY"))
        .collect();

    let mut routes = Vec::new();

    for loc in &economy_locations {
        let mut neighbors: Vec<(usize, f32)> = economy_locations.iter()
            .filter(|other| other.id != loc.id)
            .map(|other| {
                let dx = (loc.x as i32 - other.x as i32).unsigned_abs();
                let dy = (loc.y as i32 - other.y as i32).unsigned_abs();
                let dist_sq = (dx * dx + dy * dy) as f32;
                (other.id, dist_sq.sqrt())
            })
            .collect();

        neighbors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        for (target_id, distance) in neighbors.iter().take(max_routes_per_location) {
            // Only add route once (from lower ID to higher ID)
            if loc.id < *target_id || !routes.iter().any(|r: &TradeRoute| r.from == *target_id && r.to == loc.id) {
                routes.push(TradeRoute {
                    from: loc.id,
                    to: *target_id,
                    distance: *distance,
                });
            }
        }
    }

    routes
}

/// Apply trade route effects: blend price multipliers between connected economies
pub fn apply_trade_to_economy(
    routes: &[TradeRoute],
    economies: &mut HashMap<usize, PricingContext>,
) {
    let route_map: HashMap<usize, Vec<&TradeRoute>> = routes.iter()
        .fold(HashMap::new(), |mut acc, r| {
            acc.entry(r.from).or_default().push(r);
            acc.entry(r.to).or_default().push(r);
            acc
        });

    for (&loc_id, connected_routes) in &route_map {
        let Some(pricing) = economies.get(&loc_id) else { continue; };
        let mut adjusted = pricing.price_multipliers.clone();

        for route in connected_routes {
            let partner_id = if route.from == loc_id { route.to } else { route.from };
            let Some(partner) = economies.get(&partner_id) else { continue; };
            let influence = (1.0 - (route.distance / 200.0).min(0.9)) * 0.3;

            for (tag, partner_mult) in &partner.price_multipliers {
                let local_mult = adjusted.get(tag).copied().unwrap_or(1.0);
                let blended = local_mult * (1.0 - influence) + partner_mult * influence;
                adjusted.insert(tag.clone(), blended);
            }
        }

        if let Some(pricing) = economies.get_mut(&loc_id) {
            pricing.price_multipliers = adjusted;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_loc(id: usize, x: u32, y: u32, has_economy: bool) -> PlacedLocation {
        PlacedLocation {
            id, location_type: if has_economy { "city".to_string() } else { "cave".to_string() },
            name: format!("Loc {}", id), x, y, zone_radius: 10,
            tags: if has_economy { vec!["HAS_ECONOMY".to_string()] } else { vec![] },
            faction: None,
        }
    }

    #[test]
    fn trade_routes_link_nearest_economies() {
        let locations = vec![
            make_loc(1, 10, 10, true),
            make_loc(2, 15, 15, true),
            make_loc(3, 80, 80, true),
            make_loc(4, 5, 5, false),  // no economy
        ];
        let routes = generate_trade_routes(&locations, 2);
        assert!(!routes.is_empty(), "should create trade routes");
        assert!(routes.iter().any(|r| r.from == 1 && r.to == 2), "should link close economies");
        assert!(routes.iter().all(|r| r.from != 4 && r.to != 4), "should not link non-economy");
    }

    #[test]
    fn trade_blends_pricing() {
        let locations = vec![make_loc(1, 10, 10, true), make_loc(2, 20, 20, true)];
        let routes = generate_trade_routes(&locations, 2);

        let mut economies = HashMap::new();
        let mut eco1 = HashMap::new();
        eco1.insert("FOOD_WILD".to_string(), 0.5f32);
        let mut eco2 = HashMap::new();
        eco2.insert("FOOD_WILD".to_string(), 2.0f32);

        economies.insert(1, PricingContext { price_multipliers: eco1, prosperity: 0.5, location_supply: Vec::new() });
        economies.insert(2, PricingContext { price_multipliers: eco2, prosperity: 0.5, location_supply: Vec::new() });

        apply_trade_to_economy(&routes, &mut economies);

        // After trade, prices should have blended toward each other
        let p1 = economies.get(&1).unwrap().price_multipliers.get("FOOD_WILD").copied().unwrap_or(0.5);
        let p2 = economies.get(&2).unwrap().price_multipliers.get("FOOD_WILD").copied().unwrap_or(2.0);
        assert!(p1 < 2.0, "trade should lower expensive prices");
        assert!(p2 > 0.5, "trade should raise cheap prices");
    }
}
