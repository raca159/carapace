use bevy_ecs::prelude::*;
use bevy_ecs::schedule::Schedules;
use bevy_ecs::event::Events;
use bevy_state::prelude::*;
use bevy_state::state::{setup_state_transitions_in_world, FreelyMutableState, StateTransitionEvent};

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppScreen {
    #[default]
    Boot,
    MainMenu,
    CreateWorld,
    WorldGenProgress,
    NewCharacter,
    WorldOverview,
    InWorld,
    PauseMenu,
    Dead,
}

impl AppScreen {
    pub fn is_gameplay(&self) -> bool {
        matches!(self, AppScreen::InWorld | AppScreen::Dead)
    }

    pub fn is_menu(&self) -> bool {
        matches!(
            self,
            AppScreen::MainMenu
                | AppScreen::CreateWorld
                | AppScreen::NewCharacter
                | AppScreen::WorldOverview
        )
    }

    pub fn transitions_to_gameplay(&self) -> bool {
        matches!(self, AppScreen::NewCharacter | AppScreen::WorldOverview)
    }

    pub fn transition_allowed(from: &AppScreen, to: &AppScreen) -> bool {
        matches!(
            (from, to),
            (AppScreen::Boot, AppScreen::MainMenu)
                | (AppScreen::MainMenu, AppScreen::CreateWorld)
                | (AppScreen::MainMenu, AppScreen::NewCharacter)
                | (AppScreen::MainMenu, AppScreen::WorldOverview)
                | (AppScreen::CreateWorld, AppScreen::WorldGenProgress)
                | (AppScreen::CreateWorld, AppScreen::MainMenu)
                | (AppScreen::WorldGenProgress, AppScreen::NewCharacter)
                | (AppScreen::WorldGenProgress, AppScreen::CreateWorld)
                | (AppScreen::NewCharacter, AppScreen::WorldOverview)
                | (AppScreen::NewCharacter, AppScreen::InWorld)
                | (AppScreen::NewCharacter, AppScreen::MainMenu)
                | (AppScreen::NewCharacter, AppScreen::CreateWorld)
                | (AppScreen::WorldOverview, AppScreen::NewCharacter)
                | (AppScreen::WorldOverview, AppScreen::InWorld)
                | (AppScreen::WorldOverview, AppScreen::MainMenu)
                | (AppScreen::InWorld, AppScreen::PauseMenu)
                | (AppScreen::InWorld, AppScreen::Dead)
                | (AppScreen::InWorld, AppScreen::MainMenu)
                | (AppScreen::PauseMenu, AppScreen::InWorld)
                | (AppScreen::Dead, AppScreen::MainMenu)
        )
    }
}

pub fn register_app_screen_state(world: &mut World) {
    setup_state_transitions_in_world(world);
    world.insert_resource(Events::<StateTransitionEvent<AppScreen>>::default());

    let mut schedules = world.resource_mut::<Schedules>();
    let transition_schedule = schedules
        .get_mut(StateTransition)
        .expect("StateTransition schedule should exist after setup");

    AppScreen::register_state(transition_schedule);

    world.init_resource::<State<AppScreen>>();
}

pub fn request_screen_transition(world: &mut World, target: AppScreen) {
    world.insert_resource(NextState::Pending(target));
}

pub fn get_current_screen(world: &World) -> AppScreen {
    world
        .get_resource::<State<AppScreen>>()
        .map(|s| s.get().clone())
        .unwrap_or(AppScreen::Boot)
}

pub fn run_screen_transitions(world: &mut World) {
    world.run_schedule(StateTransition);
}

#[derive(Resource, Debug, Clone)]
pub struct TransitionTimer {
    pub total_duration: f32,
    pub remaining: f32,
    pub active: bool,
    pub input_blocked: bool,
    pub skip_requested: bool,
}

impl Default for TransitionTimer {
    fn default() -> Self {
        Self {
            total_duration: 0.5,
            remaining: 0.0,
            active: false,
            input_blocked: false,
            skip_requested: false,
        }
    }
}

impl TransitionTimer {
    pub fn start(duration: f32) -> Self {
        Self {
            total_duration: duration,
            remaining: duration,
            active: true,
            input_blocked: true,
            skip_requested: false,
        }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.active {
            return false;
        }
        if self.skip_requested {
            self.remaining = 0.0;
            self.active = false;
            self.input_blocked = false;
            self.skip_requested = false;
            return true;
        }
        self.remaining -= dt;
        if self.remaining <= 0.0 {
            self.remaining = 0.0;
            self.active = false;
            self.input_blocked = false;
            return true;
        }
        false
    }

    pub fn skip(&mut self) {
        self.skip_requested = true;
    }

    pub fn is_fading(&self) -> bool {
        self.active
    }

    pub fn progress(&self) -> f32 {
        if self.total_duration > 0.0 {
            (1.0 - (self.remaining / self.total_duration)).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}
