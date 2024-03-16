use bevy::asset::AssetServer;
use bevy::math::Vec3;
use bevy::prelude::{Commands, default, Entity, GlobalTransform, Query, Res, SpriteSheetBundle, Time, Transform, Vec2};
use bevy_asepritesheet::animator::{AnimatedSpriteBundle, SpriteAnimator};
use bevy_asepritesheet::core::load_spritesheet;
use bevy_asepritesheet::prelude::AnimHandle;
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
                spawn_projectile(&mut commands, &asset_server, &gun, translation, delta, time.elapsed().as_millis());
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

pub fn destroy_bullets(bullets: Query<(&Bullet, Entity)>,
                       mut _commands: Commands,
                       time: Res<Time>,
) {
    for (bullet, entity) in bullets.iter() {
        if bullet.timestamp + bullet.lifetime < time.elapsed().as_millis()
            || bullet.hits > bullet.pierce
        {
            _commands.entity(entity).despawn();
        }
    }
}

fn spawn_projectile(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    gun: &Gun,
    position: Vec3,
    direction: Vec2,
    timestamp: u128,
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
        { damage: 5.0, timestamp, pierce: gun.pierce, ..default() },

    };
    commands.spawn(bundle);
}