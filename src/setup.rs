use bevy_rapier2d::geometry::{ActiveEvents, Collider, Restitution, Sensor};
use bevy_rapier2d::prelude::{CollisionGroups};
use crate::*;
use crate::components::{Cooldown, AttackSpeed, FireBallGun, XPVacuum, Flask, AbilityLevel, ParentMoveSpeedMultiplier, PassiveMoveSpeedMultiplier, IceBallGun};
use crate::constants::{SCORE_COLOR, SCOREBOARD_FONT_SIZE, SCOREBOARD_TEXT_PADDING, TEXT_COLOR};
use crate::physics::layers::game_layer;

// Add the game's entities to our world
// #[bevycheck::system]
pub fn setup(
    mut commands: Commands,
    atlases: ResMut<Atlases>,
) {
    // Camera
    let camera = commands.spawn(Camera2dBundle::default());


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
            //fireball gun
            parent.spawn((Cooldown::with_cooldown(1000),
                          FireBallGun {},
                          Name::new("Fireball"),
                          AbilityLevel { level: 1, ..default() },
                          SpatialBundle { ..default() }));
            //iceball gun
            parent.spawn((Cooldown::with_cooldown(1800),
                          IceBallGun {},
                          Name::new("Iceball"),
                          AbilityLevel { level: 0, ..default() },
                          SpatialBundle { ..default() }));
            // flask gun
            parent.spawn((Cooldown::with_cooldown(5000),
                          Flask {},
                          Name::new("Molotov"),
                          AbilityLevel { level: 0, ..default() },
                          SpatialBundle { ..default() }));
            //move speed ability
            parent.spawn((PassiveMoveSpeedMultiplier {..default()},
                            Name::new("Move Speed"),
                          ParentMoveSpeedMultiplier { value: 0.0 },
                          AbilityLevel { level: 0, ..default() },
                          // SpatialBundle { ..default() },
                          ));
            //xp gatherer
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
            parent.spawn((AttackSpeed { percent: 100.0 },
            ));
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