use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::prelude::{EventReader, KeyCode, Query, Res, Time, Transform, With, Without};
use bevy::utils::HashMap;
use bevy_ggrs::{LocalInputs, LocalPlayers, PlayerInputs};
use bevy_rapier2d::dynamics::Velocity;
use bevy_rapier2d::prelude::*;
// The first generic parameter, u8, is the input type: 4-directions + fire fits
// easily in a single byte
// The second parameter is the address type of peers: Matchbox' WebRtcSocket
// addresses are called `PeerId`s

use crate::components::{
    AbilityLevel, BaseMoveSpeed, Cold, FollowPlayer, MoveSpeed, ParentMoveSpeedMultiplier,
    PassiveXPMultiplier, Player, XPMultiplier, XPPickupRadius, XPVacuum,
};
use crate::Config;
use crate::systems::guns::LevelableData;

pub fn set_follower_velocity(
    query: Query<
        (&mut Velocity, &MoveSpeed, &Transform),
        (With<FollowPlayer>, Without<Player>),
    >,
    player_query: Query<&mut Transform, With<Player>>,
) {
    // let player = player_query.iter().choose_multiple(1).;
    //
    // for (mut velocity, move_speed, transform) in query.iter_mut() {
    //     let direction = (player.translation - transform.translation).normalize_or_zero();
    //
    //     let new_velocity = direction * move_speed.value;
    //
    //     velocity.linvel = to_vec2(new_velocity);
    // }
}

pub fn apply_xp_radius(
    mut modifier_query: Query<
        (Entity, &XPVacuum, &AbilityLevel, &mut Collider),
        Changed<AbilityLevel>,
    >,
    mut commands: Commands,
) {
    for (entity, vacuum, ability, collider) in modifier_query.iter_mut() {
        commands.entity(entity).insert(Collider::ball(
            XPPickupRadius::get_data_for_level(ability.level).radius,
        ));
    }
}

pub fn apply_xp_multiplier(
    mut modifier_query: Query<(&PassiveXPMultiplier, &AbilityLevel, &Parent), Changed<AbilityLevel>>,
    mut player_query: Query<(&mut XPMultiplier, &Player)>,
) {
    for (_, ability, parent) in modifier_query.iter_mut() {
        let (mut xp_multi, player) = player_query.get_mut(parent.get()).unwrap();
        xp_multi.value = XPMultiplier::get_data_for_level(ability.level).value;
    }
}

pub fn apply_move_speed_multiplier(
    mut parent_query: Query<(
        Entity,
        &mut MoveSpeed,
        &BaseMoveSpeed,
        Option<&Children>,
        Option<&mut Cold>,
    )>,
    modifier_query: Query<&ParentMoveSpeedMultiplier>,
    commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut move_speed, base_move, children_maybe, cold_maybe) in &mut parent_query {
        let mut multiplier = 1.0;

        if let Some(children) = children_maybe {
            for modifier in modifier_query.iter_many(children) {
                multiplier += modifier.value;
            }
        }

        if let Some(cold) = cold_maybe {
            multiplier -= cold.multiplier;
        }
        move_speed.value = base_move.value * multiplier;
    }
}

pub fn camera_follow(
    local_players: Res<LocalPlayers>,
    mut query: Query<(&mut Transform, &Camera2d), Without<Player>>,
    player_query: Query<(&Transform, &Player)>,
) {
    for (player_transform, player, ) in player_query.iter() {
        // only follow the local player
        if !local_players.0.contains(&player.handle) {
            continue;
        }
        for (mut transform, camera) in query.iter_mut() {
            transform.translation = player_transform.translation.xy().extend(transform.translation.z);
        }
    }
}

pub fn _debug_collisions(mut collision_events: EventReader<CollisionEvent>) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(collider1, collider2, flags) => {
                println!(
                    "Collision started between {:?} and {:?}",
                    collider1, collider2
                );
            }
            CollisionEvent::Stopped(collider1, collider2, flags) => {
                println!(
                    "Collision stopped between {:?} and {:?}",
                    collider1, collider2
                );
            }
        }
    }
}

const INPUT_UP: u8 = 1 << 0;
const INPUT_DOWN: u8 = 1 << 1;
const INPUT_LEFT: u8 = 1 << 2;
const INPUT_RIGHT: u8 = 1 << 3;
const INPUT_FIRE: u8 = 1 << 4;

pub fn read_local_inputs(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    local_players: Res<LocalPlayers>,
) {
    let mut local_inputs = HashMap::new();

    for handle in &local_players.0 {
        let mut input = 0u8;

        if keys.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
            input |= INPUT_UP;
        }
        if keys.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
            input |= INPUT_DOWN;
        }
        if keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
            input |= INPUT_LEFT
        }
        if keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
            input |= INPUT_RIGHT;
        }
        if keys.any_pressed([KeyCode::Space, KeyCode::Enter]) {
            input |= INPUT_FIRE;
        }

        local_inputs.insert(*handle, input);
    }

    commands.insert_resource(LocalInputs::<Config>(local_inputs));
}

pub fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &MoveSpeed, &Player)>,
    time: Res<Time>,
    inputs: Res<PlayerInputs<Config>>,
) {
    for (mut velocity, move_speed, player) in &mut query {
        let mut direction: Vec2 = Default::default();
        let (input, _) = inputs[player.handle];

        if input & INPUT_LEFT != 0 {
            direction.x -= 1.0;
        }

        if input & INPUT_RIGHT != 0 {
            direction.x += 1.0;
        }
        if input & INPUT_DOWN != 0 {
            direction.y -= 1.0;
            direction = direction.normalize();
        }

        if input & INPUT_UP != 0 {
            direction.y += 1.0;
            direction = direction.normalize();
        }

        // Calculate the new horizontal position based on player input
        let new_player_velocity: Vec2 = direction * move_speed.value;

        velocity.linvel = new_player_velocity;
    }
}
