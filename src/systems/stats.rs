use bevy::asset::Assets;
use bevy::hierarchy::Parent;
use bevy::math::Vec3Swizzles;
use bevy::prelude::{ColorMaterial, Commands, Entity, Mesh, NextState, Query, ResMut, Transform};
use bevy_xpbd_2d::prelude::CollidingEntities;

use crate::AppState;
use crate::bundles::spawn_xp;
use crate::components::{Enemy, GainXPOnTouch, Health, Player, XP, XPVacuum};

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

pub fn pickup_xp(mut query: Query<(Entity, &mut XPVacuum, &Parent, &CollidingEntities)>,
                 xps: Query<(Entity, &GainXPOnTouch)>,
                 mut player_xps: Query<(&Player, &mut XP)>,
                 mut commands: Commands,
) {
    for (_, _player, parent, collisions) in query.iter_mut() {
        for collision in collisions.iter() {
            if let Ok(xp) = xps.get(*collision) {
                if let Ok((_, mut player_xp)) = player_xps.get_mut(parent.get()) {
                    commands.entity(*collision).despawn();
                    player_xp.amount = player_xp.amount + xp.1.value;
                }
            }
        }
    }
}

pub fn level_up(
    mut query: Query<(Entity, &mut Player, &XP)>,
    mut next_state: ResMut<NextState<AppState>>) {
    for (_, mut player, xp) in query.iter_mut() {
        if xp.amount / 2 > player.level {
            next_state.set(AppState::LevelUp);
            player.level = player.level + 1;
        }
    }
}