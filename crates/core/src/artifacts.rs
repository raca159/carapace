use bevy_ecs::prelude::*;

use crate::components::MessageLog;
use crate::turn::TurnCounter;
use crate::{Glyph, Name, Position};
use game_tags::{TagRegistry, Tags};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactType {
    NanoForge,         // Instantly crafts any known recipe
    PhaseShifter,      // Teleports to a random revealed map position
    StasisField,       // Freezes all NPCs for 5 turns
    DataCache,         // Reveals all undiscovered map areas
    MedicalNanites,    // Fully heals the player
    PlasmaGrenade,     // Deals 50 damage to all enemies in radius
    GravityAnchor,     // Returns the player to the overworld entrance
}

impl ArtifactType {
    pub fn all() -> &'static [ArtifactType] {
        &[
            ArtifactType::NanoForge,
            ArtifactType::PhaseShifter,
            ArtifactType::StasisField,
            ArtifactType::DataCache,
            ArtifactType::MedicalNanites,
            ArtifactType::PlasmaGrenade,
            ArtifactType::GravityAnchor,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ArtifactType::NanoForge => "NanoForge",
            ArtifactType::PhaseShifter => "Phase Shifter",
            ArtifactType::StasisField => "Stasis Field",
            ArtifactType::DataCache => "Data Cache",
            ArtifactType::MedicalNanites => "Medical Nanites",
            ArtifactType::PlasmaGrenade => "Plasma Grenade",
            ArtifactType::GravityAnchor => "Gravity Anchor",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ArtifactType::NanoForge => "Instantly crafts any known recipe without materials.",
            ArtifactType::PhaseShifter => "Teleports you to a random revealed position.",
            ArtifactType::StasisField => "Freezes all nearby creatures for 5 turns.",
            ArtifactType::DataCache => "Reveals all undiscovered areas on the map.",
            ArtifactType::MedicalNanites => "Fully restores your health.",
            ArtifactType::PlasmaGrenade => "Deals massive damage to all enemies nearby.",
            ArtifactType::GravityAnchor => "Returns you to the overworld entrance.",
        }
    }

    pub fn glyph(&self) -> char {
        match self {
            ArtifactType::NanoForge => '⚙',
            ArtifactType::PhaseShifter => '⟐',
            ArtifactType::StasisField => '❄',
            ArtifactType::DataCache => '☐',
            ArtifactType::MedicalNanites => '✦',
            ArtifactType::PlasmaGrenade => '※',
            ArtifactType::GravityAnchor => '⏚',
        }
    }

    pub fn max_charges(&self) -> u32 {
        match self {
            ArtifactType::NanoForge => 3,
            ArtifactType::PhaseShifter => 3,
            ArtifactType::StasisField => 2,
            ArtifactType::DataCache => 2,
            ArtifactType::MedicalNanites => 1,
            ArtifactType::PlasmaGrenade => 3,
            ArtifactType::GravityAnchor => 2,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Artifact {
    pub artifact_type: ArtifactType,
    pub charges: u32,
    pub max_charges: u32,
}

impl Artifact {
    pub fn new(artifact_type: ArtifactType) -> Self {
        Self { artifact_type, charges: artifact_type.max_charges(), max_charges: artifact_type.max_charges() }
    }

    pub fn use_charge(&mut self, world: &mut World) -> bool {
        if self.charges == 0 { return false; }
        self.charges -= 1;

        let _turn = world.get_resource::<TurnCounter>().map(|tc| tc.0).unwrap_or(0);
        let name = self.artifact_type.name();
        let msg = if self.charges == 0 {
            format!("{} activated. No charges remaining.", name)
        } else {
            format!("{} activated. ({} charge{} remaining)", name, self.charges, if self.charges == 1 { "" } else { "s" })
        };
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.push(msg);
        }
        self.charges == 0
    }

    pub fn is_depleted(&self) -> bool {
        self.charges == 0
    }
}

pub fn spawn_artifact(world: &mut World, pos: Position, artifact_type: ArtifactType) -> bevy_ecs::entity::Entity {
    let tags = {
        let mut t = Tags::new(64);
        if let Some(registry) = world.get_resource::<TagRegistry>() {
            if let Some(tid) = registry.tag_id("ARTIFACT") { t.add_tag(tid, game_tags::TagValue::None, registry); }
            if let Some(tid) = registry.tag_id("VALUABLE") { t.add_tag(tid, game_tags::TagValue::None, registry); }
        }
        t
    };

    world.spawn((
        crate::Item,
        Name(artifact_type.name().to_string()),
        Glyph { char: artifact_type.glyph(), color: (255, 215, 0) },
        pos,
        Artifact::new(artifact_type),
        tags,
    )).id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Health, Player};

    fn test_world() -> World {
        let mut w = World::new();
        w.insert_resource(TurnCounter(1));
        w.insert_resource(MessageLog::new(10));
        w.insert_resource(game_tags::TagRegistryBuilder::default().build().unwrap());
        w.spawn((Player, Position { z: 0, x: 0, y: 0 }, Health { current: 50, max: 100 }));
        w
    }

    #[test]
    fn all_seven_artifacts_have_names() {
        for a in ArtifactType::all() {
            assert!(!a.name().is_empty());
            assert!(!a.description().is_empty());
        }
    }

    #[test]
    fn all_artifacts_have_charges() {
        for a in ArtifactType::all() {
            assert!(a.max_charges() > 0);
        }
    }

    #[test]
    fn nano_forge_has_three_charges() {
        let af = ArtifactType::NanoForge;
        assert_eq!(af.max_charges(), 3);
    }

    #[test]
    fn artifact_use_charge_reduces() {
        let mut world = test_world();
        let mut artifact = Artifact::new(ArtifactType::PhaseShifter);
        assert_eq!(artifact.charges, 3);
        artifact.use_charge(&mut world);
        assert_eq!(artifact.charges, 2);
        artifact.use_charge(&mut world);
        assert_eq!(artifact.charges, 1);
    }

    #[test]
    fn artifact_is_depleted_when_zero() {
        let mut world = test_world();
        let mut artifact = Artifact::new(ArtifactType::MedicalNanites);
        assert_eq!(artifact.max_charges, 1);
        assert!(!artifact.is_depleted());
        artifact.use_charge(&mut world);
        assert!(artifact.is_depleted());
    }

    #[test]
    fn cannot_use_depleted_artifact() {
        let mut world = test_world();
        let mut artifact = Artifact::new(ArtifactType::MedicalNanites);
        artifact.use_charge(&mut world);
        assert!(!artifact.use_charge(&mut world));
    }

    #[test]
    fn spawn_artifact_creates_entity() {
        let mut world = test_world();
        let entity = spawn_artifact(&mut world, Position { z: 0, x: 5, y: 5 }, ArtifactType::NanoForge);
        assert!(world.get::<Artifact>(entity).is_some());
        assert_eq!(world.get::<Artifact>(entity).unwrap().charges, 3);
    }
}
