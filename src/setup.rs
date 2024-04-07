use bevy::math::vec3;
use bevy_ecs_ldtk::LdtkWorldBundle;
use bevy_rapier2d::geometry::{ActiveEvents, Collider, Restitution, Sensor};
use bevy_rapier2d::parry::transformation::utils::transform;
use bevy_rapier2d::prelude::{CollisionGroups};
use crate::*;
use crate::components::{Cooldown, AttackSpeed, FireBallGun, XPVacuum, Flask, AbilityLevel, ParentMoveSpeedMultiplier, PassiveMoveSpeedMultiplier, IceBallGun, Player, PassiveXPMultiplier, XPPickupRadius};
use crate::constants::{PIXEL_SCALE, SCORE_COLOR, SCOREBOARD_FONT_SIZE, SCOREBOARD_TEXT_PADDING, TEXT_COLOR};
use crate::physics::layers::game_layer;

// Add the game's entities to our world
// #[bevycheck::system]
pub fn setup(
    mut commands: Commands,
    atlases: ResMut<Atlases>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    let camera = commands.spawn(Camera2dBundle::default());

    // Sound
    // let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    // commands.insert_resource(CollisionSound(ball_collision_sound));
    // Get the specific entity you want

    spawn_player(&mut commands, atlases, Vec2::ZERO);

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("levels/cemetery-0/cemetery-0.ldtk"),
        transform: Transform::from_translation(Vec3::splat(0.0)).with_scale(Vec2::splat(PIXEL_SCALE).extend(1.0)),
        ..Default::default()
    });

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
                position: Vec2,
) {
    commands.spawn(bundles::PlayerBundle::with_sprite(atlases, position))

        .with_children(|parent| {
            //fireball gun
            parent.spawn((Cooldown::with_cooldown(500),
                          FireBallGun {},
                          Name::new("Fireball"),
                          AbilityLevel { level: 1, ..default() },
                          SpatialBundle { ..default() }));
            //iceball gun
            parent.spawn((Name::new("Snowball"),
                          Cooldown::with_cooldown(900),
                          IceBallGun {},
                          AbilityLevel { level: 0, ..default() },
                          SpatialBundle { ..default() }));
            // flask gun
            parent.spawn((Name::new("Molotov"),
                          Cooldown::with_cooldown(0),
                          Flask {},
                          AbilityLevel { level: 0, ..default() },
                          SpatialBundle { ..default() }));
            //move speed ability
            parent.spawn((Name::new("Move Speed"),
                          PassiveMoveSpeedMultiplier { ..default() },
                          ParentMoveSpeedMultiplier { value: 0.0 },
                          AbilityLevel { level: 0, ..default() },
                          // SpatialBundle { ..default() },
            ));
            parent.spawn((Name::new("XP Bonus"),
                          PassiveXPMultiplier {},
                          AbilityLevel { level: 0, ..default() },
                          // SpatialBundle { ..default() },
            ));
            //xp gatherer
            parent.spawn((Name::new("XP Pickup Radius"),
                          XPPickupRadius { radius: 0.0 },
                          XPVacuum {},
                          AbilityLevel { level: 0, ..default() },
                          Collider::ball(50.0),
                          // Friction::ZERO,
                          Restitution::new(1.0),
                          CollisionGroups::new(game_layer::XP, game_layer::XP),
                          // LockedAxes::ROTATION_LOCKED,
                          SpatialBundle { ..default() },
                          Sensor,
                          ActiveEvents::COLLISION_EVENTS,
            ));
            parent.spawn((AttackSpeed { percent: 0.0 },
            ));
        });
}

fn spawn_background(commands: &mut Commands, asset_server: &AssetServer) {}