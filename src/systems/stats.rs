use bevy::asset::Assets;
use bevy::math::Vec3Swizzles;
use bevy::prelude::{ColorMaterial, Commands, Entity, Mesh, NextState, Query, ResMut, Transform};
use bevy_xpbd_2d::prelude::CollidingEntities;

use crate::AppState;
use crate::bundles::spawn_xp;
use crate::components::{Enemy, GainXPOnTouch, Health, Player};

pub fn die_at_zero_health(query: Query<(Entity, &Enemy, &Health, &Transform)>,
                          mut commands: Commands,
                          mut meshes: ResMut<Assets<Mesh>>,
                          mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, enemy, health, transform) in query.iter() {
        if health.value <= 0.0
        {
            commands.entity(entity).despawn();
            spawn_xp(&mut commands, &mut meshes, &mut materials, enemy.xp, transform.translation.xy());
        }
    }
}

pub fn pickup_xp(mut query: Query<(Entity, &mut Player, &CollidingEntities)>,
                 xps: Query<(Entity, &GainXPOnTouch)>,
                 mut commands: Commands,
                 mut next_state: ResMut<NextState<AppState>>,
) {
    for (_, mut player, collisions) in query.iter_mut() {
        for collision in collisions.iter() {
            if let Ok(xp) = xps.get(*collision) {
                player.xp = player.xp + xp.1.value;
                commands.entity(*collision).despawn();
                if player.xp / 2 > player.level {
                    next_state.set(AppState::LevelUp);
                }
            }
        }
    }
}