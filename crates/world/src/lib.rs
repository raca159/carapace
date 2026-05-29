pub mod behavior;
pub mod biome;
pub mod cascade;
pub mod debug;
pub mod pathfinding;
pub mod spatial;
pub mod wfc;
pub mod dungeon;
pub mod entity_gen;
pub mod export;
pub mod faction;
pub mod generator;
pub mod interior;
pub mod latitude;
pub mod loader;
pub mod loot;
pub mod map;
pub mod noise_gen;
pub mod seed;
pub mod spawner;
pub mod tile;

pub use cascade::CascadeEngine;
pub use behavior::{
    load_behavior_rules, process_npc_turns, BehaviorRule, BehaviorRules, NpcAction,
};
pub use biome::{BiomeClassifier, BiomeEnvironment, BiomeRule};
pub use debug::{biome_summary, render_map_ascii, render_map_plain};
pub use dungeon::{
    generate_dungeon, dungeon_spawn_positions, DungeonConfig, DungeonMap, DungeonTile,
    DungeonTileType, DungeonType, MapLayer, RoomRect,
};
pub use entity_gen::{generate_entity, load_entity_templates, EntityTemplate};
pub use faction::{
    load_factions, Faction, FactionDef, FactionId, FactionRelationships, FactionStanding,
    ReputationTracker, PLAYER_FACTION_ID, REP_ATTACK_PENALTY, REP_KILL_PENALTY, REP_QUEST_REWARD,
    REP_THRESHOLD_ALLY, REP_THRESHOLD_HOSTILE,
};
pub use generator::{generate_world, WorldConfig, WorldGenResources, WorldPlugin};
pub use loader::{load_biome_rules, load_world_config, load_world_config_str, WorldGenConfig};
pub use map::WorldMap;
pub use noise_gen::{NoiseGenerator, NoiseLayerConfig};
pub use seed::WorldSeed;
pub use spawner::{load_spawn_rules, spawn_entities, SpawnRule};
pub use tile::{Tile, TilePos};
pub use loot::{LootDropInstance, LootEntryDef, LootTableDef, LootTables, load_loot_tables, place_dungeon_chests, roll_loot_for_creature, roll_loot_for_table, spawn_dungeon_floor_loot, spawn_loot_drop};
pub use cascade::inventory_populate::{populate_inventories, PopulateContext};
pub use pathfinding::{a_star_step, has_line_of_sight};
