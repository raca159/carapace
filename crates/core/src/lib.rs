pub mod barter;
pub mod calc;
pub mod camera;
pub mod color_theme;
pub mod components;
pub mod crafting;
pub mod dialogue;
pub mod encounters;
pub mod durability;
pub mod emotion;
pub mod environment;
pub mod events;
pub mod examine;
pub mod narrative;
pub mod npc_action;
pub mod npc_personality;
pub mod personality;
pub mod quest;
pub mod reputation;
pub mod save;
pub mod screen;
// pub mod snapshot;  // Depends on deleted GameState — restored in Phase 4
pub mod stats;
pub mod traps;
pub mod turn;
pub mod interaction_test;
pub mod weather;
pub mod weather_def;
pub mod world_overview;

pub use barter::{BarterItem, BarterOffer, BarterResult, resolve_barter};
pub use weather::{TimeOfDay, WeatherState, WeatherContext, weather_tags_for_context};
pub use calc::{calc_armor_protection, calc_weapon_damage, get_quality_multiplier, get_quality_prefix};
pub use camera::Camera;
pub use color_theme::ColorTheme;
pub use components::{
    ArmorProtection, Creature, Equipment, EquipmentSlot, Glyph, Health, Inventory, Item,
    ItemEffects, MessageLog, Name, OverworldEntity, Player, Position, WeaponDamage, WeatherSensitive,
};
pub use crafting::{CraftingRecipe, RecipeAvailability, load_crafting_recipes, find_available_recipes, execute_recipe};
pub use dialogue::{DialogueLine, DialogueLinesResource, load_dialogue, select_dialogue};
pub use encounters::{Encounters, EncounterDef, roll_encounter, spawn_encounter, load_encounters};
pub use environment::EnvironmentalScores;
pub use durability::Durability;
pub use emotion::{Emotion, ConversationState, NpcEmotionalState};
pub use events::{EventBus, GameEvent, GameEventLog, InteractionKind, format_event, push_event};
pub use examine::ExamineMode;
pub use narrative::{LoreFragment, LoreFragmentsResource, NarrativeCooldowns, NarrativeEvent, NarrativeEvents, load_narrative_events, load_lore_fragments, check_narrative_events};
pub use npc_action::{NpcAction, NpcContext, EntityContext, EnvironmentContext, ActionWeightDef, NpcActionWeights, load_npc_action_weights, score_action};
pub use npc_personality::{NpcPersonality, NpcPersonalitiesResource, load_npc_personalities};
pub use personality::{PersonalityScores, tags_from_personality};
pub use quest::{ActiveQuest, QuestBoard, QuestBoardEntry, QuestBoardState, QuestGiver, QuestLog, QuestObjective, QuestState, QuestTemplate, QuestTemplates, load_quest_templates, generate_quests, generate_board_quests, accept_board_quest, check_quest_completion, check_quest_failures, check_quest_board_refresh, check_player_near_quest_board, turn_in_quest, track_kill, track_kill_area, track_collect, track_reach, handle_quest_turn_in};
pub use reputation::{FactionReputation, ReputationRank};
pub use save::{SaveGame, save_game, load_game, list_saves, should_auto_save};
pub use stats::PlayerStats;
pub use screen::{AppScreen, register_app_screen_state, request_screen_transition, get_current_screen, run_screen_transitions};
pub use traps::{Trap, TrapType, TrappedStatus, process_trapped_status, trigger_trap, try_detect_trap, try_disarm_trap};
pub use turn::{BehaviorState, TurnCounter, TurnPhase, TurnState};
