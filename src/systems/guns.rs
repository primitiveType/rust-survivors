use std::time::Duration;

use bevy::asset::{Assets, AssetServer, Handle};
use bevy::math::Vec3;
use bevy::prelude::{Commands, default, Entity, EventReader, GlobalTransform, Query, Res, SpriteSheetBundle, Time, Transform, Vec2, With};
use bevy::time::Timer;
use bevy::time::TimerMode::Once;
use bevy_asepritesheet::animator::{AnimatedSpriteBundle, AnimFinishEvent, SpriteAnimator};
use bevy_asepritesheet::core::{load_spritesheet, load_spritesheet_then};
use bevy_asepritesheet::prelude::{AnimEndAction, AnimEventSender, AnimHandle, Spritesheet};
use bevy_xpbd_2d::components::{CollisionLayers, Friction, LinearVelocity, Mass, Restitution};
pub use bevy_xpbd_2d::parry::na::DimAdd;
use bevy_xpbd_2d::prelude::{Collider, CollidingEntities, RigidBody, SpatialQuery, SpatialQueryFilter};

use crate::components::{Bullet, BulletBundle, Enemy, Gun, Health};
use crate::extensions::vectors::to_vec2;
use crate::physics::layers::GameLayer;

const BULLET_SIZE: Vec3 = Vec3::new(5.0, 5.0, 1.0);


pub fn player_shoot(
    mut commands: Commands,
    mut query: Query<(&mut Gun, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    for (mut gun, transform) in query.iter_mut() {
        if time.elapsed().as_millis() - gun.last_shot_time > gun.cooldown {
            let translation = transform.translation();
            gun.last_shot_time = time.elapsed().as_millis();
            let maybe_projection =
                spatial_query.project_point(
                    to_vec2(translation),
                    true,
                    SpatialQueryFilter::from_mask(GameLayer::Enemy),
                );
            if let Some(projection) = maybe_projection {
                println!("Bang!");
                let mut delta = projection.point - to_vec2(translation);
                delta = delta.normalize();
                spawn_projectile(&mut commands, &asset_server, &gun, translation, delta);
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
                       asset_server: Res<AssetServer>,
                       time: Res<Time>,
) {
    for (mut bullet, entity, transform) in bullets.iter_mut() {
        bullet.timer.tick(time.delta());
        if bullet.timer.finished()
            || bullet.hits > bullet.pierce
        {
            commands.entity(entity).despawn();
            spawn_particle(transform.translation, &mut commands, &asset_server, "bullets.json".to_string());
        }
    }
}

const FIREBALL_EXPLODE_ANIMATION: &'static str = "Fireball_explode";

fn spawn_particle(position: Vec3, commands: &mut Commands, asset_server: &Res<AssetServer>, sprite_sheet: String) {
    let sheet_handle = load_spritesheet_then(
        commands,
        asset_server,
        sprite_sheet,
        bevy::sprite::Anchor::Center,
        |sheet| {
            let explode = sheet.get_anim_handle(FIREBALL_EXPLODE_ANIMATION);//TODO: i guess its not possible to pass a satring to spawn_particle for the animation. consider using the same animation name
            let mut explode_mut = sheet.get_anim_mut(&explode);
            explode_mut.unwrap().end_action = AnimEndAction::Stop;
        },
    );
    commands.spawn((
        AnimEventSender,
        AnimatedSpriteBundle {
            animator: SpriteAnimator::from_anim(AnimHandle::from_index(
                1)),
            spritesheet: sheet_handle,
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
        println!("Animation {:?} complete!", event.anim);
    }
}


fn spawn_projectile(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    gun: &Gun,
    position: Vec3,
    direction: Vec2,
) {
    //todo: cache this and store in what is currently called atlases.
    let sheet_handle = load_spritesheet(
        commands,
        asset_server,
        "bullets.json",
        bevy::sprite::Anchor::Center,
    );


    let speed = gun.bullet_speed;
    let bundle = BulletBundle {
        sprite_sheet: AnimatedSpriteBundle {
            animator: SpriteAnimator::from_anim(AnimHandle::from_index(0)),
            spritesheet: sheet_handle,
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

    };
    commands.spawn(bundle);
}