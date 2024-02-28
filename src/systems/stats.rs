use bevy::asset::Assets;
use bevy::math::Vec3Swizzles;
use bevy::prelude::{ColorMaterial, Commands, Entity, Mesh, Query, ResMut, Transform, With, Without};
use bevy_xpbd_2d::prelude::CollidingEntities;


use crate::{Enemy, GainXPOnTouch, Health, Player};
use crate::bundles::spawn_xp;

pub fn die_at_zero_health(mut query: Query<(Entity, &Enemy, &Health, &Transform)>,
                          mut commands: Commands,
                          mut meshes: ResMut<Assets<Mesh>>,
                          mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, enemy, health, transform) in query.iter() {
        if health.value <= 0.0
        {
            commands.entity(entity).despawn();
            spawn_xp(&mut commands, &mut meshes, &mut materials, enemy.xp, transform.translation.xy());
            println!("entity died.");
        }
    }
}

pub fn pickup_xp(mut query: Query<(Entity, &mut Player, &CollidingEntities)>,
                 mut xps: Query<(Entity, &GainXPOnTouch)>,
                 mut commands: Commands,
) {
    for (_, mut player, collisions) in query.iter_mut() {
        for collision in collisions.iter() {
            if let Ok(xp) = xps.get(*collision) {
                player.xp = player.xp + xp.1.value;
                commands.entity(*collision).despawn();
            }
        }
    }
}