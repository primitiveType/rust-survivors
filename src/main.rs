use bevy_rapier2d::prelude::RapierDebugRenderPlugin;
use bevy_rapier2d::prelude::NoUserData;
use bevy_rapier2d::prelude::RapierPhysicsPlugin;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use bevy::prelude::*;
use bevy_asepritesheet::core::SpriteAnimController;
use bevy_asepritesheet::prelude::AsepritesheetPlugin;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use bevy_rapier2d::pipeline::CollisionEvent;
use bevy_rapier2d::plugin::{PhysicsSet, RapierConfiguration, TimestepMode};
use bevy_tween::DefaultTweenPlugins;

use components::{HealthUi};
use constants::BACKGROUND_COLOR;
use inspector::add_inspector;

use crate::{
    initialization::inspector,
    initialization::register_types::register_types,
    systems::*,
};
use crate::initialization::load_prefabs::Atlases;

mod components;

mod physics;

mod constants;

mod systems;

mod stepping;
mod setup;
mod extensions;
mod initialization;
mod bundles;
mod time;


#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum AppState {
    #[default]
    InGame,
    LevelUp,
}


fn main() {
    //TODO:
    //1 minute timer
    //make spawn rate more interesting
    //scale difficulty
    //level ups offer real choices
    // - 3 weapons
    // - 3 passives
    // show gun info on level up choice
    // display enemy health (maybe)
    // camera moves with player
    // add background
    // get rid of walls
//PATH=C:\Users\Arthu\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\bin\;E:\Unity Projects\rust-survivors\target\debug\deps
    let mut app_binding = App::new();
    let app: &mut App = app_binding
        .init_state::<AppState>()
        .insert_resource(Msaa::Off)
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            // timestep_mode: TimestepMode::Fixed {
            //     dt: time::DEFAULT_TIMESTEP.as_secs_f32(),
            //     substeps: 1,
            // },
            ..default()
        })
        .insert_resource(SpriteAnimController::default())
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))// prevents blurry sprites
        .add_plugins(DefaultTweenPlugins)
        .init_asset::<bevy_asepritesheet::aseprite_data::SpritesheetData>()
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0).with_default_system_setup(true).in_schedule(time::PhysicsSchedule))
        .add_plugins(time::TimePlugin)
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(
            AsepritesheetPlugin::new(&["sprite.json"]),
        )
        .add_plugins(
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .add_schedule(FixedUpdate)

                .at(Val::Percent(35.0), Val::Percent(50.0)),
        )
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Atlases { sprite_sheets: HashMap::new() })
        .add_event::<CollisionEvent>()
        //physics stuff, so that we can pause physics
        .add_systems(PostUpdate, time::run_physics_schedule)
        //startup systems, spawn player etc
        .add_systems(
            Startup,
            (
                initialization::load_prefabs::load_sprites,
                setup::setup,
                ui::setup,
                initialization::load_prefabs::load_gun_test,
            ).chain(),
        )
        // Add our gameplay simulation systems to the fixed timestep schedule
        // which runs at 64 Hz by default
        .add_systems(
            FixedUpdate,
            (
                spawning::enemy_spawn_cycle,
                guns::player_shoot,
                // audio::play_collision_sound,
                stats::die_at_zero_health,
                guns::destroy_bullets,
                animation::set_spritesheet_from_animation_info,
                animation::flip_sprite,
                animation::update_animation_state,
                guns::destroy_explosions,
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
             // movement::_debug_collisions,
             guns::deal_damage_on_collide,
             stats::pick_up_xp_on_touch,
             stats::vacuum_xp_on_touch,
             stats::level_up,
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
            (ui::toggle_level_ui_system, ui::pause_animations),
        )
        .add_systems(
            OnExit(AppState::LevelUp),
            (ui::toggle_level_ui_system, ui::resume_animations),
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
