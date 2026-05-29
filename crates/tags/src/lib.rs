pub mod component;
pub mod definition;
pub mod error;
pub mod id;
pub mod interaction;
pub mod loader;
pub mod query;
pub mod registry;
pub mod serialization;

pub use component::{TagValue, Tags};
pub use error::{LoadError, RegistryError};
pub use id::{ArchetypeId, TagId};
pub use interaction::{InteractionRule, InteractionRules};
pub use loader::{load_interaction_rules, load_tag_registry};
pub use registry::{
    ArchetypeDef, Exclusivity, TagDef, TagRegistry, TagRegistryBuilder,
};
pub use query::TagQuery;
pub use serialization::{snapshot_to_tags, tags_to_snapshot, TagValueSnapshot, TagsSnapshot};
