use bevy::prelude::{Commands, Entity, Query, Res, ResMut, Time, Transform, With, Without};
use bevy_xpbd_2d::prelude::{CollidingEntities, LinearVelocity};
use crate::{Ball, Brick, FollowPlayer, MoveSpeed, Paddle, Scoreboard};
use crate::extensions::vectors::to_vec2;

pub fn set_follower_velocity(
    mut query: Query<(&mut LinearVelocity, &MoveSpeed, &Transform), (With<FollowPlayer>, Without<Paddle>)>,
    player_query: Query<&mut Transform, With<Paddle>>,
    time: Res<Time>,
) {
    let player = player_query.single();

    for (mut velocity, move_speed, transform) in query.iter_mut() {
        let direction = (player.translation - transform.translation).normalize();
        let new_velocity = direction * move_speed.value;
        velocity.0 = to_vec2(new_velocity);
    }
}

pub fn destroy_brick_on_collide(query: Query<(Entity, &CollidingEntities), With<Ball>>, bricks: Query<Entity, With<Brick>>, mut commands: Commands,
                                mut scoreboard: ResMut<Scoreboard>) {
    for (entity, colliding_entities) in &query {
        for collision in colliding_entities.iter() {
            let brick_result = bricks.get(*collision);
            if let Ok(brick) = brick_result {
                scoreboard.score += 1;
                commands.entity(brick).despawn();
            }
        }
    }
}

pub fn log_paddle_collide(query: Query<(Entity, &CollidingEntities)>) {
    return;
    for (entity, colliding_entities) in &query {
        if (colliding_entities.iter().count() > 0) {
            println!(
                "{:?} is colliding with the following entities: {:?}",
                entity,
                colliding_entities
            );
        }
    }
}