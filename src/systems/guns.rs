use std::time::Duration;

use bevy::asset::{Assets, Handle};
use bevy::ecs::query::QueryEntityError;
use bevy::math::{Vec3, Vec3Swizzles};
use bevy::prelude::{Commands, default, Entity, EventReader, GlobalTransform, Mut, Query, Res, SpriteSheetBundle, Time, Transform, Vec2, With};
use bevy_asepritesheet::animator::{AnimatedSpriteBundle, AnimFinishEvent, SpriteAnimator};
use bevy_asepritesheet::prelude::{AnimEventSender, AnimHandle, Spritesheet};
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::geometry::{Collider, CollisionGroups, Restitution};
use bevy_rapier2d::parry::transformation::utils::transform;
use bevy_rapier2d::pipeline::CollisionEvent;
use bevy_rapier2d::plugin::RapierContext;
use bevy_rapier2d::prelude::{Group, QueryFilter, Velocity};
use rand::Rng;

use crate::bundles::PhysicalBundle;
use crate::components::{Cooldown, Bullet, BulletBundle, DamageOnTouch, Enemy, FireBallGun, Health, AttackSpeed, Flask, FlaskProjectileBundle, Lifetime, Expired};
use crate::extensions::vectors::to_vec2;
use crate::initialization::load_prefabs::Atlases;
use crate::Name;
use crate::physics::layers::game_layer;

pub fn advance_cooldowns(
    mut query: Query<&mut Cooldown>,
    mut cdr_query: Query<&mut AttackSpeed>,
    time: Res<Time>,
) {//assumes only player needs concept of abilities and CDR, which might change.
    for (mut ability) in query.iter_mut() {
        let mut total_cdr = 100.0f32;
        for (cdr) in cdr_query.iter_mut() {
            total_cdr = total_cdr + cdr.percent;
        }

        let multiplier = total_cdr * 0.01f32;//convert percentage to multiplier
        println!("{}", multiplier);
        let delta_seconds = time.delta().as_secs_f32();
        println!("Delta seconds: {}", delta_seconds);
        let multiplied_delta = (delta_seconds * multiplier);
        println!("Delta seconds times multiplier: {}", multiplied_delta);

        let duration = Duration::from_secs_f32(multiplied_delta);
        println!("{:?}", duration.as_millis());
        //this idea of advancing the timer will make less sense if we
        //display the timer for the user. If that happens, we will have to
        //track the timer duration and update it based on stats when they change.
        ability.timer.tick(duration);
    }
}

pub fn flask_weapon(
    mut commands: Commands,
    mut query: Query<(&mut Cooldown, &GlobalTransform, &Flask)>,
    atlases: Res<Atlases>,
) {
    for (mut ability, transform, flask) in query.iter_mut() {
        if ability.timer.just_finished() {
            let translation = transform.translation();

            let mut rng = rand::thread_rng();
            let value = rng.gen_range(0.0..1.0);
            let angle = value * 2.0 * std::f32::consts::PI;
            // Calculate the direction vector from the angle
            let mut direction = Vec2::new(angle.cos(), angle.sin());
            
            
            let distance = Vec2::splat(rng.gen_range(50.0..400.0));
            direction = direction * distance;
            
            spawn_flask_projectile(&mut commands, flask, direction, &atlases);
        }
    }
}


pub fn fireball_gun(
    mut commands: Commands,
    mut query: Query<(&mut Cooldown, &GlobalTransform, &FireBallGun)>,
    rapier_context: Res<RapierContext>,
    atlases: Res<Atlases>,
) {
    for (mut ability, transform, gun) in query.iter_mut() {
        if ability.timer.just_finished() {
            let translation = transform.translation();
            if let Some((entity, projection)) = rapier_context.project_point(
                to_vec2(translation),
                true,
                QueryFilter {
                    flags: Default::default(),
                    groups: Some(CollisionGroups::new(game_layer::ENEMY, game_layer::ENEMY)),//is this filter correct?
                    exclude_collider: None,
                    exclude_rigid_body: None,
                    predicate: None,
                },
            ) {
                // The collider closest to the point has this `handle`.
                // println!("Projected point on entity {:?}. Point projection: {}", entity, projection.point);
                // println!("Point was inside of the collider shape: {}", projection.is_inside);

                let mut delta = projection.point - to_vec2(translation);
                delta = delta.normalize();

                spawn_fireball(&mut commands, &gun, translation, delta, &atlases);
            }
        }
    }
}

pub fn deal_damage_on_collide(
    mut collision_events: EventReader<CollisionEvent>,
    mut health_query: Query<(Entity, &mut Health)>,
    enemy_query: Query<(Entity, &Enemy)>,//HACK do something smarter.
    mut damage_query: Query<(Entity, &mut DamageOnTouch)>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _flags) => {
                {//entity 2 damages entity 1 if it can
                    let entity1_health = health_query.get_mut(*entity1);
                    let entity2_damage = damage_query.get_mut(*entity2);

                    let enemy1 = enemy_query.get(*entity1);
                    let enemy2 = enemy_query.get(*entity2);
                    if enemy1.is_ok() && enemy2.is_ok() {
                        continue;
                    }
                    try_deal_damage(entity2_damage, entity1_health);
                }
                {//entity 1 damages entity 2 if it can
                    let entity2_health = health_query.get_mut(*entity2);
                    let entity1_damage = damage_query.get_mut(*entity1);
                    try_deal_damage(entity1_damage, entity2_health);
                }
            }
            _ => {}
        }
    }
}

fn try_deal_damage(entity1_damage: Result<(Entity, Mut<DamageOnTouch>), QueryEntityError>, entity2_health: Result<(Entity, Mut<Health>), QueryEntityError>) {
    match (entity1_damage, entity2_health) {
        (Ok((_, mut damage)), Ok((_, mut health))) => {
            health.value = health.value - damage.value;
            damage.count_triggers = damage.count_triggers + 1;
        }
        _ => {}
    }
}


pub fn expire_bullets_on_hit(mut bullets: Query<(&mut Bullet, Entity, &Transform, &DamageOnTouch)>,
                             mut commands: Commands,
) {
    for (bullet, entity, transform, damage) in bullets.iter_mut() {
        if damage.count_triggers > bullet.pierce.into()
        {
            commands.entity(entity).insert(Expired {});
        }
    }
}

pub fn expired_bullets_explode(mut bullets: Query<(Entity, &Bullet, &Transform), With<Expired>>,
                               mut commands: Commands,
                               atlases: Res<Atlases>,
                               sprite_assets: Res<Assets<Spritesheet>>,) {
    for (bullet, entity, transform) in bullets.iter_mut() {
        spawn_particle(transform.translation, &mut commands, "bullet".to_string(), FIREBALL_EXPLODE_ANIMATION, &atlases, &sprite_assets);
    }
}

pub fn expire_entities(mut lifetimes: Query<(Entity, &mut Lifetime)>,
                       mut commands: Commands,
                       time: Res<Time>) {
    for (entity, mut lifetime) in lifetimes.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).insert(Expired {});
        }
    }
}

pub fn destroy_expired_entities(mut lifetimes: Query<(Entity, &Expired)>,
                                mut commands: Commands,
) {
    for (entity, _) in lifetimes.iter_mut() {
        commands.entity(entity).despawn();
    }
}

const FIREBALL_EXPLODE_ANIMATION: &'static str = "Fireball_explode";

fn spawn_particle(position: Vec3, commands: &mut Commands, sprite_sheet: String, animation: &str, atlases: &Res<Atlases>, sprite_assets: &Res<Assets<Spritesheet>>) {
    let spritesheet = atlases.sprite_sheets.get(&sprite_sheet).expect("failed to find explode animation!").clone();
    let mut anim_handle = AnimHandle::from_index(0);
    // Attempt to get the asset using the handle
    if let Some(asset) = sprite_assets.get(&spritesheet) {
        // Now you have access to the asset (`T`) here
        // Do something with the asset
        anim_handle = asset.get_anim_handle(animation);
    } else {
        // The asset is not loaded yet, you might handle this case accordingly
        println!("Asset not loaded yet");
    }
    commands.spawn((
        AnimEventSender,
        AnimatedSpriteBundle {
            animator: SpriteAnimator::from_anim(anim_handle),
            spritesheet,
            sprite_bundle: SpriteSheetBundle
            {
                transform: Transform::from_translation(position),
                ..default()
            },

            ..Default::default()
        }));
}

pub fn destroy_explosions(
    mut commands: Commands,
    mut events: EventReader<AnimFinishEvent>,
    spritesheet_assets: Res<Assets<Spritesheet>>,
    animated_sprite_query: Query<&Handle<Spritesheet>, With<SpriteAnimator>>,
) {
    for event in events.read() {
        // get the spritesheet handle off the animated sprite entity
        if let Ok(sheet_handle) = animated_sprite_query.get(event.entity) {
            if let Some(anim_sheet) = spritesheet_assets.get(sheet_handle) {
                // get the animation reference from the spritesheet
                if let Ok(anim) = anim_sheet.get_anim(&event.anim) {
                    if anim.name == FIREBALL_EXPLODE_ANIMATION {
                        commands.entity(event.entity).despawn();
                    }
                }
            }
        }
    }
}

pub const BACKGROUND_PROJECTILE_LAYER: f32 = -1.0;
fn spawn_flask_projectile(
    commands: &mut Commands,
    gun: &Flask,
    position: Vec2,
    atlases: &Res<Atlases>,
) {
    let size = gun.bullet_size;
    let bundle = FlaskProjectileBundle {
        sprite_sheet: AnimatedSpriteBundle {
            animator: SpriteAnimator::from_anim(AnimHandle::from_index(0)),
            spritesheet: atlases.sprite_sheets.get("bullet").expect("failed to find asset for bullet!").clone(),
            sprite_bundle: SpriteSheetBundle {
                transform: Transform::from_translation(position.extend(BACKGROUND_PROJECTILE_LAYER)).with_scale(Vec2::splat(10.0).extend(1.0)),
                ..default()
            },

            ..Default::default()
        },
        physical: PhysicalBundle {
            collider: Collider::ball(
                10.0),
            restitution: Restitution::new(1.0),
            velocity: Velocity { linvel: Vec2::ZERO, angvel: 0.0 },
            collision_layers: CollisionGroups::new(game_layer::PLAYER, Group::from(game_layer::GROUND | game_layer::ENEMY)),
            rigid_body: RigidBody::Dynamic,

            ..default()
        },
        name: Name::new("flask"),
        sensor: Default::default(),
        damage: DamageOnTouch { value: 100.0 , ..default() },
        lifetime: Lifetime::from_seconds(2.0),
    };
    commands.spawn(bundle);
}

fn spawn_fireball(
    commands: &mut Commands,
    gun: &FireBallGun,
    position: Vec3,
    direction: Vec2,
    atlases: &Res<Atlases>,
) {
    let speed = gun.bullet_speed;
    let bundle = BulletBundle {
        sprite_sheet: AnimatedSpriteBundle {
            animator: SpriteAnimator::from_anim(AnimHandle::from_index(0)),
            spritesheet: atlases.sprite_sheets.get("bullet").expect("failed to find asset for bullet!").clone(),
            sprite_bundle: SpriteSheetBundle {
                transform: Transform::from_translation(position).with_scale(Vec3::new(2.0, 2.0, 0.0)),
                ..default()
            },

            ..Default::default()
        },
        physical: PhysicalBundle {
            collider: Collider::ball(
                0.5),
            restitution: Restitution::new(1.0),
            velocity: Velocity { linvel: direction * speed, angvel: 0.0 },
            collision_layers: CollisionGroups::new(game_layer::PLAYER, Group::from(game_layer::GROUND | game_layer::ENEMY)),
            rigid_body: RigidBody::Dynamic,

            ..default()
        },
        bullet: Bullet
        { pierce: gun.pierce, ..default() },

        name: Name::new("bullet"),
        sensor: Default::default(),
        damage: DamageOnTouch { value: 5.0 , ..default()},
        lifetime: Lifetime::from_seconds(2.0),
    };
    commands.spawn(bundle);
}