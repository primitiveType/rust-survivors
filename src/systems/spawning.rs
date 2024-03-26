use bevy::prelude::{Commands, Query, ResMut};
pub use bevy::utils::petgraph::visit::Walker;
use bevy_prng::WyRand;
use bevy_rand::prelude::GlobalEntropy;

use crate::bundles::spawn_enemy;
use crate::components::Enemy;
use crate::initialization::load_prefabs::Atlases;

pub fn enemy_spawn_cycle(
    query: Query<&Enemy>,
    _commands: Commands,
    atlases: ResMut<Atlases>,
    rng: ResMut<GlobalEntropy<WyRand>>,
) {
    let mut count = query.iter().len();


    if count < 200 {
        spawn_enemy(1, _commands, atlases, rng);
    }
}