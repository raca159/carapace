use bevy_ecs::prelude::*;

use crate::components::Position;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TurnCounter(pub u64);

impl TurnCounter {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn current(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TurnPhase {
    #[default]
    Player,
    Npcs,
    Processing,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TurnState {
    pub phase: TurnPhase,
}

impl TurnState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_phase(&mut self, phase: TurnPhase) {
        self.phase = phase;
    }

    pub fn is_player_turn(&self) -> bool {
        self.phase == TurnPhase::Player
    }

    pub fn is_npc_turn(&self) -> bool {
        self.phase == TurnPhase::Npcs
    }
}

#[derive(Component, Debug, Clone)]
pub struct BehaviorState {
    pub home_pos: Option<Position>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_counter_starts_at_zero() {
        let counter = TurnCounter::new();
        assert_eq!(counter.current(), 0);
        assert_eq!(counter.0, 0);
    }

    #[test]
    fn turn_counter_default_is_zero() {
        let counter = TurnCounter::default();
        assert_eq!(counter.current(), 0);
    }

    #[test]
    fn turn_counter_increments() {
        let mut counter = TurnCounter::new();
        assert_eq!(counter.current(), 0);
        counter.increment();
        assert_eq!(counter.current(), 1);
        counter.increment();
        assert_eq!(counter.current(), 2);
    }

    #[test]
    fn turn_state_defaults_to_player_phase() {
        let state = TurnState::new();
        assert_eq!(state.phase, TurnPhase::Player);
        assert!(state.is_player_turn());
        assert!(!state.is_npc_turn());
    }

    #[test]
    fn turn_state_phase_transitions() {
        let mut state = TurnState::new();
        assert_eq!(state.phase, TurnPhase::Player);

        state.set_phase(TurnPhase::Npcs);
        assert_eq!(state.phase, TurnPhase::Npcs);
        assert!(state.is_npc_turn());
        assert!(!state.is_player_turn());

        state.set_phase(TurnPhase::Processing);
        assert_eq!(state.phase, TurnPhase::Processing);
        assert!(!state.is_player_turn());
        assert!(!state.is_npc_turn());

        state.set_phase(TurnPhase::Player);
        assert_eq!(state.phase, TurnPhase::Player);
        assert!(state.is_player_turn());
    }

    #[test]
    fn behavior_state_with_home_pos() {
        let pos = Position { x: 5, y: 10, z: 0 };
        let state = BehaviorState {
            home_pos: Some(pos),
        };
        assert_eq!(state.home_pos.unwrap().x, 5);
        assert_eq!(state.home_pos.unwrap().y, 10);
    }

    #[test]
    fn behavior_state_without_home_pos() {
        let state = BehaviorState { home_pos: None };
        assert!(state.home_pos.is_none());
    }
}
