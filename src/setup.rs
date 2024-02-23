use bevy_rand::resource::GlobalEntropy;
use bevy::prelude::{Bundle, Camera2dBundle, Circle, Color, ColorMaterial, Commands, default, Mesh, PositionType, Res, ResMut, Sprite, SpriteBundle, Style, TextBundle, TextSection, TextStyle, Transform};
use bevy::asset::{Assets, AssetServer};
use bevy::core::Name;
use bevy::math::{Vec2, Vec3};
use bevy::sprite::MaterialMesh2dBundle;
use bevy_xpbd_2d::math::Vector2;
use crate::{Ball, BALL_COLOR, BALL_DIAMETER, BALL_SPEED, BALL_STARTING_POSITION, BOTTOM_WALL, Brick, BRICK_COLOR, BRICK_SIZE, Collider, CollisionSound, Enemy, FollowPlayer, GAP_BETWEEN_BRICKS, GAP_BETWEEN_BRICKS_AND_CEILING, GAP_BETWEEN_BRICKS_AND_SIDES, GAP_BETWEEN_PADDLE_AND_BRICKS, GAP_BETWEEN_PADDLE_AND_FLOOR, INITIAL_BALL_DIRECTION, LEFT_WALL, MoveSpeed, Paddle, PADDLE_COLOR, PADDLE_SIZE, RIGHT_WALL, SCORE_COLOR, SCOREBOARD_FONT_SIZE, SCOREBOARD_TEXT_PADDING, ScoreboardUi, TEXT_COLOR, TOP_WALL, WallBundle, WallLocation};
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use rand_core::RngCore;
use bevy_xpbd_2d::prelude::*;
use crate::physics::layers::GameLayer;

const ENEMY_COLOR: Color = Color::rgb(1.0, 0.1, 0.1);

// Add the game's entities to our world
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<GlobalEntropy<WyRand>>
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // Paddle
    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;


    commands.spawn((
        RigidBody::Kinematic,
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
        Paddle,
        Mass(10.0),
        Collider::rectangle(1.0, 1.0),
        Friction::ZERO,
        Restitution::new(1.0),
        LinearVelocity(Vector2::ZERO),
        Name::new("Player"),
        CollisionLayers::new(GameLayer::Player, [GameLayer::Ball, GameLayer::Ground]),

    ));

    let mut bundles: Vec<(RigidBody, MaterialMesh2dBundle<ColorMaterial>, Collider, Friction, Restitution, Ball, CollisionLayers, LinearVelocity)> = vec![];
    let count = 1;
    for index in 0..count {
        let offset : f32 = (index as f32) - (count as f32 / 2.0);
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


    let ball_command = commands.spawn_batch(bundles);


    // Scoreboard
    commands.spawn((
        ScoreboardUi,
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
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

    // Bricks
    let total_width_of_bricks = (RIGHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES;
    let bottom_edge_of_bricks = paddle_y + GAP_BETWEEN_PADDLE_AND_BRICKS;
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
            let spawned = commands.spawn((
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

    spawn_enemies(commands, meshes, materials, asset_server);
}

pub fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut spawned = commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(ENEMY_COLOR),
            transform: Transform::from_translation(BALL_STARTING_POSITION)
                .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.)),
            ..default()
        },
        Enemy, FollowPlayer
    ));

    spawned.insert(MoveSpeed::new(100.0));
    spawned.insert(LinearVelocity(Vec2::new(0.0, 0.0)));
    spawned.insert(Collider::circle(0.5));
    spawned.insert(RigidBody::Dynamic);

    spawned.insert(Name::new("enemy"));
}