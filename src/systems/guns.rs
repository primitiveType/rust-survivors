use bevy::asset::Assets;

use bevy::math::Vec3;
use bevy::prelude::{Circle, ColorMaterial, Commands, default, Entity, Mesh, Query, Res, ResMut, Time, Transform, Vec2, With};
use bevy::sprite::MaterialMesh2dBundle;
use bevy_xpbd_2d::components::{CollisionLayers, Friction, LinearVelocity, Mass, Restitution};
pub use bevy_xpbd_2d::parry::na::DimAdd;
use bevy_xpbd_2d::prelude::{Collider, CollidingEntities, RigidBody, SpatialQuery, SpatialQueryFilter};

use crate::components::{Bullet, BulletBundle, Enemy, Gun, Health, Player};
use crate::constants::BALL_COLOR;
use crate::extensions::vectors::to_vec2;
use crate::physics::layers::GameLayer;

const BULLET_SIZE: Vec3 = Vec3::new(5.0, 5.0, 1.0);

pub fn player_shoot(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&mut Gun, &Transform), With<Player>>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    let cooldown = 1_000;

    for (mut gun, transform) in query.iter_mut() {
        if time.elapsed().as_millis() - gun.last_shot_time > cooldown {
            gun.last_shot_time = time.elapsed().as_millis();
            let maybe_projection =
                spatial_query.project_point(
                    to_vec2(transform.translation),
                    true,
                    SpatialQueryFilter::from_mask(GameLayer::Enemy),
                );
            if let Some(projection) = maybe_projection {
                let mut delta = projection.point - to_vec2(transform.translation);
                delta = delta.normalize();
                spawn_projectile(&mut commands, &mut materials, &mut meshes, transform.translation, delta, time.elapsed().as_millis());
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
                    bullet.hits = bullet.hits + 1;
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
            println!("zap");
            _commands.entity(entity).despawn();
        }
    }
}

fn spawn_projectile(commands: &mut Commands, materials: &mut ResMut<Assets<ColorMaterial>>, meshes: &mut ResMut<Assets<Mesh>>, position: Vec3, direction: Vec2, timestamp: u128) {
    println!("bang!");
    let speed = 500.0;
    let bundle = BulletBundle {
        rigid_body: RigidBody::Dynamic,
        material: MaterialMesh2dBundle {
            //TODO: do I need to make sure I add resources only once?
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(BALL_COLOR),
            transform: Transform::from_translation(position)
                .with_scale(BULLET_SIZE),
            ..default()
        },

        mass: Mass(1.0),
        collider: Collider::circle(0.5),
        friction: Friction::ZERO,
        restitution: Restitution::new(1.0),
        linear_velocity: LinearVelocity(direction * speed),
        mask: CollisionLayers::new(GameLayer::Player,
                                   [GameLayer::Ground,
                                       GameLayer::Enemy]),
        bullet: Bullet { damage: 5.0, timestamp, ..default() },
    };
    commands.spawn(bundle);
}