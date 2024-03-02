use bevy::asset::{Assets, AssetServer};
use bevy::core::Name;
use bevy::math::Vec3;
use bevy::prelude::{Camera2dBundle, ColorMaterial, Commands, default, Mesh, PositionType, Res, ResMut, Sprite, SpriteBundle, Style, TextBundle, TextSection, TextStyle, Transform};
use bevy_xpbd_2d::math::Vector2;
use bevy_xpbd_2d::prelude::*;

use crate::*;
use crate::components::{Gun, WallBundle, WallLocation};
use crate::constants::{BOTTOM_WALL, PADDLE_SIZE};
use crate::constants::{GAP_BETWEEN_PADDLE_AND_FLOOR, PADDLE_COLOR, SCORE_COLOR, SCOREBOARD_FONT_SIZE, SCOREBOARD_TEXT_PADDING, TEXT_COLOR};
use crate::physics::layers::GameLayer;

// Add the game's entities to our world
pub fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    spawn_player(&mut commands);

    spawn_background(&mut commands);

    // spawn_balls(&mut commands, &mut meshes, &mut materials, rng);


    // Scoreboard
    commands.spawn((
        HealthUi,
        TextBundle::from_sections([
            TextSection::new(
                "Health: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
            TextSection::new(
                " XP: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
        ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            }),
    ));

    // Walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));


    bundles::spawn_enemy(commands, meshes, materials);
}


fn spawn_player(commands: &mut Commands) {
    // Paddle
    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;

    commands.spawn((
        RigidBody::Dynamic,
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, paddle_y, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        Player { ..default() },
        Health { value: 100.0 },
        Mass(10.0),
        Collider::rectangle(1.0, 1.0),
        Friction::ZERO,
        Restitution::new(1.0),
        LinearVelocity(Vector2::ZERO),
        Name::new("Player"),
        CollisionLayers::new(GameLayer::Player, [GameLayer::Ball, GameLayer::Ground, GameLayer::Enemy, GameLayer::XP]),
        LockedAxes::ROTATION_LOCKED,
    )).with_children(|parent| {
        parent.spawn((Gun { last_shot_time: 0, cooldown: 1_000 }, SpatialBundle { ..default() }));
        parent.spawn((Gun { last_shot_time: 0, cooldown: 2_000 }, SpatialBundle { ..default() }));
        parent.spawn((Gun { last_shot_time: 0, cooldown: 500 }, SpatialBundle { ..default() }));
        parent.spawn((Gun { last_shot_time: 0, cooldown: 125 }, SpatialBundle { ..default() }));
    });
}

fn spawn_background(_commands: &mut Commands) {

    // commands.spawn((
    //     SpriteBundle {
    //         transform: Transform {
    //             translation: Vec3::new(0.0, 0.0, -10.0),
    //             scale: PADDLE_SIZE,
    //             ..default()
    //         },
    //         sprite: Sprite {
    //             color: PADDLE_COLOR,
    //             ..default()
    //         },
    //         ..default()
    //     },
    //
    //     Name::new("Background"),
    // ));
}