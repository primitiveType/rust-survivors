use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::prelude::{EventReader, KeyCode, Query, Res, Time, Transform, With, Without};
use bevy::prelude::*;
use bevy_rapier2d::dynamics::Velocity;
use bevy_rapier2d::prelude::*;

use crate::components::{BaseMoveSpeed, FollowPlayer, MoveSpeed, ParentMoveSpeedMultiplier, Player};
use crate::extensions::vectors::to_vec2;

pub fn set_follower_velocity(
    mut query: Query<(&mut Velocity, &MoveSpeed, &Transform), (With<FollowPlayer>, Without<Player>)>,
    player_query: Query<&mut Transform, With<Player>>,
) {
    let player = player_query.single();

    for (mut velocity, move_speed, transform) in query.iter_mut() {
        let direction = (player.translation - transform.translation).normalize_or_zero();

        let new_velocity = direction * move_speed.value;

        velocity.linvel = to_vec2(new_velocity);
    }
}

pub fn apply_move_speed_multiplier(
    mut parent_query: Query<(&mut MoveSpeed, &BaseMoveSpeed, &Children)>,
    modifier_query: Query<(&ParentMoveSpeedMultiplier)>,
) {
    for (mut move_speed, base_move, children) in &mut parent_query {
        let mut multiplier = 1.0;
        for modifier in modifier_query.iter_many(children) {
            multiplier += modifier.value;
        }
        move_speed.value = base_move.value * multiplier;
    }
}

pub fn camera_follow(mut query: Query<(&mut Transform, &Camera2d), Without<Player>>,
                     player_query: Query<&mut Transform, With<Player>>,
) {
    let player = player_query.single();

    for (mut transform, camera) in query.iter_mut() {
        transform.translation = player.translation;
    }
}

pub fn _debug_collisions(
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(collider1, collider2, flags) => {
                println!("Collision started between {:?} and {:?}", collider1, collider2);
            }
            CollisionEvent::Stopped(collider1, collider2, flags) => {
                println!("Collision stopped between {:?} and {:?}", collider1, collider2);
            }
        }
    }
}

pub fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &MoveSpeed), With<Player>>,
    time: Res<Time>,
) {
    let (mut velocity, move_speed) = query.single_mut();
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


    // Calculate the new horizontal position based on player input
    let new_player_velocity: Vec2 =
        direction * move_speed.value;


    velocity.linvel = new_player_velocity;
}

