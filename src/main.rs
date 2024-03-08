use std::fmt::Debug;
use std::hash::Hash;

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use bevy_xpbd_2d::prelude::*;

use components::{CollisionEvent, CollisionSound, HealthUi};
use constants::BACKGROUND_COLOR;
use inspector::add_inspector;

use crate::{
    initialization::inspector,
    initialization::register_types::register_types,
    systems::*,
};

mod components;

mod physics;

mod constants;

mod systems;

mod stepping;
mod setup;
mod extensions;
mod initialization;
mod bundles;


#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum AppState {
    #[default]
    InGame,
    LevelUp,
}


fn main() {
    //TODO:
    // data drive guns
    // show gun info on level up choice
    // display enemy health (maybe)
    // camera moves with player
    // add background
    // get rid of walls
//PATH=C:\Users\Arthu\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\bin\;E:\Unity Projects\rust-survivors\target\debug\deps
    let mut app_binding = App::new();
    let app: &mut App = app_binding
        .init_state::<AppState>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))// prevents blurry sprites
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .add_schedule(FixedUpdate)

                .at(Val::Percent(35.0), Val::Percent(50.0)),
        )
        .add_plugins(EntropyPlugin::<WyRand>::default())

        // .add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(Gravity(Vec2::default()))
        .insert_resource(SubstepCount(6))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_event::<CollisionEvent>()
        .add_systems(Startup, (setup::setup, ui::setup, initialization::load_prefabs::load_gun_test, bundles::setup_assets))
        // Add our gameplay simulation systems to the fixed timestep schedule
        // which runs at 64 Hz by default
        .add_systems(
            FixedUpdate,
            (
                spawning::enemy_spawn_cycle,
                guns::player_shoot,
                audio::play_collision_sound,
                stats::die_at_zero_health,
                guns::destroy_bullets,
                bundles::animate_sprite,
                bundles::flip_sprite,
            ).run_if(in_state(AppState::InGame))
                // `chain`ing systems together runs them in order
                .chain(),
        )
        .add_systems(
            //InGame update loop
            Update,
            (movement::move_player,
             movement::set_follower_velocity,
             ui::update_player_health_ui,
             movement::player_takes_damage_from_enemy,
             guns::enemy_takes_damage_from_bullets,
             stats::pickup_xp,
            ).run_if(in_state(AppState::InGame)))
        .add_systems(Update,
                     (//Always update loop
                      bevy::window::close_on_esc,
                      dev::log_transitions
                     ),
        )
        .add_systems(Update,
                     (
                         //level up update loop
                         ui::button_system,
                     ).run_if(in_state(AppState::LevelUp)))
        .add_systems(
            OnEnter(AppState::LevelUp),
            ui::toggle_level_ui_system,
        )
        .add_systems(
            OnExit(AppState::LevelUp),
            ui::toggle_level_ui_system,
        )
        .add_systems(
            OnEnter(AppState::InGame),
            physics::time::unpause,
        )
        .add_systems(
            OnExit(AppState::InGame),
            physics::time::pause,
        )
        ;

    let app: &mut App = add_inspector(app);
    let app: &mut App = register_types(app);

    app.run();
}
