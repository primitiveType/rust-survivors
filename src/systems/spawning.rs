use bevy::asset::{Assets, Handle};
use bevy::math::{Vec3, Vec3Swizzles};
use bevy::prelude::{
    Added, Color, Commands, Component, Entity, Gizmos, Query, Res, ResMut, Resource, Time, Timer,
    Transform, Vec2, Without,
};
pub use bevy::utils::petgraph::visit::Walker;
use bevy_ecs_ldtk::prelude::{LdtkProject, LevelMetadataAccessor};
use bevy_ecs_ldtk::*;

use crate::bundles::{EnemySpawnData, Object, PlayerSpawn};
use crate::components::{Enemy, Player};
use crate::constants::{PIXEL_SCALE, PLAYER_LAYER};
use crate::extensions::spew_extensions::{Spawn, Spawner};

#[derive(Resource)]
pub struct RoundTimer {
    pub timer: Timer,
}

#[derive(Component, Clone, Copy)]
pub struct LevelBounds {
    pub min: Vec2,
    pub max: Vec2,
}

pub fn set_level_bounds(
    levels: Query<(Entity, &Handle<LdtkProject>, &Transform), Added<LevelSet>>,
    ldtk_assets: Res<Assets<LdtkProject>>,
    mut commands: Commands,
) {
    for (project_entity, ldtk_handle, transform) in levels.iter() {
        if let Some(ldtk_asset) = ldtk_assets.get(ldtk_handle) {
            let level = &ldtk_asset
                .data()
                .find_raw_level_by_level_selection(&LevelSelection::index(1))
                .unwrap(); // Access the first level, adjust if needed

            let level_width_meters = level.px_wid as f32 * PIXEL_SCALE; // Convert pixels to meters if necessary
            let level_height_meters = level.px_hei as f32 * PIXEL_SCALE;

            let padding = 32f32 * PIXEL_SCALE * 2.0; // I honestly don't understand why I have to double it
            commands.entity(project_entity).insert(LevelBounds {
                min: Vec2::new(
                    transform.translation.x + padding,
                    transform.translation.y + padding,
                ),
                max: Vec2::new(
                    level_width_meters + transform.translation.x - padding,
                    level_height_meters + transform.translation.y - padding,
                ),
            });
        }
    }
}

pub fn draw_level_bounds(mut gizmos: Gizmos, bounds_query: Query<&LevelBounds>) {
    for bound in bounds_query.iter() {
        let size = bound.max - bound.min;
        gizmos.rect_2d(bound.min + (size * 0.5), 0.0, bound.max, Color::CYAN);
    }
}

pub fn enemy_spawn_cycle(
    query: Query<&Enemy>,
    player_query: Query<(&Player, &Transform)>,
    bounds_query: Query<&LevelBounds>,
    _commands: Commands,
    mut spawner: Spawner<EnemySpawnData>,
    time: Res<Time>,
    mut round_time: ResMut<RoundTimer>,
) {
    round_time.timer.tick(time.delta());
    if round_time.timer.finished() || bounds_query.is_empty() {
        return;
    }
    let count = query.iter().len();
    let bounds = bounds_query.single();
    let total_translation: Vec3 = player_query
        .iter()
        .map(|(_player, transform)| transform.translation)
        .sum();
    let avg_translation: Vec3 = total_translation / Vec3::splat(player_query.iter().len() as f32);
    // let (map_size, tile_size, map_transform) = level_query.iter().next();
    if count < 5 {
        //round_time.timer.elapsed().as_secs() as usize {
        // spawn_enemy(1, _commands, atlases, rng);
        spawner.spawn(
            Object::Enemy,
            EnemySpawnData {
                enemy_id: "bat".to_string(),
                player_position: avg_translation.xy(),
                bounds: *bounds,
            },
        );
        spawner.spawn(
            Object::Enemy,
            EnemySpawnData {
                enemy_id: "zombie".to_string(),
                player_position: avg_translation.xy(),
                bounds: *bounds,
            },
        );

        // spawner.send(SpawnEvent::with_data();
    }
}

pub fn move_player_to_spawn_point(
    mut commands: Commands,
    spawn_point: Query<(Entity, &PlayerSpawn, &Transform), Without<Player>>,
    mut player_query: Query<(&Player, &mut Transform)>,
) {
    for (_player, mut transform) in player_query.iter_mut() {
        for (entity, _, spawn) in spawn_point.iter() {
            transform.translation =
                Vec2::new(spawn.translation.x, spawn.translation.y).extend(PLAYER_LAYER);
            commands.entity(entity).despawn();
        }
    }
}
