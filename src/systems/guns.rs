use std::time::Duration;

use bevy::asset::{Assets, Handle};
use bevy::math::Vec3;
use bevy::prelude::{Commands, default, Entity, EventReader, GlobalTransform, Query, Res, SpriteSheetBundle, Time, Transform, Vec2, With};
use bevy::time::Timer;
use bevy::time::TimerMode::Once;
use bevy_asepritesheet::animator::{AnimatedSpriteBundle, AnimFinishEvent, SpriteAnimator};
use bevy_asepritesheet::prelude::{AnimEventSender, AnimHandle, Spritesheet};
use bevy_xpbd_2d::components::{CollisionLayers, Friction, LinearVelocity, Mass, Restitution};
use bevy_xpbd_2d::prelude::{Collider, CollidingEntities, RigidBody, SpatialQuery, SpatialQueryFilter};

use crate::components::{Bullet, BulletBundle, Enemy, Gun, Health};
use crate::extensions::vectors::to_vec2;
use crate::initialization::load_prefabs::Atlases;
use crate::Name;
use crate::physics::layers::GameLayer;

pub fn player_shoot(
    mut commands: Commands,
    mut query: Query<(&mut Gun, &GlobalTransform)>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
    atlases: Res<Atlases>
) {
    for (mut gun, transform) in query.iter_mut() {
        if time.elapsed().as_millis() - gun.last_shot_time > gun.cooldown {

            let translation = transform.translation();
            let maybe_projection =
                spatial_query.project_point(
                    to_vec2(translation),
                    true,
                    SpatialQueryFilter::from_mask(GameLayer::Enemy),
                );
            if let Some(projection) = maybe_projection {
                gun.last_shot_time = time.elapsed().as_millis();

                let mut delta = projection.point - to_vec2(translation);
                delta = delta.normalize();

                spawn_projectile(&mut commands, &gun, translation, delta, &atlases);
            }
        }
    }
}


pub fn enemy_takes_damage_from_bullets(mut query: Query<(&mut Health, &Enemy, &CollidingEntities)>,
                                       mut bullets: Query<&mut Bullet>,
                                       _commands: Commands,
) {
    for (mut entity, _enemy, colliding_entities) in query.iter_mut() {
        for colliding_entity in colliding_entities.iter() {
            let damager = bullets.get_mut(*colliding_entity);
            if let Ok(mut bullet) = damager {
                if bullet.hits <= bullet.pierce {
                    entity.value -= bullet.damage;
                    bullet.hits += 1;
                }
            }
        }
    }
}

pub fn destroy_bullets(mut bullets: Query<(&mut Bullet, Entity, &Transform)>,
                       mut commands: Commands,
                       time: Res<Time>,
                       atlases: Res<Atlases>,
                       sprite_assets: Res<Assets<Spritesheet>>
) {
    for (mut bullet, entity, transform) in bullets.iter_mut() {
        bullet.timer.tick(time.delta());
        if bullet.timer.finished()
            || bullet.hits > bullet.pierce
        {
            commands.entity(entity).despawn();
            spawn_particle(transform.translation, &mut commands, "bullet".to_string(), FIREBALL_EXPLODE_ANIMATION, &atlases, &sprite_assets);

            if bullet.hits > bullet.pierce {
                println!("Destroying bullet due to collisions.");
            } else {
                println!("Projectile expired.")
            }
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
        rigid_body: RigidBody::Dynamic,
        mass: Mass(
            1.0),
        collider: Collider::circle(
            0.5),
        friction: Friction::ZERO,
        restitution: Restitution::new(
            1.0),

        linear_velocity: LinearVelocity(direction * speed),
        mask: CollisionLayers::new(GameLayer::Player,
                                   [GameLayer::Ground,
                                       GameLayer::Enemy]),
        bullet: Bullet
        { damage: 5.0, timer: Timer::new(Duration::from_secs(2_u64), Once), pierce: gun.pierce, ..default() },

        name: Name::new("bullet"),
    };
    commands.spawn(bundle);
}