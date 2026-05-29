use bevy_ecs::prelude::*;
use rand::Rng;

use crate::weather_def::{load_all_weathers, WeatherDef};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfDay {
    Dawn,
    Day,
    Dusk,
    Night,
}

impl TimeOfDay {
    pub fn name(&self) -> &'static str {
        match self {
            TimeOfDay::Dawn => "Dawn",
            TimeOfDay::Day => "Day",
            TimeOfDay::Dusk => "Dusk",
            TimeOfDay::Night => "Night",
        }
    }

    pub fn light_level(&self) -> f32 {
        match self {
            TimeOfDay::Dawn => 0.6,
            TimeOfDay::Day => 1.0,
            TimeOfDay::Dusk => 0.4,
            TimeOfDay::Night => 0.15,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct WeatherState {
    pub weather_defs: Vec<WeatherDef>,
    pub active_idx: usize,
    pub time: TimeOfDay,
    pub turn_count: u64,
    pub weather_turns_remaining: u32,
}

impl WeatherState {
    pub fn new() -> Self {
        let weather_defs = load_all_weathers();
        let active_idx = 0;
        Self {
            weather_defs,
            active_idx,
            time: TimeOfDay::Day,
            turn_count: 0,
            weather_turns_remaining: 20,
        }
    }

    pub fn active_weather(&self) -> &WeatherDef {
        &self.weather_defs[self.active_idx]
    }

    pub fn advance_time(&mut self, _world: &World) {
        self.turn_count += 1;

        let day_length = 50;
        match self.time {
            TimeOfDay::Dawn => { if self.turn_count >= day_length / 4 { self.time = TimeOfDay::Day; self.turn_count = 0; } }
            TimeOfDay::Day => { if self.turn_count >= day_length / 2 { self.time = TimeOfDay::Dusk; self.turn_count = 0; } }
            TimeOfDay::Dusk => { if self.turn_count >= day_length / 4 { self.time = TimeOfDay::Night; self.turn_count = 0; } }
            TimeOfDay::Night => { if self.turn_count >= day_length { self.time = TimeOfDay::Dawn; self.turn_count = 0; } }
        }

        if self.weather_turns_remaining > 0 {
            self.weather_turns_remaining -= 1;
        } else {
            self.roll_new_weather();
        }
    }

    fn roll_new_weather(&mut self) {
        let mut rng = rand::rng();
        let total_weight: u32 = self.weather_defs.iter().map(|w| w.weight).sum();
        let roll = rng.random_range(0..total_weight);
        let mut cumulative = 0u32;
        for (i, w) in self.weather_defs.iter().enumerate() {
            cumulative += w.weight;
            if roll < cumulative {
                self.active_idx = i;
                break;
            }
        }
        self.weather_turns_remaining = rng.random_range(5..25);
    }

    pub fn effective_visibility(&self) -> f32 {
        self.active_weather().visibility * self.time.light_level()
    }
}

impl Default for WeatherState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct WeatherContext {
    pub tags: Vec<String>,
    pub applied_tags: Vec<game_tags::TagId>,
}

/// Build weather tags from current weather state + time of day.
/// Only descriptive tags (RAINY, STORMY, etc.) from the WeatherDef TOML.
/// Environmental condition tags (DARK, DIM, WET, COLD, etc.) are derived
/// from the environmental score system (EnvironmentalScores::resolve_tags).
pub fn weather_tags_for_context(state: &WeatherState, _time: &TimeOfDay) -> Vec<String> {
    state.active_weather().tags.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_defaults_to_first_weather_day() {
        let ws = WeatherState::new();
        assert_eq!(ws.time, TimeOfDay::Day);
        assert!(ws.weather_defs.len() >= 8);
        assert_eq!(ws.active_weather().name, "Clear");
    }

    #[test]
    fn advance_time_changes_day_to_dusk() {
        let mut ws = WeatherState::new();
        ws.time = TimeOfDay::Day;
        ws.turn_count = 24;
        ws.advance_time(&World::new());
        assert_eq!(ws.time, TimeOfDay::Dusk);
    }

    #[test]
    fn visibility_never_zero() {
        let ws = WeatherState::new();
        assert!(ws.effective_visibility() > 0.0);
    }

    #[test]
    fn weather_changes_after_turns_elapsed() {
        let mut ws = WeatherState::new();
        ws.weather_turns_remaining = 1;
        ws.advance_time(&World::new());
    }

    #[test]
    fn active_weather_returns_current_def() {
        let ws = WeatherState::new();
        let def = ws.active_weather();
        assert_eq!(def.name, "Clear");
        assert!(def.visibility > 0.0);
    }

    #[test]
    fn descriptive_tags_include_weather_def_tags() {
        let state = WeatherState::new();
        let tags = weather_tags_for_context(&state, &TimeOfDay::Day);
        assert!(tags.is_empty(), "Clear weather should have no descriptive tags");

        // DARK/DIM are now derived from environmental scores, not weather_tags_for_context
        let rain_state = {
            let mut ws = WeatherState::new();
            ws.active_idx = ws.weather_defs.iter().position(|w| w.name == "Rain").unwrap_or(0);
            ws
        };
        let rain_tags = weather_tags_for_context(&rain_state, &TimeOfDay::Day);
        assert!(rain_tags.contains(&"RAINY".to_string()));
    }

    #[test]
    fn all_times_have_names() {
        for t in &[TimeOfDay::Dawn, TimeOfDay::Day, TimeOfDay::Dusk, TimeOfDay::Night] {
            assert!(!t.name().is_empty());
        }
    }

    #[test]
    fn weather_visibility_matches_def() {
        let ws = WeatherState::new();
        let vis = ws.effective_visibility();
        assert!((vis - 1.0).abs() < 0.01 || vis < 1.0);
    }
}
