use bevy::prelude::{Circle, Color, ColorMaterial, Commands, default, Mesh, ResMut, Transform};
use bevy::asset::Assets;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::math::Vec2;
use bevy_xpbd_2d::components::{CollisionLayers, LinearVelocity, RigidBody};
use bevy_xpbd_2d::prelude::{Collider, Sensor};
use bevy::core::Name;
use bevy_prng::WyRand;
use bevy_rand::prelude::GlobalEntropy;
use rand::Rng;
use crate::{BALL_DIAMETER, BALL_STARTING_POSITION, DamageOnTouch, Enemy, FollowPlayer, GainXPOnTouch, Health, MoveSpeed, XP_DIAMETER};
use crate::physics::layers::GameLayer;

const ENEMY_COLOR: Color = Color::rgb(1.0, 0.1, 0.1);
const XP_COLOR: Color = Color::rgb(0.0, 1.0, 0.1);

pub fn spawn_enemy(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut spawned = commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(ENEMY_COLOR),
            transform: Transform::from_translation(BALL_STARTING_POSITION)
                .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.0)),
            ..default()
        },
        Enemy{xp : 1}, FollowPlayer
    ));


    let mut rng = rand::thread_rng(); // Get a random number generator
    let speed = rng.gen_range(100.0..300.0);
    spawned.insert(MoveSpeed::new(speed));
    spawned.insert(Health { value: 10.0 });
    spawned.insert(LinearVelocity(Vec2::new(0.0, 0.0)));
    spawned.insert(Collider::circle(0.5));
    spawned.insert(RigidBody::Dynamic);
    spawned.insert(CollisionLayers::new(GameLayer::Enemy, [GameLayer::Player, GameLayer::Enemy, GameLayer::Ground]));
    spawned.insert(DamageOnTouch { value: 1.0 });

    // spawned.insert(Sensor);
    spawned.insert(Name::new("enemy"));
}

pub fn spawn_xp(
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    amount : u16,
    position : Vec2,
) {
    let mut spawned = commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(XP_COLOR),
            transform: Transform::from_translation(position.extend(0.0))
                .with_scale(Vec2::splat(XP_DIAMETER).extend(1.0)),
            ..default()
        },
    ));


    let mut rng = rand::thread_rng(); // Get a random number generator
    let speed = rng.gen_range(100.0..300.0);
    spawned.insert(LinearVelocity(Vec2::new(0.0, 0.0)));
    spawned.insert(Collider::circle(0.5));
    spawned.insert(RigidBody::Dynamic);
    spawned.insert(CollisionLayers::new(GameLayer::XP, [GameLayer::Player]));
    spawned.insert(GainXPOnTouch { value: 1u16 });

    spawned.insert(Sensor);
    spawned.insert(Name::new("xp"));
}