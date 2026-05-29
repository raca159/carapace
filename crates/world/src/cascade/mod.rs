use bevy_ecs::prelude::Resource;
use serde::Deserialize;
use std::collections::HashMap;

pub mod equipment;
pub mod inventory;
pub mod economy;
pub mod locations;
pub mod trade;
pub mod inventory_populate;

#[derive(Debug, Clone, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub glyph: char,
    pub color: [u8; 3],
    pub base_value: u32,
    pub tags: Vec<String>,
    pub weight: f32,
    #[serde(default)]
    pub quality_bias: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ItemsToml {
    #[serde(rename = "item")]
    items: Vec<ItemDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegionBiome {
    pub tags: Vec<String>,
    pub produces: Vec<TaggedWeight>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaggedWeight {
    pub tag: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct RegionBiomesToml {
    #[serde(rename = "biome")]
    biomes: Vec<RegionBiome>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FactionEconomy {
    pub id: String,
    pub name: String,
    pub produces: Vec<TaggedWeight>,
    pub consumes: Vec<TaggedWeight>,
}

#[derive(Debug, Clone, Deserialize)]
struct FactionEconomyToml {
    #[serde(rename = "faction")]
    factions: Vec<FactionEconomy>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocationType {
    pub id: String,
    pub name: String,
    pub pass: u32,
    pub weight: f32,
    pub min_distance_from_same: u32,
    pub zone_radius: u32,
    pub habitability_threshold: f32,
    #[serde(default)]
    pub tags: Vec<String>,
    pub biome_affinity: Vec<String>,
    #[serde(default)]
    pub faction_affinity: Vec<String>,
    #[serde(default)]
    pub interior: Option<InteriorDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InteriorDef {
    pub generator: Option<String>,
    pub tileset: Option<String>,
    pub scale: Option<[u32; 2]>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub environment: HashMap<String, u32>,
    #[serde(default)]
    pub spawn_rules: Vec<String>,
    pub depth_range: Option<[u32; 2]>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocationTypesToml {
    #[serde(rename = "location_type")]
    types: Vec<LocationType>,
}

#[derive(Debug, Clone, Resource, Default)]
pub struct LocationMap {
    pub locations: Vec<PlacedLocation>,
}

#[derive(Debug, Clone)]
pub struct PlacedLocation {
    pub id: usize,
    pub location_type: String,
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub zone_radius: u32,
    pub tags: Vec<String>,
    pub faction: Option<String>,
}

#[derive(Debug, Clone, Resource, Default)]
pub struct RegionEconomies {
    pub economies: HashMap<usize, crate::cascade::economy::PricingContext>,
}

#[derive(Resource, Debug, Clone)]
pub struct CascadeEngine {
    pub items: Vec<ItemDef>,
    pub item_by_id: HashMap<String, ItemDef>,
    pub region_biomes: Vec<RegionBiome>,
    pub faction_economies: Vec<FactionEconomy>,
    pub location_types: Vec<LocationType>,
}

impl CascadeEngine {
    pub fn load(
        items_toml: &str,
        biomes_toml: &str,
        factions_toml: &str,
        location_types_toml: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let items_file: ItemsToml = toml::from_str(items_toml)?;
        let biomes_file: RegionBiomesToml = toml::from_str(biomes_toml)?;
        let factions_file: FactionEconomyToml = toml::from_str(factions_toml)?;
        let location_file: LocationTypesToml = toml::from_str(location_types_toml)?;

        let item_by_id: HashMap<String, ItemDef> = items_file.items.iter()
            .map(|item| (item.id.clone(), item.clone()))
            .collect();

        Ok(Self {
            items: items_file.items,
            item_by_id,
            region_biomes: biomes_file.biomes,
            faction_economies: factions_file.factions,
            location_types: location_file.types,
        })
    }

    pub fn items_with_tags(&self, required_tags: &[&str], registry: &game_tags::TagRegistry) -> Vec<&ItemDef> {
        let tag_ids: Vec<_> = required_tags.iter()
            .filter_map(|t| registry.tag_id(t))
            .collect();
        self.items.iter()
            .filter(|item| {
                tag_ids.iter().all(|&tid| {
                    item.tags.iter()
                        .filter_map(|t| registry.tag_id(t))
                        .any(|it| it == tid)
                })
            })
            .collect()
    }

    pub fn preferred_material(&self, entity_tags: &game_tags::Tags, registry: &game_tags::TagRegistry) -> Option<&'static str> {
        if entity_tags.has(registry.tag_id("HUMANOID")?) { Some("METAL") }
        else if entity_tags.has(registry.tag_id("UNDEAD")?) { Some("BONE") }
        else if entity_tags.has(registry.tag_id("BEAST")?) { Some("BONE") }
        else if entity_tags.has(registry.tag_id("CONSTRUCT")?) { Some("METAL") }
        else { None }
    }

    pub fn slot_needs(&self, entity_tags: &game_tags::Tags, registry: &game_tags::TagRegistry) -> Vec<&'static str> {
        let mut slots = Vec::new();
        let is_humanoid = registry.tag_id("HUMANOID").is_some_and(|id| entity_tags.has(id));
        let is_aggressive = registry.tag_id("AGGRESSIVE").is_some_and(|id| entity_tags.has(id));
        let is_territorial = registry.tag_id("TERRITORIAL").is_some_and(|id| entity_tags.has(id));

        if is_humanoid || is_aggressive || is_territorial {
            slots.push("EQUIP_WEAPON");
        }
        if is_humanoid || is_territorial {
            slots.push("EQUIP_ARMOR");
        }
        slots
    }
}
