#![feature(duration_constructors)]

use crate::physics::walls::WallBundle;
use bevy_rapier2d::prelude::{PhysicsSet, RapierDebugRenderPlugin};
use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowTheme};
use bevy_asepritesheet::core::SpriteAnimController;
use bevy_asepritesheet::prelude::AsepritesheetPlugin;
use bevy_ecs_ldtk::{LdtkPlugin, LevelSelection};
use bevy_ecs_ldtk::app::LdtkEntityAppExt;
use bevy_ecs_ldtk::prelude::LdtkIntCellAppExt;
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use bevy_rapier2d::pipeline::CollisionEvent;
use bevy_rapier2d::plugin::PhysicsSet::Writeback;
use bevy_rapier2d::plugin::RapierConfiguration;
use bevy_rapier2d::prelude::NoUserData;
use bevy_rapier2d::prelude::RapierPhysicsPlugin;
use bevy_tween::DefaultTweenPlugins;
use spew::prelude::{SpewApp, SpewPlugin};

use components::HealthUi;
use constants::BACKGROUND_COLOR;

use crate::{
    initialization::register_types::register_types,
    systems::*,
};
use crate::bundles::{CorpseSpawnData, EnemySpawnData, Object, PlayerBundle, PlayerSpawn, XPSpawnData};
use crate::initialization::inspector::add_inspector;
use crate::initialization::load_prefabs::{Atlases, Enemies};
use crate::physics::walls::Wall;
use crate::systems::guns::{DamageTextSpawnData, FireballSpawnData, FlaskSpawnData, IceballSpawnData, ParticleSpawnData};

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
    // this method needs to be inside main() method
    env::set_var("RUST_BACKTRACE", "full");
    //TODO:
    //replace ice spike sprite
    //replace molotov sprite
    //1 minute timer
    //make spawn rate more interesting
    //scale difficulty
    //level ups offer real choices
    // - 3 passives
    // --move speed
    // xp gain
    // damage

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
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()).set(WindowPlugin {
            primary_window: Some(Window {
                title: "Survive!".into(),
                name: Some("Survive!".into()),
                ..default()
            }),
            ..default()
        }),// prevents blurry sprites
                      DefaultTweenPlugins,
                      RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0).with_default_system_setup(true).in_schedule(time::PhysicsSchedule),
                      time::TimePlugin,
                      // (RapierDebugRenderPlugin::default()),
                      AsepritesheetPlugin::new(&["sprite.json"]),
                      stepping::SteppingPlugin::default()
                          .add_schedule(Update)
                          .add_schedule(FixedUpdate)

                          .at(Val::Percent(35.0), Val::Percent(50.0)),
                      EntropyPlugin::<WyRand>::default(),
                      EguiPlugin,
                      LdtkPlugin,
        ))
        .add_plugins((SpewPlugin::<Object, EnemySpawnData>::default(),
                      SpewPlugin::<Object, FireballSpawnData>::default(),
                      SpewPlugin::<Object, IceballSpawnData>::default(),
                      SpewPlugin::<Object, FlaskSpawnData>::default(),
                      SpewPlugin::<Object, DamageTextSpawnData>::default(),
                      SpewPlugin::<Object, CorpseSpawnData>::default(),
                      SpewPlugin::<Object, XPSpawnData>::default(),
                      SpewPlugin::<Object, ParticleSpawnData>::default(),
        ))
        .register_ldtk_entity::<PlayerSpawn>("Player_spawn")
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Atlases { sprite_sheets: HashMap::new() })
        .insert_resource(Enemies { datas: HashMap::new() })
        .insert_resource(SpriteAnimController::default())
        .insert_resource(LevelSelection::index(1))
        .insert_resource(spawning::RoundTimer { timer: Timer::new(Duration::from_mins(5), TimerMode::Once) })
        .init_asset::<bevy_asepritesheet::aseprite_data::SpritesheetData>()
        .add_event::<CollisionEvent>()
        .register_ldtk_int_cell_for_layer::<WallBundle>("Walls", 1)
        .add_spawner((Object::Enemy, bundles::spawn_enemy))
        .add_spawner((Object::Fireball, guns::spawn_fireball))
        .add_spawner((Object::Iceball, guns::spawn_iceball))
        .add_spawner((Object::Flask, guns::spawn_flask_projectile))
        .add_spawner((Object::DamageNumber, guns::spawn_damage_text))
        .add_spawner((Object::Corpse, bundles::spawn_corpse))
        .add_spawner((Object::XP, bundles::spawn_xp))
        .add_spawner((Object::Particle, guns::spawn_particle))
        //physics stuff, so that we can pause physics
        .add_systems(PostUpdate, time::run_physics_schedule)
        //startup systems, spawn player etc
        .add_systems(
            Startup,
            (
                initialization::load_prefabs::load_sprites,
                initialization::load_prefabs::load_enemy_prefabs,
                setup::setup,
                initialization::load_prefabs::load_gun_test,
            ).chain(),
        )
        // Add our gameplay simulation systems to the fixed timestep schedule
        // which runs at 64 Hz by default
        .add_systems(
            FixedUpdate,
            (
                guns::expire_entities,
                guns::expire_bullets_on_hit,
                guns::expired_bullets_explode,
                spawning::enemy_spawn_cycle,
                //abilities
                guns::advance_cooldowns,
                guns::fireball_gun,
                guns::iceball_gun,
                guns::flask_weapon,
                // audio::play_collision_sound,
                //stats
                stats::die_at_zero_health,
                guns::expire_bullets_on_hit,
                animation::set_spritesheet_from_animation_info,
                animation::flip_sprite,
                animation::update_animation_state,
                guns::destroy_after_death_anim,
                guns::destroy_expired_entities,
                stats::cold_enemies_spawn_particles
            ).run_if(in_state(AppState::InGame))
                // `chain`ing systems together runs them in order
                .chain(),
        )

        .add_systems(
            //InGame update loop
            Update,
            (
                // spawning::draw_level_bounds,
                spawning::set_level_bounds,
                physics::walls::spawn_wall_collision,
                spawning::move_player_to_spawn_point,
                (stats::update_move_speed_from_passive,
                 movement::apply_move_speed_multiplier,
                 movement::move_player,
                 movement::set_follower_velocity,
                 stats::move_speed_mod_affects_animation_speed
                ).chain(),
                ui::update_player_health_ui,
                // movement::_debug_collisions,
                guns::deal_damage_on_collide,
                guns::apply_cold_on_collide,
                movement::apply_xp_radius,
                movement::apply_xp_multiplier,
                stats::pick_up_xp_on_touch,
                stats::vacuum_xp_on_touch,
                stats::level_up,
                ui_example_system,
                ui::fade_text,
                (stats::reset_enemy_color,
                 stats::cold_objects_are_blue,
                ).chain(),
                (movement::camera_follow).after(PhysicsSet::Writeback),
            ).run_if(in_state(AppState::InGame)))
        .add_systems(Update,
                     (stats::update_level_descriptions_xp_multiplier,
                      stats::update_level_descriptions_xp_radius,
                      stats::update_level_descriptions_flask,
                      stats::update_level_descriptions_fireball,
                      stats::update_level_descriptions_move_speed,
                      stats::update_level_descriptions_iceball, ))
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
            (ui::prepare_level_up, ui::pause_animations),
        )
        .add_systems(
            OnExit(AppState::LevelUp),
            (ui::resume_animations, ui::cleanup_level_up),
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
    println!("{}", app.is_plugin_added::<EguiPlugin>());
    // let app: &mut App = add_inspector(app);
    let app: &mut App = register_types(app);

    app.run();
}

fn ui_example_system(contexts: EguiContexts) {}

