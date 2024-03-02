use bevy::asset::Assets;
use bevy::prelude::{ColorMaterial, Commands, Mesh, Query, ResMut};

pub use bevy::utils::petgraph::visit::{Walker};

use crate::components::Enemy;
use crate::bundles::spawn_enemy;

pub fn enemy_spawn_cycle(
    query: Query<&Enemy>,
    _commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    let count = query.iter().len();

    if count < 3 {
        spawn_enemy(_commands, meshes, materials);
    }
}