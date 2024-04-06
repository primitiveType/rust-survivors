use std::default;
use std::fmt::Display;
use std::string::String;

use bevy::asset::Assets;
use bevy::core::Name;
use bevy::hierarchy::Parent;
use bevy::math::{Vec2, Vec3, Vec3Swizzles};
use bevy::prelude::{Changed, Color, ColorMaterial, Commands, default, Entity, EventReader, Mesh, NextState, Query, ResMut, Sprite, SpriteSheetBundle, Transform, With, Without};
use bevy_asepritesheet::animator::{AnimatedSpriteBundle, SpriteAnimator};
use bevy_asepritesheet::sprite::Spritesheet;
use bevy_egui::egui::debug_text::print;
use bevy_rapier2d::parry::transformation::utils::transform;
use bevy_rapier2d::pipeline::CollisionEvent;
use rand::Rng;

use crate::AppState;
use crate::bundles::{CorpseBundle, CorpseSpawnData, Object, spawn_xp, XPSpawnData};
use crate::components::{AbilityLevel, BaseMoveSpeed, Cold, Enemy, FireBallGun, Flask, FollowPlayer, GainXPOnTouch, Health, IceBallGun, Lifetime, MoveSpeed, ParentMoveSpeedMultiplier, PassiveMoveSpeedMultiplier, Player, XP, XPVacuum};
use crate::extensions::spew_extensions::{Spawn, Spawner};
use crate::systems::guns::{FireballSpawnData, FlaskSpawnData, IceballSpawnData, LevelableData, ParticleSpawnData};

pub fn die_at_zero_health(query: Query<(Entity, &Enemy, &Health, &Transform, &Name, &Sprite)>,
                          mut commands: Commands,
                          mut spawner: Spawner<CorpseSpawnData>,
                          mut xp_spawner: Spawner<XPSpawnData>,
) {
    for (entity, enemy, health, transform, name, sprite) in query.iter() {
        if health.value <= 0.0
        {
            let position = transform.translation.xy();
            spawner.spawn(Object::Corpse, CorpseSpawnData { name: name.to_string(), position, flip: sprite.flip_x });
            commands.entity(entity).despawn();
            xp_spawner.spawn(Object::XP, XPSpawnData { amount: enemy.xp, position })
        }
    }
}

pub fn update_move_speed_from_passive(mut abilities: Query<(&AbilityLevel, &PassiveMoveSpeedMultiplier, &mut ParentMoveSpeedMultiplier)>,
) {
    for (ability, _, mut parent_move_speed) in abilities.iter_mut() {
        parent_move_speed.value = 0.10 * ability.level as f32;
    }
}


pub fn cold_objects_are_blue(mut sprites: Query<&mut Sprite, With<Cold>>,
) {
    for mut sprite in sprites.iter_mut() {
        sprite.color = Color::rgba(0.5, 0.5, 1.0, 1.0);
    }
}

pub fn cold_enemies_spawn_particles(mut sprites: Query<(Entity, &Enemy), With<Cold>>, mut spawner: Spawner<ParticleSpawnData>,
) {
    for (entity, enemy) in sprites.iter() {
        let mut rng = rand::thread_rng();
        let value = rng.gen_range(0.0..1.0);
        let angle = value * 2.0 * std::f32::consts::PI;
        // Calculate the direction vector from the angle
        let mut direction = Vec2::new(angle.cos(), angle.sin());


        let distance = Vec2::splat(rng.gen_range(1.0..30.0));
        direction *= distance;
        spawner.spawn(
            Object::Particle,
            ParticleSpawnData {
                parent: Some(entity),
                sprite_sheet: "ice_particle".to_string(),
                color: Color::rgb(0.4, 0.5, 1.0),
                animation: "Idle".to_string(),
                lifetime: Lifetime::from_seconds(0.5),
                position: direction,
                scale: Vec2::new(0.3, 0.3),
            },
        )
    }
}

pub fn reset_enemy_color(mut sprites: Query<&mut Sprite, With<Enemy>>,
) {
    for mut sprite in sprites.iter_mut() {
        sprite.color = Color::default();
    }
}

pub fn move_speed_mod_affects_animation_speed(mut sprites: Query<(&mut SpriteAnimator, &MoveSpeed, &BaseMoveSpeed)>,
) {
    for (mut animator, move_speed, base_move) in sprites.iter_mut() {
        animator.time_scale = move_speed.value / base_move.value;
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
        push_stat_block(&mut description, "Damage", current_level.damage, next_level.damage);
        push_stat_block(&mut description, "Bullet Speed", current_level.bullet_speed, next_level.bullet_speed);
        push_stat_block(&mut description, "Size", current_level.bullet_size, next_level.bullet_size);
        push_stat_block(&mut description, "Pierce", current_level.pierce, next_level.pierce);

        ability.description = description;
    }
}

pub fn update_level_descriptions_iceball(mut abilities: Query<(&mut AbilityLevel, &IceBallGun), Changed<AbilityLevel>>,
) {
    for (mut ability, _) in abilities.iter_mut() {
        println!("Updating iceball description.");
        let current_level = IceballSpawnData::get_data_for_level(ability.level);
        let next_level = IceballSpawnData::get_data_for_level(ability.level + 1);
        let mut description = "Ice Spike".to_string();
        // ability.description = format!("Molotov Cocktail\r\nSize:\r\n{} -> {}\r\n Cooldown:\r\n{} -> {}", current_level.scale, next_level.scale, current_level.cooldown.display_seconds(), next_level.cooldown.timer.display_seconds()).to_string();
        push_stat_block(&mut description, "Slow duration", current_level.slow_seconds, next_level.slow_seconds);
        push_stat_block(&mut description, "Bullet Lifetime", current_level.bullet_lifetime_seconds, next_level.bullet_lifetime_seconds);
        push_stat_block(&mut description, "Damage", current_level.damage, next_level.damage);

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
        if xp.amount / 2 > player.level as u32 {
            next_state.set(AppState::LevelUp);
            player.level += 1;
        }
    }
}