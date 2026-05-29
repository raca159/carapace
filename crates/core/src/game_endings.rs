use bevy_ecs::prelude::*;

use crate::turn::TurnCounter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEnding {
    None,
    Ascension,
    Transformation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndingState {
    NotTriggered,
    InProgress,
    Completed(GameEnding),
}

#[derive(Resource, Debug, Clone)]
pub struct EndingTracker {
    pub state: EndingState,
    pub ascension_progress: u32,     // 0..5 artifacts needed
    pub transformation_turns: u64,   // how long since transformation started
}

impl Default for EndingTracker {
    fn default() -> Self {
        Self { state: EndingState::NotTriggered, ascension_progress: 0, transformation_turns: 0 }
    }
}

impl EndingTracker {
    pub fn new() -> Self { Self::default() }

    pub fn is_active(&self) -> bool { self.state == EndingState::InProgress }

    pub fn is_complete(&self) -> bool {
        matches!(self.state, EndingState::Completed(_))
    }

    pub fn collect_artifact(&mut self) -> Option<GameEnding> {
        self.ascension_progress += 1;
        if self.ascension_progress >= 5 {
            self.state = EndingState::Completed(GameEnding::Ascension);
            return Some(GameEnding::Ascension);
        }
        None
    }

    pub fn start_transformation(&mut self) {
        self.state = EndingState::InProgress;
        self.transformation_turns = 0;
    }

    pub fn tick_transformation(&mut self, world: &World) -> Option<GameEnding> {
        if self.state != EndingState::InProgress { return None; }
        let turn = world.get_resource::<TurnCounter>().map(|tc| tc.0).unwrap_or(0);
        if turn >= self.transformation_turns + 20 {
            self.state = EndingState::Completed(GameEnding::Transformation);
            return Some(GameEnding::Transformation);
        }
        None
    }

    pub fn ending_text(&self) -> &'static str {
        match self.state {
            EndingState::Completed(GameEnding::Ascension) => {
                "You have gathered all five ancient artifacts. \
                 The ascension mechanism hums to life. \
                 A pillar of light envelops you as you transcend \
                 beyond the physical realm. You have won."
            }
            EndingState::Completed(GameEnding::Transformation) => {
                "The corruption has spread too far. \
                 Your body contorts, your mind fragments. \
                 You are no longer human — you have become \
                 part of the world itself. Forever changed."
            }
            EndingState::Completed(GameEnding::None) => unreachable!(),
            EndingState::InProgress => {
                "The transformation is underway. \
                 You feel yourself changing..."
            }
            EndingState::NotTriggered => {
                ""
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracker_starts_not_triggered() {
        let t = EndingTracker::new();
        assert!(!t.is_active());
        assert!(!t.is_complete());
    }

    #[test]
    fn ascension_requires_five_artifacts() {
        let mut t = EndingTracker::new();
        for i in 0..5 {
            let r = t.collect_artifact();
            if i < 4 { assert!(r.is_none()); }
            else { assert_eq!(r, Some(GameEnding::Ascension)); }
        }
    }

    #[test]
    fn transformation_takes_20_turns() {
        let mut world = World::new();
        world.insert_resource(TurnCounter(0));
        let mut t = EndingTracker::new();
        t.start_transformation();
        assert!(t.is_active());

        world.insert_resource(TurnCounter(10));
        assert!(t.tick_transformation(&world).is_none());

        world.insert_resource(TurnCounter(20));
        assert_eq!(t.tick_transformation(&world), Some(GameEnding::Transformation));
    }

    #[test]
    fn ending_text_not_empty_for_completed() {
        let mut t = EndingTracker::new();
        t.collect_artifact();
        t.collect_artifact();
        t.collect_artifact();
        t.collect_artifact();
        t.collect_artifact();
        assert!(!t.ending_text().is_empty());
    }
}
