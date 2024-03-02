use std::fmt::Debug;
use std::hash::Hash;

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use bevy_xpbd_2d::prelude::*;

use inspector::add_inspector;

use crate::initialization::inspector;
use crate::initialization::register_types::register_types;
use crate::physics::layers::GameLayer;
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

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
const PADDLE_SIZE: Vec3 = Vec3::new(50.0, 50.0, 1.0);
const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
const PADDLE_SPEED: f32 = 100_000.0;
// How close can the paddle get to the wall
const PADDLE_PADDING: f32 = 10.0;

// We set the z-value of the ball to 1 so it renders on top in the case of overlapping sprites.
const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 0.0);
const BALL_DIAMETER: f32 = 30.;
const XP_DIAMETER: f32 = 5.;

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;


const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(20.0);

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);


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
                         log_transitions
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

/// print when an `AppState` transition happens
/// also serves as an example of how to use `StateTransitionEvent`
fn log_transitions(mut transitions: EventReader<StateTransitionEvent<AppState>>) {
    for transition in transitions.read() {
        info!(
            "transition: {:?} => {:?}",
            transition.before, transition.after
        );
    }
}
#[derive(Component, Default)]
struct Player {
    xp: u16,
    level: u16,
}

#[derive(Component)]
struct Gun {
    last_shot_time: u128,
}

#[derive(Component)]
struct Bullet {
    damage: f32,
    hits: u8,
    pierce: u8,
    lifetime: u128,
    timestamp: u128,
}

impl Default for Bullet {
    fn default() -> Self {
        Bullet {
            damage: 1.0,
            hits: 0,
            pierce: 0,
            lifetime: 5_000,
            timestamp: 0,
        }
    }
}

#[derive(Component)]
struct Health {
    value: f32,
}

#[derive(Component, Clone)]
struct Ball;

#[derive(Component)]
struct FollowPlayer;


#[derive(Component, Reflect)]
struct MoveSpeed {
    value: f32,
}

impl MoveSpeed {
    pub(crate) fn new(value: f32) -> Self {
        MoveSpeed { value }
    }
}

#[derive(Component)]
struct Enemy {
    xp: u16,
}

#[derive(Component)]
struct DamageOnTouch {
    value: f32,
}

#[derive(Component)]
struct GainXPOnTouch {
    value: u16,
}

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Component)]
struct Brick;

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
struct BulletBundle {
    material: MaterialMesh2dBundle<ColorMaterial>,
    collider: Collider,
    rigid_body: RigidBody,
    friction: Friction,
    restitution: Restitution,
    mask: CollisionLayers,
    bullet: Bullet,
    mass: Mass,
    linear_velocity: LinearVelocity,
}


// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
struct WallBundle {
    // You can nest bundles inside of other bundles like this
    // Allowing you to compose their functionality
    sprite_bundle: SpriteBundle,
    collider: Collider,
    rigid_body: RigidBody,
    friction: Friction,
    restitution: Restitution,
    mask: CollisionLayers,
}

/// Which side of the arena is this wall located on?
enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        // Make sure we haven't messed up our constants
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    // This "builder method" allows us to reuse logic across our wall entities,
    // making our code easier to read and less prone to bugs when we change the logic
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                    // This is used to determine the order of our sprites
                    translation: location.position().extend(0.0),
                    // The z-scale of 2D objects must always be 1.0,
                    // or their ordering will be affected in surprising ways.
                    // See https://github.com/bevyengine/bevy/issues/4149
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider::rectangle(1.0, 1.0),
            rigid_body: RigidBody::Static,
            friction: Friction::ZERO,
            restitution: Restitution::new(1.0),
            mask: CollisionLayers::new(GameLayer::Ground, [GameLayer::Ball, GameLayer::Player, GameLayer::Enemy]),
        }
    }
}


#[derive(Component)]
struct HealthUi;

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
