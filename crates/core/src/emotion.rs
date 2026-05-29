use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Emotion {
    Neutral,
    Happy,
    Angry,
    Fearful,
    Sad,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationState {
    Idle,
    Greeting,
    Dialogue,
    Trading,
    Fighting,
    Goodbye,
}

impl Default for ConversationState {
    fn default() -> Self { Self::Idle }
}

#[derive(Component, Debug, Clone)]
pub struct NpcEmotionalState {
    pub stress: f32,
    pub dominant_emotion: Emotion,
    pub conversation: ConversationState,
}

impl Default for NpcEmotionalState {
    fn default() -> Self {
        Self { stress: 0.0, dominant_emotion: Emotion::Neutral, conversation: ConversationState::Idle }
    }
}

impl NpcEmotionalState {
    /// Apply an emotional impact (positive = bad/stressful, negative = good)
    pub fn apply_event(&mut self, emotional_impact: f32) {
        self.stress = (self.stress + emotional_impact).clamp(0.0, 1.0);
        self.dominant_emotion = if self.stress > 0.7 { Emotion::Fearful }
            else if emotional_impact > 0.3 { Emotion::Angry }
            else if emotional_impact < -0.2 { Emotion::Happy }
            else { Emotion::Neutral };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_stress_is_neutral() {
        let state = NpcEmotionalState::default();
        assert_eq!(state.dominant_emotion, Emotion::Neutral);
    }

    #[test]
    fn high_impact_triggers_anger() {
        let mut state = NpcEmotionalState::default();
        state.apply_event(0.5);
        assert_eq!(state.dominant_emotion, Emotion::Angry);
    }

    #[test]
    fn stress_accumulates() {
        let mut state = NpcEmotionalState::default();
        state.apply_event(0.4);
        state.apply_event(0.4);
        assert_eq!(state.dominant_emotion, Emotion::Fearful, "stress > 0.7 should be fearful");
    }

    #[test]
    fn negative_impact_is_happy() {
        let mut state = NpcEmotionalState::default();
        state.apply_event(-0.3);
        assert_eq!(state.dominant_emotion, Emotion::Happy);
    }
}
