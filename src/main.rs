#![feature(duration_constructors)]

use crate::physics::walls::WallBundle;
use bevy_rapier2d::prelude::{PhysicsSet, RapierDebugRenderPlugin};
use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

use bevy::prelude::*;
use bevy::utils::tracing::field::debug;
use bevy_asepritesheet::core::SpriteAnimController;
use bevy_asepritesheet::prelude::AsepritesheetPlugin;
use bevy_ecs_ldtk::app::LdtkEntityAppExt;
use bevy_ecs_ldtk::prelude::LdtkIntCellAppExt;
use bevy_ecs_ldtk::{LdtkPlugin, LevelSelection};
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_ggrs::{checksum_hasher, GgrsApp, GgrsPlugin, GgrsSchedule, ReadInputs};
use bevy_matchbox::matchbox_socket::PeerId;
use bevy_rapier2d::dynamics::Velocity;

use bevy_rapier2d::pipeline::CollisionEvent;
use bevy_rapier2d::plugin::{RapierConfiguration, TimestepMode};
use bevy_rapier2d::prelude::NoUserData;
use bevy_rapier2d::prelude::RapierPhysicsPlugin;
use bevy_rapier2d::rapier::dynamics::RigidBodyPosition;
use bevy_tween::DefaultTweenPlugins;
use clap::Parser;
use spew::prelude::{SpewApp, SpewPlugin};

use components::HealthUi;
use constants::BACKGROUND_COLOR;

use crate::bundles::{CorpseSpawnData, EnemySpawnData, Object, PlayerSpawn, XPSpawnData};
use crate::components::Cold;
use crate::initialization::load_prefabs::{Atlases, Enemies};
use crate::systems::guns::{
    DamageTextSpawnData, Damaged, FireballSpawnData, FlaskSpawnData, IceballSpawnData,
    ParticleSpawnData,
};
use crate::{initialization::register_types::register_types, systems::*};
use crate::args::Args;
use crate::initialization::inspector::add_inspector;
use crate::systems::spawning::enemy_spawn_cycle;

mod components;

mod physics;

mod constants;

mod systems;

mod bundles;
mod extensions;
mod initialization;
mod setup;
mod stepping;
mod time;
pub mod random;
mod args;

type Config = bevy_ggrs::GgrsConfig<u8, PeerId>;

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum AppState {
    #[default]
    WaitingForPlayers,
    InGame,
    LevelUp,
}

fn main() {
    let args = Args::parse();
    eprintln!("{args:?}");
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
        .insert_resource(args.clone())
        .init_state::<AppState>()
        .insert_resource(Msaa::Off)
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            timestep_mode: TimestepMode::Fixed {
                dt: time::DEFAULT_TIMESTEP.as_secs_f32(),
                substeps: 1,
            },
            ..default()
        })
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Survive!".into(),
                        name: Some("Survive!".into()),
                        ..default()
                    }),
                    ..default()
                }), // prevents blurry sprites
            DefaultTweenPlugins,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0)
                .with_default_system_setup(true)
                .in_schedule(time::PhysicsSchedule),
            time::TimePlugin,
            AsepritesheetPlugin::new(&["sprite.json"]),
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .add_schedule(FixedUpdate)
                .at(Val::Percent(35.0), Val::Percent(50.0)),

            EguiPlugin,
            LdtkPlugin,
        ))
        .add_plugins((
            GgrsPlugin::<Config>::default(), // NEW
            SpewPlugin::<Object, EnemySpawnData>::default(),
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
        .insert_resource(Atlases {
            sprite_sheets: HashMap::new(),
        })
        .insert_resource(Enemies {
            datas: HashMap::new(),
        })
        .insert_resource(SpriteAnimController::default())
        .insert_resource(LevelSelection::index(1))
        .insert_resource(spawning::RoundTimer {
            timer: Timer::new(Duration::from_mins(5), TimerMode::Once),
        })
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
            )
                .chain(),
        )
        // Add our gameplay simulation systems to the fixed timestep schedule
        // which runs at 64 Hz by default
        .add_systems(
            FixedUpdate,
            (
                guns::expire_entities,
                guns::expire_bullets_on_hit,
                guns::expired_bullets_explode,
                //abilities
                guns::advance_cooldowns,
                guns::fireball_gun,
                guns::iceball_gun,
                // audio::play_collision_sound,
                //stats
                stats::die_at_zero_health,
                guns::expire_bullets_on_hit,
                animation::set_spritesheet_from_animation_info,
                animation::flip_sprite,
                animation::update_animation_state,
                guns::destroy_after_death_anim,
                guns::destroy_expired_entities,
                stats::cold_enemies_spawn_particles,
            )
                .run_if(in_state(AppState::InGame))
                // `chain`ing systems together runs them in order
                .chain(),
        )
        .add_systems(PreUpdate, spawning::set_level_bounds)
        .add_systems(ReadInputs, movement::read_local_inputs)

        .add_systems(
            GgrsSchedule,
            (movement::move_player,
             spawning::enemy_spawn_cycle,
             guns::flask_weapon).chain()
        )
        .rollback_component_with_clone::<Transform>() // NEW
        .rollback_component_with_clone::<Velocity>() // NEW
        // We must add a specific checksum check for everything we want to include in desync detection.
        // It is probably OK to just check the components, but for demo purposes let's make sure Rapier always agrees.
        .checksum_resource_with_hash::<PhysicsRollbackState>()
        .rollback_resource_with_clone::<PhysicsRollbackState>()
        // Store everything that Rapier updates in its Writeback stage
        .rollback_component_with_reflect::<GlobalTransform>()
        .rollback_component_with_reflect::<Transform>()
        .rollback_component_with_reflect::<Velocity>()
        .rollback_component_with_reflect::<Sleeping>()
        // Game stuff
        .rollback_resource_with_reflect::<EnablePhysicsAfter>();
        .checksum_component::<Transform>(checksum_transform) // new
        .add_systems(
            //InGame update loop
            Update,
            (
                spawning::set_level_bounds,
                physics::walls::spawn_wall_collision,
                (
                    stats::update_move_speed_from_passive,
                    movement::apply_move_speed_multiplier,
                    movement::set_follower_velocity,
                    stats::move_speed_mod_affects_animation_speed,
                )
                    .chain(),
                ui::update_player_health_ui,
                // movement::_debug_collisions,
                guns::deal_damage_on_collide,
                guns::deal_damage_on_collide_start,
                guns::apply_cold_on_collide,
                guns::apply_cold_on_collide_start,
                movement::apply_xp_radius,
                movement::apply_xp_multiplier,
                stats::pick_up_xp_on_touch,
                stats::vacuum_xp_on_touch,
                stats::level_up,
                ui::fade_text,
                (
                    stats::reset_sprite_color,
                    stats::cold_objects_are_blue,
                    stats::highlight_damaged,
                )
                    .chain(),
                (movement::camera_follow).after(PhysicsSet::Writeback),
                guns::process_temporary_component::<Damaged>,
                guns::process_temporary_component::<Cold>,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                (setup::start_sync_test).run_if(synctest_mode),
                (setup::start_p2p).run_if(p2p_mode),
            ).run_if(in_state(AppState::WaitingForPlayers)))
        .add_systems(
            Update,
            (
                stats::update_level_descriptions_xp_multiplier,
                stats::update_level_descriptions_xp_radius,
                stats::update_level_descriptions_flask,
                stats::update_level_descriptions_fireball,
                stats::update_level_descriptions_move_speed,
                stats::update_level_descriptions_iceball,
            ),
        )
        .add_systems(
            Update,
            (
                //Always update loop
                bevy::window::close_on_esc,
                dev::log_transitions,
            ),
        )
        .add_systems(
            Update,
            (
                //level up update loop
                ui::button_system,
            )
                .run_if(in_state(AppState::LevelUp)),
        )

        .add_systems( Update, (spawning::move_player_to_spawn_point),)
        .add_systems(
            OnEnter(AppState::LevelUp),
            (ui::prepare_level_up, ui::pause_animations),
        )
        .add_systems(
            OnExit(AppState::LevelUp),
            (ui::resume_animations, ui::cleanup_level_up),
        )
        .add_systems(OnEnter(AppState::InGame), physics::time::unpause)
        .add_systems(OnExit(AppState::InGame), physics::time::pause);
    println!("{}", app.is_plugin_added::<EguiPlugin>());

    if args.debug {
        let app: &mut App = add_inspector(app);

        app.add_plugins(RapierDebugRenderPlugin::default());

        app.add_systems(FixedUpdate, spawning::draw_level_bounds);
    }
    let app: &mut App = register_types(app);

    app.run();
}
pub fn checksum_transform(transform: &Transform) -> u64 {
    let mut hasher = checksum_hasher();

    assert!(
        transform.is_finite(),
        "Hashing is not stable for NaN f32 values."
    );

    transform.translation.x.to_bits().hash(&mut hasher);
    transform.translation.y.to_bits().hash(&mut hasher);
    transform.translation.z.to_bits().hash(&mut hasher);

    transform.rotation.x.to_bits().hash(&mut hasher);
    transform.rotation.y.to_bits().hash(&mut hasher);
    transform.rotation.z.to_bits().hash(&mut hasher);
    transform.rotation.w.to_bits().hash(&mut hasher);

    hasher.finish()
}
fn synctest_mode(args: Res<Args>) -> bool {
    args.synctest
}

fn p2p_mode(args: Res<Args>) -> bool {
    !args.synctest
}
