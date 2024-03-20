use bevy::asset::AssetServer;
use bevy::prelude::{Commands, Query, Res, ResMut};
pub use bevy::utils::petgraph::visit::Walker;
use bevy_prng::WyRand;
use bevy_rand::prelude::GlobalEntropy;

use crate::bundles::spawn_enemy;
use crate::components::Enemy;
use crate::initialization::load_prefabs::Atlases;

pub fn enemy_spawn_cycle(
    query: Query<&Enemy>,
    _commands: Commands,
    asset_server: Res<AssetServer>,
    atlases: ResMut<Atlases>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    let count = query.iter().len();


    if count < 200 {
        println!("Spawning.");
        spawn_enemy(1, _commands, asset_server, atlases, rng);
    }
}