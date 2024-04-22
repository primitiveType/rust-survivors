use crate::components::{
    AbilityLevel, AttackSpeed, Cooldown, FireBallGun, Flask, IceBallGun, ParentMoveSpeedMultiplier,
    PassiveMoveSpeedMultiplier, PassiveXPMultiplier, XPPickupRadius, XPVacuum,
};
use crate::constants::{
    PIXEL_SCALE, SCOREBOARD_FONT_SIZE, SCOREBOARD_TEXT_PADDING, SCORE_COLOR, TEXT_COLOR,
};
use crate::physics::layers::game_layer;
use crate::*;
use bevy_ecs_ldtk::LdtkWorldBundle;
use bevy_ggrs::{ggrs, AddRollbackCommandExtension};
use bevy_matchbox::matchbox_socket::SingleChannel;
use bevy_matchbox::MatchboxSocket;
use bevy_rapier2d::geometry::{ActiveEvents, Collider, Restitution, Sensor};
use bevy_rapier2d::prelude::CollisionGroups;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use random::*;
use rand::{Rng, RngCore};

pub(crate) fn wait_for_players(
    mut commands: Commands,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if socket.get_channel(0).is_err() {
        return; // we've already started
    }

    // Check for new connections
    socket.update_peers();
    let players = socket.players();

    let num_players = 2;
    if players.len() < num_players {
        return; // wait for more players
    }

    info!("All peers have joined, going in-game");
    // create a GGRS P2P session
    let mut session_builder = ggrs::SessionBuilder::<Config>::new()
        .with_num_players(num_players)
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    // determine the seed
    let id = socket.id().expect("no peer id assigned").0.as_u64_pair();
    let mut seed = id.0 ^ id.1;
    for peer in socket.connected_peers() {
        let peer_id = peer.0.as_u64_pair();
        seed ^= peer_id.0 ^ peer_id.1;
    }
    let session_seed = (0);
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(session_seed);
    rng.gen_range(0..10);
    commands.insert_resource(SessionRng { rng });

    // move the channel out of the socket (required because GGRS takes ownership of it)
    let channel = socket.take_channel(0).unwrap();

    // start the GGRS session
    let ggrs_session = session_builder
        .start_p2p_session(channel)
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));

    next_state.set(AppState::InGame);
}

// Add the game's entities to our world
// #[bevycheck::system]
pub fn setup(mut commands: Commands, atlases: ResMut<Atlases>, asset_server: Res<AssetServer>) {
    // Camera
    let _camera = commands.spawn(Camera2dBundle::default());

    //start matchbox
    let room_url = "ws://127.0.0.1:3536/extreme_bevy?next=2";
    info!("connecting to matchbox server: {room_url}");
    commands.insert_resource(MatchboxSocket::new_ggrs(room_url));
    // Sound
    // let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    // commands.insert_resource(CollisionSound(ball_collision_sound));
    // Get the specific entity you want

    spawn_player(&mut commands, &atlases, Vec2::ZERO, 0);
    spawn_player(&mut commands, &atlases, Vec2::ONE, 1);

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("levels/cemetery-0/cemetery-0.ldtk"),
        transform: Transform::from_translation(Vec3::splat(0.0))
            .with_scale(Vec2::splat(PIXEL_SCALE).extend(1.0)),
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

fn spawn_player(commands: &mut Commands, atlases: &ResMut<Atlases>, position: Vec2, handle: usize) {
    commands
        .spawn(bundles::PlayerBundle::with_sprite(
            atlases, position, handle,
        ))
        .with_children(|parent| {
            //fireball gun
            parent.spawn((
                Cooldown::with_cooldown(500),
                FireBallGun {},
                Name::new("Fireball"),
                AbilityLevel {
                    level: 1,
                    ..default()
                },
                SpatialBundle { ..default() },
            ));
            //iceball gun
            parent.spawn((
                Name::new("Snowball"),
                Cooldown::with_cooldown(900),
                IceBallGun {},
                AbilityLevel {
                    level: 0,
                    ..default()
                },
                SpatialBundle { ..default() },
            ));
            // flask gun
            parent.spawn((
                Name::new("Molotov"),
                Cooldown::with_cooldown(0),
                Flask {},
                AbilityLevel {
                    level: 1,
                    ..default()
                },
                SpatialBundle { ..default() },
            ));
            //move speed ability
            parent.spawn((
                Name::new("Move Speed"),
                PassiveMoveSpeedMultiplier { ..default() },
                ParentMoveSpeedMultiplier { value: 0.0 },
                AbilityLevel {
                    level: 0,
                    ..default()
                },
                // SpatialBundle { ..default() },
            ));
            parent.spawn((
                Name::new("XP Bonus"),
                PassiveXPMultiplier {},
                AbilityLevel {
                    level: 0,
                    ..default()
                },
                // SpatialBundle { ..default() },
            ));
            //xp gatherer
            parent
                .spawn((
                    Name::new("XP Pickup Radius"),
                    XPPickupRadius { radius: 0.0 },
                    XPVacuum {},
                    AbilityLevel {
                        level: 0,
                        ..default()
                    },
                    Collider::ball(50.0),
                    // Friction::ZERO,
                    Restitution::new(1.0),
                    CollisionGroups::new(game_layer::XP_ABSORB, game_layer::XP),
                    // LockedAxes::ROTATION_LOCKED,
                    SpatialBundle { ..default() },
                    Sensor,
                    ActiveEvents::COLLISION_EVENTS,
                ));

            parent.spawn((AttackSpeed { percent: 0.0 }, ));
        }).add_rollback();
}

