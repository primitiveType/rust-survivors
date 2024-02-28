use bevy::prelude::{Commands, Entity, Query, Transform, With, Without};
use bevy_xpbd_2d::prelude::{CollidingEntities, LinearVelocity};

use crate::{Ball, Brick, DamageOnTouch, FollowPlayer, Health, MoveSpeed, Player};
use crate::extensions::vectors::to_vec2;


pub fn set_follower_velocity(
    mut query: Query<(&mut LinearVelocity, &MoveSpeed, &Transform), (With<FollowPlayer>, Without<Player>)>,
    player_query: Query<&mut Transform, With<Player>>,
) {
    let player = player_query.single();

    for (mut velocity, move_speed, transform) in query.iter_mut() {
        let direction = (player.translation - transform.translation).normalize();
        let new_velocity = direction * move_speed.value;
        velocity.0 = to_vec2(new_velocity);
    }
}

pub fn destroy_brick_on_collide(
    query: Query<(Entity, &CollidingEntities), With<Ball>>,
    bricks: Query<Entity, With<Brick>>,
    mut commands: Commands,
) {
    for (_entity, colliding_entities) in &query {
        for collision in colliding_entities.iter() {
            let brick_result = bricks.get(*collision);
            if let Ok(brick) = brick_result {
                commands.entity(brick).despawn();
            }
        }
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

pub fn log_paddle_collide(_query: Query<(Entity, &CollidingEntities)>) {
    return;
    // for (entity, colliding_entities) in &query {
    //     if colliding_entities.iter().count() > 0 {
    //         println!(
    //             "{:?} is colliding with the following entities: {:?}",
    //             entity,
    //             colliding_entities
    //         );
    //     }
    // }
}

pub fn player_picks_up_xp(mut query: Query<(&mut Health, &CollidingEntities), With<Player>>,
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