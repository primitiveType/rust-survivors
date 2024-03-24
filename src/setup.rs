use bevy_rapier2d::geometry::{ActiveEvents, Collider, Restitution, Sensor};
use bevy_rapier2d::prelude::CollisionGroups;
use crate::*;
use crate::components::{Gun, XPVacuum};
use crate::constants::{SCORE_COLOR, SCOREBOARD_FONT_SIZE, SCOREBOARD_TEXT_PADDING, TEXT_COLOR};
use crate::physics::layers::game_layer;

// Add the game's entities to our world
// #[bevycheck::system]
pub fn setup(
    mut commands: Commands,
    atlases: ResMut<Atlases>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Sound
    // let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    // commands.insert_resource(CollisionSound(ball_collision_sound));

    spawn_player(&mut commands, atlases);

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
    // commands.spawn(WallBundle::new(WallLocation::Left));
    // commands.spawn(WallBundle::new(WallLocation::Right));
    // commands.spawn(WallBundle::new(WallLocation::Bottom));
    // commands.spawn(WallBundle::new(WallLocation::Top));
}


fn spawn_player(commands: &mut Commands,
                atlases: ResMut<Atlases>,
) {
    commands.spawn(bundles::PlayerBundle::with_sprite(atlases))

        .with_children(|parent| {
            // parent.spawn((Gun { last_shot_time: 0, cooldown: 1_000 }, SpatialBundle { ..default() }));
            parent.spawn((Gun {
                last_shot_time: 0,
                cooldown: 1_00,
                bullet_size: 50_f32,
                pierce: 0,
                bullet_speed: 380_f32,
            }, SpatialBundle { ..default() }));
            // 
            parent.spawn((XPVacuum {},
                          Collider::ball(50.0),
                          // Friction::ZERO,
                          Restitution::new(1.0),
                          CollisionGroups::new(game_layer::XP, game_layer::XP),
                          // LockedAxes::ROTATION_LOCKED,
                          SpatialBundle { ..default() },
                          Sensor,
                          ActiveEvents::COLLISION_EVENTS,
            ));
            // parent.spawn((Gun { last_shot_time: 0, cooldown: 500 }, SpatialBundle { ..default() }));
            // parent.spawn((Gun { last_shot_time: 0, cooldown: 125 }, SpatialBundle { ..default() }));
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