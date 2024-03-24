use std::time::Duration;

use bevy::asset::{Assets, Handle};
use bevy::ecs::query::QueryEntityError;
use bevy::math::Vec3;
use bevy::prelude::{Commands, default, Entity, EventReader, GlobalTransform, Mut, Or, Query, Res, SpriteSheetBundle, Time, Transform, Vec2, With};
use bevy::time::Timer;
use bevy::time::TimerMode::Once;
use bevy_asepritesheet::animator::{AnimatedSpriteBundle, AnimFinishEvent, SpriteAnimator};
use bevy_asepritesheet::prelude::{AnimEventSender, AnimHandle, Spritesheet};
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::geometry::{ActiveEvents, Collider, CollisionGroups, Restitution};
use bevy_rapier2d::na::point;
use bevy_rapier2d::pipeline::CollisionEvent;
use bevy_rapier2d::plugin::RapierContext;
use bevy_rapier2d::prelude::{Group, QueryFilter, Velocity};
use crate::bundles::PhysicalBundle;

use crate::components::{Bullet, BulletBundle, DamageOnTouch, Enemy, Gun, Health, Player};
use crate::extensions::vectors::to_vec2;
use crate::initialization::load_prefabs::Atlases;
use crate::Name;
use crate::physics::layers::game_layer;

pub fn player_shoot(
    mut commands: Commands,
    mut query: Query<(&mut Gun, &GlobalTransform)>,
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    atlases: Res<Atlases>,
) {
    for (mut gun, transform) in query.iter_mut() {
        if time.elapsed().as_millis() - gun.last_shot_time > gun.cooldown {
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

                gun.last_shot_time = time.elapsed().as_millis();

                let mut delta = projection.point - to_vec2(translation);
                delta = delta.normalize();

                spawn_projectile(&mut commands, &gun, translation, delta, &atlases);
            }
        }
    }
}

pub fn deal_damage_on_collide(
    mut collision_events: EventReader<CollisionEvent>,
    mut health_query: Query<(Entity, &mut Health)>,
    enemy_query: Query<(Entity, &Enemy)>,//HACK do something smarter.
    damage_query: Query<(Entity, &DamageOnTouch)>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _flags) => {
                {//entity 2 damages entity 1 if it can
                    let entity1_health = health_query.get_mut(*entity1);
                    let entity2_damage = damage_query.get(*entity2);

                    let enemy1 = enemy_query.get(*entity1);
                    let enemy2 = enemy_query.get(*entity2);
                    if enemy1.is_ok() && enemy2.is_ok() {
                        continue;
                    }
                    try_deal_damage(entity2_damage, entity1_health);
                }
                {//entity 1 damages entity 2 if it can
                    let entity2_health = health_query.get_mut(*entity2);
                    let entity1_damage = damage_query.get(*entity1);
                    try_deal_damage(entity1_damage, entity2_health);
                }
            }
            _ => {}
        }
    }
}

fn try_deal_damage(entity1_damage: Result<(Entity, &DamageOnTouch), QueryEntityError>, entity2_health: Result<(Entity, Mut<Health>), QueryEntityError>) {
    match (entity1_damage, entity2_health) {
        (Ok((_, damage)), Ok((_, mut health))) => {
            health.value = health.value - damage.value;
        }
        _ => {}
    }
}


pub fn destroy_bullets(mut bullets: Query<(&mut Bullet, Entity, &Transform)>,
                       mut commands: Commands,
                       time: Res<Time>,
                       atlases: Res<Atlases>,
                       sprite_assets: Res<Assets<Spritesheet>>,
) {
    for (mut bullet, entity, transform) in bullets.iter_mut() {
        bullet.timer.tick(time.delta());
        if bullet.timer.finished()
            || bullet.hits > bullet.pierce
        {
            commands.entity(entity).despawn();
            spawn_particle(transform.translation, &mut commands, "bullet".to_string(), FIREBALL_EXPLODE_ANIMATION, &atlases, &sprite_assets);
        }
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


fn spawn_projectile(
    commands: &mut Commands,
    gun: &Gun,
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
                transform: Transform::from_translation(position),
                ..default()
            },

            ..Default::default()
        },
        physical: PhysicalBundle {
            collider: Collider::cuboid(
                0.5, 0.5),
            restitution: Restitution::new(1.0),
            velocity: Velocity { linvel: direction * speed, angvel: 0.0 },
            collision_layers: CollisionGroups::new(game_layer::PLAYER, Group::from(game_layer::GROUND | game_layer::ENEMY)),
            rigid_body: RigidBody::Dynamic,

            ..default()
        },
        bullet: Bullet
        { timer: Timer::new(Duration::from_secs(2_u64), Once), pierce: gun.pierce, ..default() },

        name: Name::new("bullet"),
        sensor: Default::default(),
        damage: DamageOnTouch { value: 5.0 },
    };
    commands.spawn(bundle);
}