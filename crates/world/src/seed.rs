use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bevy_ecs::prelude::Resource)]
pub struct WorldSeed(pub u64);

impl WorldSeed {
    pub fn random() -> Self {
        Self(rand::random())
    }

    pub fn from_value(v: u64) -> Self {
        Self(v)
    }

    pub fn from_string(s: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_from_value() {
        let seed = WorldSeed::from_value(42);
        assert_eq!(seed.0, 42);
    }

    #[test]
    fn seed_from_string_deterministic() {
        let s1 = WorldSeed::from_string("hello");
        let s2 = WorldSeed::from_string("hello");
        assert_eq!(s1, s2);
    }

    #[test]
    fn seed_from_string_different() {
        let s1 = WorldSeed::from_string("hello");
        let s2 = WorldSeed::from_string("world");
        assert_ne!(s1, s2);
    }
}
