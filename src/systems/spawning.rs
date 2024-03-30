use bevy::math::Vec3Swizzles;
use bevy::prelude::{Commands, Query, Transform};
pub use bevy::utils::petgraph::visit::Walker;


use crate::bundles::{EnemySpawnData, Object};
use crate::components::{Enemy, Player};
use crate::extensions::spew_extensions::{Spawn, Spawner};

pub fn enemy_spawn_cycle(
    query: Query<&Enemy>,
    player_query: Query<(&Player, &Transform)>,
    _commands: Commands,
    mut spawner: Spawner<EnemySpawnData>,
) {
    let count = query.iter().len();
    let (player, transform) = player_query.single();

    
    if count < 200 {
        // spawn_enemy(1, _commands, atlases, rng);
        spawner.spawn(Object::Enemy, EnemySpawnData { enemy_num: 1, player_position: transform.translation.xy() });
        // spawner.send(SpawnEvent::with_data();
    }
}