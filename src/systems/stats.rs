use std::fmt::Display;
use std::string::String;

use bevy::asset::Assets;
use bevy::math::Vec3Swizzles;
use bevy::prelude::{Changed, ColorMaterial, Commands, Entity, EventReader, Mesh, NextState, Query, ResMut, Transform};
use bevy_rapier2d::pipeline::CollisionEvent;

use crate::AppState;
use crate::bundles::spawn_xp;
use crate::components::{AbilityLevel, Enemy, FireBallGun, Flask, FollowPlayer, GainXPOnTouch, Health, MoveSpeed, ParentMoveSpeedMultiplier, PassiveMoveSpeedMultiplier, Player, XP, XPVacuum};
use crate::systems::guns::{FireballSpawnData, FlaskSpawnData, LevelableData};

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

pub fn update_move_speed_from_passive(mut abilities: Query<(&AbilityLevel, &PassiveMoveSpeedMultiplier, &mut ParentMoveSpeedMultiplier)>,
) {
    for (ability, _, mut parent_move_speed) in abilities.iter_mut() {
        parent_move_speed.value = 0.10 * ability.level as f32;
    }
}
/*
--level up--
Choices should be trackable, and it should be possible to view what choices were made in ui
- choices are children of player and have level component
Upgrade stats of player
- can react to add/remove, and mutate stats on player
add new weapon/ability
- on add/remove, add or remove weapon
increase stats of ability
- ability stats reference a table of level->value?
turn ability into new ability
- on add/remove (or level value change), remove ability and add new ability
 */

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
pub fn update_level_descriptions_move_speed(mut abilities: Query<(&mut AbilityLevel, &PassiveMoveSpeedMultiplier), Changed<AbilityLevel>>,
) {
    for (mut ability, _flask) in abilities.iter_mut() {
        let current_level = PassiveMoveSpeedMultiplier::get_data_for_level(ability.level);
        let next_level = PassiveMoveSpeedMultiplier::get_data_for_level(ability.level + 1);
        let mut description = "Upgrade Move Speed".to_string();

        push_stat_block(&mut description, "Move Speed", current_level.value, next_level.value);
        ability.description = description;
    }
}
pub fn update_level_descriptions_flask(mut abilities: Query<(&mut AbilityLevel, &Flask), Changed<AbilityLevel>>,
) {
    for (mut ability, _flask) in abilities.iter_mut() {
        println!("Updating flask description.");
        let current_level = FlaskSpawnData::get_data_for_level(ability.level);
        let next_level = FlaskSpawnData::get_data_for_level(ability.level + 1);
        let mut description = "Molotov Cocktail".to_string();
        // ability.description = format!("Molotov Cocktail\r\nSize:\r\n{} -> {}\r\n Cooldown:\r\n{} -> {}", current_level.scale, next_level.scale, current_level.cooldown.display_seconds(), next_level.cooldown.timer.display_seconds()).to_string();
        push_stat_block(&mut description, "Cooldown", current_level.cooldown.display_seconds(), next_level.cooldown.display_seconds());
        push_stat_block(&mut description, "Size", current_level.scale, next_level.scale);

        ability.description = description;
    }
}

pub fn update_level_descriptions_fireball(mut abilities: Query<(&mut AbilityLevel, &FireBallGun), Changed<AbilityLevel>>,
) {
    for (mut ability, _fireball) in abilities.iter_mut() {
        println!("Updating fireball description.");
        let current_level = FireballSpawnData::get_data_for_level(ability.level);
        let next_level = FireballSpawnData::get_data_for_level(ability.level + 1);
        let mut description = "Fireball".to_string();
        // ability.description = format!("Molotov Cocktail\r\nSize:\r\n{} -> {}\r\n Cooldown:\r\n{} -> {}", current_level.scale, next_level.scale, current_level.cooldown.display_seconds(), next_level.cooldown.timer.display_seconds()).to_string();
        push_stat_block(&mut description, "Bullet Speed", current_level.bullet_speed, next_level.bullet_speed);
        push_stat_block(&mut description, "Size", current_level.bullet_size, next_level.bullet_size);
        push_stat_block(&mut description, "Pierce", current_level.pierce, next_level.pierce);

        ability.description = description;
    }
}

pub fn push_stat_block(desc: &mut String, label: impl Display, value1: impl Display, value2: impl Display) {
    desc.push_str("\r\n");
    desc.push_str(format!("{label}:").as_str());
    desc.push_str("\r\n");
    desc.push_str(format!("{value1} --> {value2}").as_str());
}

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

                let xp = xp_query.get(*xp_entity);
                if xp.is_err() {
                    //other entity was not an xp
                    continue;
                }

                let (e_entity, _player, mut player_xp) = player.unwrap();
                player_xp.amount += xp.unwrap().1.value;
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

                let xp = xp_query.get(*xp_entity);
                if xp.is_err() {
                    //other entity was not an xp
                    continue;
                }

                let (_entity, vacuum) = vacuum.unwrap();
                let (xp_entity, xp) = xp.unwrap();
                commands.entity(xp_entity).insert((FollowPlayer, MoveSpeed { value: 500.0 }));
            }
            _ => {}
        }
    }
}


pub fn level_up(
    mut query: Query<(Entity, &mut Player, &XP)>,
    mut next_state: ResMut<NextState<AppState>>) {
    for (_, mut player, xp) in query.iter_mut() {
        if xp.amount / 2 > player.level {
            next_state.set(AppState::LevelUp);
            player.level += 1;
        }
    }
}