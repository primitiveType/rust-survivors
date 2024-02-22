use bevy::prelude::{Query, Res, Time, Transform, With, Without};
use crate::{FollowPlayer, MoveSpeed, Paddle, Velocity};
use crate::extensions::vectors::to_vec2;

pub fn set_follower_velocity(
    mut query: Query<(&mut Velocity, &MoveSpeed, &Transform), (With<FollowPlayer>, Without<Paddle>)>,
    player_query: Query<&mut Transform, With<Paddle>>,
    time: Res<Time>,
) {
    let player = player_query.single();

    for (mut velocity, move_speed, transform) in query.iter_mut(){
        let direction = (player.translation - transform.translation).normalize();
        let new_velocity = direction * move_speed.value;
        velocity.0 = to_vec2(new_velocity);
        println!("{}", velocity.0);

    }
}
