use bevy::asset::{Assets, AssetServer};
use bevy::core::Name;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Camera2dBundle, Circle, Color, ColorMaterial, Commands, default, Mesh, PositionType, Res, ResMut, Sprite, SpriteBundle, Style, TextBundle, TextSection, TextStyle, Transform};
use bevy::sprite::MaterialMesh2dBundle;
use bevy_prng::WyRand;
use bevy_rand::resource::GlobalEntropy;
use bevy_xpbd_2d::math::Vector2;
use bevy_xpbd_2d::prelude::*;
use rand_core::RngCore;

use crate::{Ball, BALL_COLOR, BALL_DIAMETER, BALL_SPEED, BALL_STARTING_POSITION, BOTTOM_WALL, Brick, BRICK_COLOR, BRICK_SIZE, bundles, Collider, CollisionSound, GAP_BETWEEN_BRICKS, GAP_BETWEEN_BRICKS_AND_CEILING, GAP_BETWEEN_BRICKS_AND_SIDES, GAP_BETWEEN_PADDLE_AND_BRICKS, GAP_BETWEEN_PADDLE_AND_FLOOR, Gun, Health, HealthUi, LEFT_WALL, PADDLE_COLOR, PADDLE_SIZE, Player, RIGHT_WALL, SCORE_COLOR, SCOREBOARD_FONT_SIZE, SCOREBOARD_TEXT_PADDING, TEXT_COLOR, TOP_WALL, WallBundle, WallLocation};
use crate::physics::layers::GameLayer;


// Add the game's entities to our world
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    rng: ResMut<GlobalEntropy<WyRand>>,
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

    // spawn_bricks(&mut commands);

    bundles::spawn_enemy(commands, meshes, materials);
}

fn spawn_bricks(commands: &mut Commands) {
    let total_width_of_bricks = (RIGHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES;
    let bottom_edge_of_bricks = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR + GAP_BETWEEN_PADDLE_AND_BRICKS;
    let total_height_of_bricks = TOP_WALL - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING;

    assert!(total_width_of_bricks > 0.0);
    assert!(total_height_of_bricks > 0.0);

    // Given the space available, compute how many rows and columns of bricks we can fit
    let n_columns = (total_width_of_bricks / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_rows = (total_height_of_bricks / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_vertical_gaps = n_columns - 1;

    // Because we need to round the number of columns,
    // the space on the top and sides of the bricks only captures a lower bound, not an exact value
    let center_of_bricks = (LEFT_WALL + RIGHT_WALL) / 2.0;
    let left_edge_of_bricks = center_of_bricks
        // Space taken up by the bricks
        - (n_columns as f32 / 2.0 * BRICK_SIZE.x)
        // Space taken up by the gaps
        - n_vertical_gaps as f32 / 2.0 * GAP_BETWEEN_BRICKS;

    // In Bevy, the `translation` of an entity describes the center point,
    // not its bottom-left corner
    let offset_x = left_edge_of_bricks + BRICK_SIZE.x / 2.;
    let offset_y = bottom_edge_of_bricks + BRICK_SIZE.y / 2.;

    for row in 0..n_rows {
        for column in 0..n_columns {
            let brick_position = Vec2::new(
                offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),
                offset_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),
            );

            // brick
            let _spawned = commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: BRICK_COLOR,
                        ..default()
                    },
                    transform: Transform {
                        translation: brick_position.extend(0.0),
                        scale: Vec3::new(BRICK_SIZE.x, BRICK_SIZE.y, 1.0),
                        ..default()
                    },
                    ..default()
                },
                Brick,
                Collider::rectangle(1.0, 1.0),
                Friction::ZERO,
                Restitution::new(1.0),
                RigidBody::Static
            ));
        }
    }
}

fn spawn_balls(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<ColorMaterial>>, mut rng: ResMut<GlobalEntropy<WyRand>>) {
    let mut bundles: Vec<(RigidBody, MaterialMesh2dBundle<ColorMaterial>, Collider, Friction, Restitution, Ball, CollisionLayers, LinearVelocity)> = vec![];
    let count = 1;
    for _ in 0..count {
        let ball: (RigidBody, MaterialMesh2dBundle<ColorMaterial>, Collider, Friction, Restitution, Ball, CollisionLayers, LinearVelocity) = (
            RigidBody::Dynamic,
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::default()).into(),
                material: materials.add(BALL_COLOR),
                transform: Transform::from_translation(BALL_STARTING_POSITION)
                    .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.)),
                ..default()
            },
            Collider::circle(0.5),
            Friction::ZERO,
            Restitution::new(1.0),
            Ball,
            CollisionLayers::new(GameLayer::Ball, [GameLayer::Ground, GameLayer::Player]),
            LinearVelocity(Vector2::new(rng.next_u32() as f32, rng.next_u32() as f32).normalize() * BALL_SPEED),
        );

        bundles.push(ball);
    }


    commands.spawn_batch(bundles);
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
        Player,
        Gun { last_shot_time: 0 },
        Health { value: 100.0 },
        Mass(10.0),
        Collider::rectangle(1.0, 1.0),
        Friction::ZERO,
        Restitution::new(1.0),
        LinearVelocity(Vector2::ZERO),
        Name::new("Player"),
        CollisionLayers::new(GameLayer::Player, [GameLayer::Ball, GameLayer::Ground, GameLayer::Enemy]),
        LockedAxes::ROTATION_LOCKED,
    ));
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