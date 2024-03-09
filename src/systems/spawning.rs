use bevy::asset::AssetServer;
use bevy::prelude::{Commands, Query, Res, ResMut};
pub use bevy::utils::petgraph::visit::Walker;

use crate::bundles::{Handles, spawn_enemy};
use crate::components::Enemy;
use crate::initialization::load_prefabs::Atlases;

pub fn enemy_spawn_cycle(
    query: Query<&Enemy>,
    _commands: Commands,
    asset_server: Res<AssetServer>,
    handles: Query<&Handles>,
    mut atlases: ResMut<Atlases>,
) {
    let count = query.iter().len();

    if count < 3 {
        spawn_enemy(count, _commands, asset_server, handles, atlases);
    }
}