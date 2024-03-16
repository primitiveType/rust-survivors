use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::prelude::{Commands, KeyCode, Query, Res, Time, Transform, With, Without};
use bevy_xpbd_2d::prelude::{CollidingEntities, LinearVelocity};

use crate::components::{DamageOnTouch, FollowPlayer, Health, MoveSpeed, Player};
use crate::constants::{BOTTOM_WALL, LEFT_WALL, PADDLE_PADDING, PADDLE_SIZE, PADDLE_SPEED, RIGHT_WALL, TOP_WALL, WALL_THICKNESS};
use crate::extensions::vectors::to_vec2;

pub fn set_follower_velocity(
    mut query: Query<(&mut LinearVelocity, &MoveSpeed, &Transform), (With<FollowPlayer>, Without<Player>)>,
    player_query: Query<&mut Transform, With<Player>>,
) {
    let player = player_query.single();

    for (mut velocity, move_speed, transform) in query.iter_mut() {
        let direction = (player.translation - transform.translation).normalize_or_zero();

        let new_velocity = direction * move_speed.value;

        velocity.0 = to_vec2(new_velocity);
    }
}

pub fn player_takes_damage_from_enemy(mut query: Query<(&mut Health, &CollidingEntities), With<Player>>,
                                      damagers: Query<&DamageOnTouch>,
                                      mut _commands: Commands,
) {
    for (mut entity, colliding_entities) in query.iter_mut() {
        for colliding_entity in colliding_entities.iter() {
            let damager = damagers.get(*colliding_entity);
            if let Ok(damage) = damager {
                entity.value -= damage.value;
            }
        }
    }
}


pub fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut LinearVelocity, With<Player>>,
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
        direction * PADDLE_SPEED * time.delta_seconds();

    // Update the paddle position,
    // making sure it doesn't cause the paddle to leave the arena
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    let upper_bound = TOP_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.y / 2.0 + PADDLE_PADDING;
    let lower_bound = BOTTOM_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.y / 2.0 - PADDLE_PADDING;
    paddle_velocity.x = new_player_velocity.x.clamp(left_bound, right_bound);
    paddle_velocity.y = new_player_velocity.y.clamp(lower_bound, upper_bound);
}

