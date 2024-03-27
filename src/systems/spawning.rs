use bevy::prelude::{Commands, EventWriter, Query, ResMut, Transform};
pub use bevy::utils::petgraph::visit::Walker;
use bevy_prng::WyRand;
use bevy_rand::prelude::GlobalEntropy;
use spew::prelude::SpawnEvent;

use crate::bundles::{EnemySpawnData, Object, spawn_enemy};
use crate::components::Enemy;
use crate::initialization::load_prefabs::Atlases;

pub fn enemy_spawn_cycle(
    query: Query<&Enemy>,
    _commands: Commands,
    mut spawner: EventWriter<SpawnEvent<Object, EnemySpawnData>>,
) {
    let mut count = query.iter().len();


    if count < 200 {
        // spawn_enemy(1, _commands, atlases, rng);
        spawner.send(SpawnEvent::with_data(Object::Enemy, EnemySpawnData { enemy_num: 1 }));
    }
}