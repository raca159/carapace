use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TagsToml {
    #[serde(rename = "archetype")]
    pub archetypes: Vec<ArchetypeToml>,
}

#[derive(Debug, Deserialize)]
pub struct ArchetypeToml {
    pub id: String,
    pub name: String,
    pub exclusivity: String,
    #[serde(default)]
    pub value_field: Option<String>,
    pub tags: Vec<TagToml>,
}

#[derive(Debug, Deserialize)]
pub struct TagToml {
    pub id: String,
    #[serde(default)]
    pub implies: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub default_magnitude: Option<f32>,
    #[serde(default)]
    pub ticks: Option<[u32; 2]>,
    #[serde(default)]
    pub multiplier: Option<f32>,
    #[serde(default)]
    pub move_cost: Option<f32>,
    #[serde(default)]
    pub range: Option<u32>,
    #[serde(default)]
    pub threshold: Option<[u32; 2]>,
    #[serde(default)]
    pub tile_occupancy: Option<f32>,
    #[serde(default)]
    pub hp_mult: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct InteractionsToml {
    #[serde(rename = "interaction")]
    pub interactions: Vec<InteractionRuleToml>,
}

#[derive(Debug, Deserialize)]
pub struct InteractionRuleToml {
    pub tag_a: String,
    pub tag_b: String,
    #[serde(default)]
    pub produces: Vec<String>,
    #[serde(default)]
    pub consumes: Vec<String>,
    pub priority: u32,
    #[serde(default)]
    pub description: String,
}
