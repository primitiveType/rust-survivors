use bevy::asset::Assets;
use bevy::math::Vec3Swizzles;
use bevy::prelude::{ColorMaterial, Commands, Entity, EventReader, Mesh, NextState, Query, ResMut, Transform, Vec2};
use bevy_rapier2d::pipeline::CollisionEvent;
use bevy_rapier2d::prelude::Velocity;

use crate::AppState;
use crate::bundles::spawn_xp;
use crate::components::{Enemy, FollowPlayer, GainXPOnTouch, Health, MoveSpeed, Player, XP, XPVacuum};

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

// pub fn pickup_xp(mut query: Query<(Entity, &mut XPVacuum, &Parent, &CollidingEntities)>,
//                  xps: Query<(Entity, &GainXPOnTouch)>,
//                  mut player_xps: Query<(&Player, &mut XP)>,
//                  mut commands: Commands,
// ) {
//     for (_, _player, parent, collisions) in query.iter_mut() {
//         for collision in collisions.iter() {
//             if let Ok(xp) = xps.get(*collision) {
//                 if let Ok((_, mut player_xp)) = player_xps.get_mut(parent.get()) {
//                     commands.entity(*collision).despawn();
//                     player_xp.amount = player_xp.amount + xp.1.value;
//                 }
//             }
//         }
//     }
// }

pub fn pick_up_xp_on_touch(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut player_query: Query<(Entity, &Player, &mut XP)>,
    xp_query: Query<(Entity, &GainXPOnTouch)>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _flags) => {
                //assume player is first entity...
                let (mut player_entity, mut xp_entity) = (entity1, entity2);
                //check if it is
                let mut player = player_query.get_mut(*player_entity);
                if player.is_err() {
                    //ok, maybe it is the second entity.
                    //swap references now that we know which is which
                    (xp_entity, player_entity) = (player_entity, xp_entity);
                    player = player_query.get_mut(*player_entity);
                    //neither entity was a player
                    if player.is_err() {
                        continue;
                    }
                }

                let mut xp = xp_query.get(*xp_entity);
                if xp.is_err() {
                    //other entity was not an xp
                    continue;
                }

                let (e_entity, _player, mut player_xp) = player.unwrap();
                player_xp.amount = player_xp.amount + xp.unwrap().1.value;
                commands.entity(*xp_entity).despawn();
            }
            _ => {}
        }
    }
}

pub fn vacuum_xp_on_touch(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut vacuum_query: Query<(Entity, &XPVacuum)>,
    xp_query: Query<(Entity, &GainXPOnTouch)>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _flags) => {
                //assume vacuum is first entity...
                let (mut vacuum_entity, mut xp_entity) = (entity1, entity2);
                //check if it is
                let mut vacuum = vacuum_query.get_mut(*vacuum_entity);
                if vacuum.is_err() {
                    //ok, maybe it is the second entity.
                    //swap references now that we know which is which
                    (xp_entity, vacuum_entity) = (vacuum_entity, xp_entity);
                    vacuum = vacuum_query.get_mut(*vacuum_entity);
                    //neither entity was a player
                    if vacuum.is_err() {
                        continue;
                    }
                }

                let mut xp = xp_query.get(*xp_entity);
                if xp.is_err() {
                    //other entity was not an xp
                    continue;
                }

                let (_entity, vacuum) = vacuum.unwrap();
                let (xp_entity, xp) = xp.unwrap();
                commands.entity(xp_entity).insert((FollowPlayer, MoveSpeed{value: 500.0}));
            }
            _ => {}
        }
    }
}


pub fn level_up(
    mut query: Query<(Entity, &mut Player, &XP)>,
    mut next_state: ResMut<NextState<AppState>>) {
    for (_, mut player, xp) in query.iter_mut() {
        // if xp.amount / 2 > player.level {
        //     next_state.set(AppState::LevelUp);
        //     player.level = player.level + 1;
        // }
    }
}