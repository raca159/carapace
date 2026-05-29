use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("duplicate tag name: {0}")]
    DuplicateTagName(String),
    #[error("duplicate archetype name: {0}")]
    DuplicateArchetypeName(String),
    #[error("unresolved implies reference: tag '{tag}' implies '{implies}', but '{implies}' is not registered")]
    UnresolvedImplies { tag: String, implies: String },
    #[error("unresolved conflicts reference: tag '{tag}' conflicts with '{conflict}', but '{conflict}' is not registered")]
    UnresolvedConflict { tag: String, conflict: String },
    #[error("self-implication: tag '{0}' implies itself")]
    SelfImplication(String),
}

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("registry error: {0}")]
    Registry(#[from] RegistryError),
    #[error("unknown exclusivity: {0}")]
    UnknownExclusivity(String),
    #[error("unknown tag: {0}")]
    UnknownTag(String),
    #[error("interaction rule references unknown tag: {0}")]
    InteractionUnknownTag(String),
    #[error("empty archetype: {0} has no tags")]
    EmptyArchetype(String),
}
