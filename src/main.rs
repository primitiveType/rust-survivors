use std::fmt::Debug;
use std::hash::Hash;

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use bevy_xpbd_2d::prelude::*;
use components::{CollisionEvent, CollisionSound, Health, HealthUi, Player};
use constants::{BACKGROUND_COLOR, BOTTOM_WALL, LEFT_WALL, PADDLE_PADDING, PADDLE_SIZE, PADDLE_SPEED, RIGHT_WALL, TOP_WALL, WALL_THICKNESS};

use inspector::add_inspector;
use systems::dev;

use crate::initialization::inspector;
use crate::initialization::register_types::register_types;
use crate::systems::{guns, stats, ui};
use crate::systems::guns::enemy_takes_damage_from_bullets;
use crate::systems::movement::{log_paddle_collide, set_follower_velocity};
use crate::systems::movement::{destroy_brick_on_collide, player_takes_damage_from_enemy};
use crate::systems::spawning::enemy_spawn_cycle;

mod systems;

mod stepping;
mod setup;
mod extensions;
mod initialization;
mod physics;
mod bundles;
mod constants;
mod components;


#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
enum AppState {
    #[default]
    InGame,
    LevelUp,
}


fn main() {
    //TODO:
    // choose weapons/bonuses when levelling? requires ui?
    // display enemy health (maybe)
    // projectiles can be added to player over time
    // camera moves with player
    // add background
    // get rid of walls

    let mut app_binding = App::new();
    let app: &mut App = app_binding
        .init_state::<AppState>()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .add_schedule(FixedUpdate)

                .at(Val::Percent(35.0), Val::Percent(50.0)),
        )
        .add_plugins(EntropyPlugin::<WyRand>::default())

        .add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(Gravity(Vec2::default()))
        .insert_resource(SubstepCount(6))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_event::<CollisionEvent>()
        .add_systems(Startup, (setup::setup, ui::setup))
        // Add our gameplay simulation systems to the fixed timestep schedule
        // which runs at 64 Hz by default
        .add_systems(
            FixedUpdate,
            (
                enemy_spawn_cycle,
                guns::player_shoot,
                play_collision_sound,
                log_paddle_collide,
                stats::die_at_zero_health,
                guns::destroy_bullets,
            ).run_if(in_state(AppState::InGame))
                // `chain`ing systems together runs them in order
                .chain(),
        )
        .add_systems(PostProcessCollisions, destroy_brick_on_collide)
        .add_systems(
            //InGame update loop
            Update,
            (move_player,
             set_follower_velocity,
             update_player_health_ui,
             player_takes_damage_from_enemy,
             enemy_takes_damage_from_bullets,
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

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut LinearVelocity, With<Player>>,
    time: Res<Time>,
) {
    let mut paddle_velocity = query.single_mut();
    let mut direction: Vec2 = Default::default();

    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
        direction = direction.normalize();
    }

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
        direction = direction.normalize();
    }


    // Calculate the new horizontal paddle position based on player input
    let new_player_velocity: Vec2 =
        direction * PADDLE_SPEED * time.delta_seconds();

    // Update the paddle position,
    // making sure it doesn't cause the paddle to leave the arena
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    let upper_bound = TOP_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.y / 2.0 + PADDLE_PADDING;
    let lower_bound = BOTTOM_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.y / 2.0 - PADDLE_PADDING;
    paddle_velocity.x = new_player_velocity.x.clamp(left_bound, right_bound);
    paddle_velocity.y = new_player_velocity.y.clamp(lower_bound, upper_bound);
}


fn update_player_health_ui(player_query: Query<(&Health, &Player)>, mut query: Query<&mut Text, With<HealthUi>>) {
    let mut text = query.single_mut();
    let (player_health, player) = player_query.single();
    text.sections[1].value = player_health.value.to_string();
    text.sections[3].value = player.xp.to_string();
}


fn play_collision_sound(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    sound: Res<CollisionSound>,
) {
    // Play a sound once per frame if a collision occurred.
    if !collision_events.is_empty() {
        // This prevents events staying active on the next frame.
        collision_events.clear();
        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            // auto-despawn the entity when playback finishes
            settings: PlaybackSettings::DESPAWN,
        });
    }
}
