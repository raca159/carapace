use game_tags::{
    load_interaction_rules, load_tag_registry, InteractionRules, TagRegistry, Tags,
    TagValue,
};

const TEST_TAGS_TOML: &str = r#"
[[archetype]]
id = "element"
name = "Element"
exclusivity = "mutual"

[[archetype.tags]]
id = "FIRE"
implies = ["HOT", "LUMINESCENT"]

[[archetype.tags]]
id = "WATER"
implies = ["WET"]

[[archetype.tags]]
id = "ICE"
implies = ["COLD", "SLIPPERY"]

[[archetype.tags]]
id = "LIGHTNING"
implies = ["CONDUCTIVE", "LUMINESCENT"]

[[archetype.tags]]
id = "ACID"
implies = ["TOXIC"]

[[archetype.tags]]
id = "EARTH"
implies = ["HARD"]

[[archetype.tags]]
id = "AIR"

[[archetype]]
id = "material"
name = "Material"
exclusivity = "mutual"

[[archetype.tags]]
id = "WOOD"
implies = ["FLAMMABLE", "POROUS", "FLOATS"]

[[archetype.tags]]
id = "STONE"
implies = ["HARD", "HEAVY"]

[[archetype.tags]]
id = "METAL"
implies = ["CONDUCTIVE", "HARD"]

[[archetype.tags]]
id = "GLASS"
implies = ["FRAGILE", "HEAT_RESISTANT"]

[[archetype]]
id = "temperature"
name = "Temperature"
exclusivity = "mutual"

[[archetype.tags]]
id = "FREEZING"

[[archetype.tags]]
id = "COLD"

[[archetype.tags]]
id = "HOT"

[[archetype.tags]]
id = "NEUTRAL"

[[archetype]]
id = "moisture"
name = "Moisture"
exclusivity = "mutual"

[[archetype.tags]]
id = "DRY"

[[archetype.tags]]
id = "WET"

[[archetype.tags]]
id = "SOAKED"

[[archetype]]
id = "status"
name = "Status"
exclusivity = "any"

[[archetype.tags]]
id = "BURNING"
ticks = [5, 15]

[[archetype.tags]]
id = "STUNNED"
ticks = [1, 3]

[[archetype.tags]]
id = "FROZEN"
ticks = [3, 8]

[[archetype.tags]]
id = "POISONED"
ticks = [10, 30]

[[archetype.tags]]
id = "REGENERATING"
ticks = [5, 15]

[[archetype]]
id = "interaction"
name = "Interaction"
exclusivity = "any"

[[archetype.tags]]
id = "FLAMMABLE"
default_magnitude = 0.7

[[archetype.tags]]
id = "FIREPROOF"

[[archetype.tags]]
id = "CONDUCTIVE"
default_magnitude = 0.8

[[archetype.tags]]
id = "INSULATING"

[[archetype.tags]]
id = "HARD"
default_magnitude = 0.8

[[archetype.tags]]
id = "LUMINESCENT"
default_magnitude = 0.5

[[archetype.tags]]
id = "TOXIC"
default_magnitude = 0.5

[[archetype.tags]]
id = "POROUS"
default_magnitude = 0.5

[[archetype.tags]]
id = "FLOATS"

[[archetype.tags]]
id = "HEAVY"

[[archetype.tags]]
id = "FRAGILE"
default_magnitude = 0.3

[[archetype.tags]]
id = "HEAT_RESISTANT"

[[archetype.tags]]
id = "SLIPPERY"

[[archetype.tags]]
id = "SPREADS"

[[archetype]]
id = "state"
name = "Physical State"
exclusivity = "mutual"

[[archetype.tags]]
id = "SOLID"

[[archetype.tags]]
id = "LIQUID"
implies = ["SPREADS", "WET"]

[[archetype]]
id = "biome"
name = "Biome"
exclusivity = "mutual"

[[archetype.tags]]
id = "BIOME_DESERT"
implies = ["HOT", "DRY"]

[[archetype.tags]]
id = "BIOME_FOREST"
implies = ["FLAMMABLE"]

[[archetype.tags]]
id = "BIOME_SWAMP"
implies = ["WET", "SOAKED"]

[[archetype]]
id = "terrain"
name = "Terrain"
exclusivity = "any"

[[archetype.tags]]
id = "WALKABLE"

[[archetype.tags]]
id = "BLOCKED"

[[archetype]]
id = "creature_type"
name = "Creature Type"
exclusivity = "mutual"

[[archetype.tags]]
id = "UNDEAD"

[[archetype.tags]]
id = "CONSTRUCT"

[[archetype.tags]]
id = "PLANT"

[[archetype.tags]]
id = "ELEMENTAL"

[[archetype]]
id = "magic"
name = "Magic"
exclusivity = "any"

[[archetype.tags]]
id = "BLESSED"

[[archetype.tags]]
id = "ARCANE"

[[archetype.tags]]
id = "NECROTIC"

[[archetype]]
id = "damage_type"
name = "Damage Type"
exclusivity = "any"

[[archetype.tags]]
id = "HOLY_DAMAGE"

[[archetype.tags]]
id = "NECROTIC_DAMAGE"
"#;

const TEST_INTERACTIONS_TOML: &str = r#"
[[interaction]]
tag_a = "FIRE"
tag_b = "FLAMMABLE"
produces = ["BURNING"]
consumes = []
priority = 10

[[interaction]]
tag_a = "WATER"
tag_b = "FIRE"
produces = []
consumes = ["FIRE"]
priority = 20

[[interaction]]
tag_a = "ICE"
tag_b = "FIRE"
produces = ["WATER"]
consumes = ["ICE"]
priority = 12

[[interaction]]
tag_a = "UNDEAD"
tag_b = "BLESSED"
produces = ["HOLY_DAMAGE"]
consumes = ["BLESSED"]
priority = 10

[[interaction]]
tag_a = "CONSTRUCT"
tag_b = "POISONED"
produces = []
consumes = ["POISONED"]
priority = 20

[[interaction]]
tag_a = "FIRE"
tag_b = "FIREPROOF"
produces = []
consumes = []
priority = 25
"#;

fn setup() -> (TagRegistry, InteractionRules) {
    let registry = load_tag_registry(TEST_TAGS_TOML).unwrap();
    let rules = load_interaction_rules(TEST_INTERACTIONS_TOML, &registry).unwrap();
    (registry, rules)
}

#[test]
fn acceptance_registry_loads_from_toml() {
    let registry = load_tag_registry(TEST_TAGS_TOML).unwrap();
    assert!(registry.tag_count() > 0);
    assert!(registry.tag_by_name("FIRE").is_some());
    assert!(registry.tag_by_name("WATER").is_some());
    assert!(registry.tag_by_name("BURNING").is_some());
    assert!(registry.tag_by_name("BIOME_DESERT").is_some());
}

#[test]
fn acceptance_tags_component_add_remove_query() {
    let (registry, _) = setup();
    let mut tags = Tags::new(registry.tag_count());

    let fire_id = registry.tag_id("FIRE").unwrap();
    tags.add_tag(fire_id, TagValue::None, &registry);

    assert!(tags.has(fire_id));
    assert!(tags.count() >= 1);

    tags.remove_tag(fire_id, &registry);
    assert!(!tags.has(fire_id));
}

#[test]
fn acceptance_implication_chains() {
    let (registry, _) = setup();
    let mut tags = Tags::new(registry.tag_count());

    let fire_id = registry.tag_id("FIRE").unwrap();
    let hot_id = registry.tag_id("HOT").unwrap();
    let lum_id = registry.tag_id("LUMINESCENT").unwrap();

    tags.add_tag(fire_id, TagValue::None, &registry);

    assert!(tags.has(fire_id));
    assert!(tags.has(hot_id));
    assert!(tags.has(lum_id));
}

#[test]
fn acceptance_mutual_exclusivity() {
    let (registry, _) = setup();
    let mut tags = Tags::new(registry.tag_count());

    let fire_id = registry.tag_id("FIRE").unwrap();
    let water_id = registry.tag_id("WATER").unwrap();

    tags.add_tag(fire_id, TagValue::None, &registry);
    assert!(tags.has(fire_id));

    tags.add_tag(water_id, TagValue::None, &registry);
    assert!(!tags.has(fire_id));
    assert!(tags.has(water_id));
}

#[test]
fn acceptance_material_implies_interaction() {
    let (registry, _) = setup();
    let mut tags = Tags::new(registry.tag_count());

    let wood_id = registry.tag_id("WOOD").unwrap();
    let flammable_id = registry.tag_id("FLAMMABLE").unwrap();
    let porous_id = registry.tag_id("POROUS").unwrap();
    let floats_id = registry.tag_id("FLOATS").unwrap();

    tags.add_tag(wood_id, TagValue::None, &registry);

    assert!(tags.has(wood_id));
    assert!(tags.has(flammable_id));
    assert!(tags.has(porous_id));
    assert!(tags.has(floats_id));
}

#[test]
fn acceptance_interaction_rules_fire_flammable() {
    let (registry, rules) = setup();

    let mut tags = Tags::new(registry.tag_count());
    let fire_id = registry.tag_id("FIRE").unwrap();
    let flammable_id = registry.tag_id("FLAMMABLE").unwrap();
    tags.add_tag(fire_id, TagValue::None, &registry);
    tags.add_tag(flammable_id, TagValue::Magnitude(0.7), &registry);

    let matched = rules.check_self_interactions(&tags);
    assert!(!matched.is_empty());

    let burning_id = registry.tag_id("BURNING").unwrap();
    let matched_rule = matched.iter().find(|r| r.produces.contains(&burning_id));
    assert!(matched_rule.is_some());
}

#[test]
fn acceptance_interaction_water_extinguishes_fire() {
    let (registry, rules) = setup();

    let water_id = registry.tag_id("WATER").unwrap();
    let fire_id = registry.tag_id("FIRE").unwrap();

    let mut tags_a = Tags::new(registry.tag_count());
    let mut tags_b = Tags::new(registry.tag_count());
    tags_a.add_tag(water_id, TagValue::None, &registry);
    tags_b.add_tag(fire_id, TagValue::None, &registry);

    let matched = rules.check_cross_interactions(&tags_a, &tags_b);
    let consume_fire = matched.iter().find(|(r, _)| r.consumes.contains(&fire_id));
    assert!(consume_fire.is_some());
}

#[test]
fn acceptance_temporary_tags_tick_down() {
    let (registry, _) = setup();
    let mut tags = Tags::new(registry.tag_count());

    let burning_id = registry.tag_id("BURNING").unwrap();
    tags.add_tag(
        burning_id,
        TagValue::Ticks { remaining: 3, max: 5 },
        &registry,
    );

    assert!(tags.has(burning_id));

    tags.tick_status(&registry);
    assert!(tags.has(burning_id));

    tags.tick_status(&registry);
    assert!(tags.has(burning_id));

    tags.tick_status(&registry);
    assert!(!tags.has(burning_id));
}

#[test]
fn acceptance_biome_implies_terrain() {
    let (registry, _) = setup();
    let mut tags = Tags::new(registry.tag_count());

    let desert_id = registry.tag_id("BIOME_DESERT").unwrap();
    let hot_id = registry.tag_id("HOT").unwrap();
    let dry_id = registry.tag_id("DRY").unwrap();

    tags.add_tag(desert_id, TagValue::None, &registry);

    assert!(tags.has(desert_id));
    assert!(tags.has(hot_id));
    assert!(tags.has(dry_id));
}

#[test]
fn acceptance_cross_interactions() {
    let (registry, rules) = setup();

    let mut tags_a = Tags::new(registry.tag_count());
    let mut tags_b = Tags::new(registry.tag_count());

    let fire_id = registry.tag_id("FIRE").unwrap();
    let flammable_id = registry.tag_id("FLAMMABLE").unwrap();
    tags_a.add_tag(fire_id, TagValue::None, &registry);
    tags_b.add_tag(flammable_id, TagValue::Magnitude(0.7), &registry);

    let matched = rules.check_cross_interactions(&tags_a, &tags_b);
    assert!(!matched.is_empty());
}

#[test]
fn acceptance_priority_ordering() {
    let (registry, rules) = setup();

    let fire_id = registry.tag_id("FIRE").unwrap();
    let fireproof_id = registry.tag_id("FIREPROOF").unwrap();

    let mut tags = Tags::new(registry.tag_count());
    tags.add_tag(fire_id, TagValue::None, &registry);
    tags.add_tag(fireproof_id, TagValue::None, &registry);

    let matched = rules.check_self_interactions(&tags);
    let fireproof_rule = matched
        .iter()
        .find(|r| r.priority == 25 && r.tag_a == fire_id && r.tag_b == fireproof_id);
    assert!(fireproof_rule.is_some());
}

#[test]
fn acceptance_has_all_has_any() {
    let (registry, _) = setup();
    let mut tags = Tags::new(registry.tag_count());

    let fire_id = registry.tag_id("FIRE").unwrap();
    let hot_id = registry.tag_id("HOT").unwrap();
    let water_id = registry.tag_id("WATER").unwrap();

    tags.add_tag(fire_id, TagValue::None, &registry);

    assert!(tags.has_all(&[fire_id, hot_id]));
    assert!(!tags.has_all(&[fire_id, water_id]));
    assert!(tags.has_any(&[fire_id, water_id]));
}

#[test]
fn acceptance_transitive_implications() {
    let (registry, _) = setup();
    let mut tags = Tags::new(registry.tag_count());

    let liquid_id = registry.tag_id("LIQUID").unwrap();
    let spreads_id = registry.tag_id("SPREADS").unwrap();
    let wet_id = registry.tag_id("WET").unwrap();

    tags.add_tag(liquid_id, TagValue::None, &registry);

    assert!(tags.has(liquid_id));
    assert!(tags.has(spreads_id));
    assert!(tags.has(wet_id));
}

#[test]
fn acceptance_removing_tag_does_not_remove_implications() {
    let (registry, _) = setup();
    let mut tags = Tags::new(registry.tag_count());

    let fire_id = registry.tag_id("FIRE").unwrap();
    let hot_id = registry.tag_id("HOT").unwrap();

    tags.add_tag(fire_id, TagValue::None, &registry);
    assert!(tags.has(hot_id));

    tags.remove_tag(fire_id, &registry);
    assert!(!tags.has(fire_id));
    assert!(tags.has(hot_id));
}
