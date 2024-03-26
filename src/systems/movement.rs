use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::prelude::{Commands, EventReader, KeyCode, Query, Res, Time, Transform, With, Without};
use bevy_rapier2d::dynamics::Velocity;

use crate::components::{ DamageOnTouch, FollowPlayer, Health, MoveSpeed, Player};
use crate::constants::{BOTTOM_WALL, LEFT_WALL, PADDLE_PADDING, PADDLE_SIZE, PLAYER_SPEED, RIGHT_WALL, TOP_WALL, WALL_THICKNESS};
use crate::extensions::vectors::to_vec2;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
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

pub fn camera_follow(mut query : Query<(&mut Transform, &Camera2d), Without<Player>>,
                     player_query: Query<&mut Transform, With<Player>>,
){
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
    mut query: Query<&mut Velocity, With<Player>>,
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
        direction * PLAYER_SPEED;

    // Update the paddle position,
    // making sure it doesn't cause the paddle to leave the arena
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    let upper_bound = TOP_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.y / 2.0 + PADDLE_PADDING;
    let lower_bound = BOTTOM_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.y / 2.0 - PADDLE_PADDING;
    paddle_velocity.linvel.x = new_player_velocity.x.clamp(left_bound, right_bound);
    paddle_velocity.linvel.y = new_player_velocity.y.clamp(lower_bound, upper_bound);
}

