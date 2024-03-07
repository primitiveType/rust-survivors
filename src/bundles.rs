use bevy::asset::{Assets, AssetServer, ErasedAssetLoader, Handle};
use bevy::core::Name;
use bevy::math::Vec2;
use bevy::prelude::{Bundle, Circle, Color, ColorMaterial, Commands, Component, default, Deref, DerefMut, Mesh, Query, Res, ResMut, TextureAtlasLayout, Time, Timer, TimerMode, Transform};
use bevy::sprite::{Anchor, MaterialMesh2dBundle, Sprite, SpriteSheetBundle, TextureAtlas};
use bevy_xpbd_2d::components::{CollisionLayers, Friction, LinearVelocity, LockedAxes, Restitution, RigidBody};
use bevy_xpbd_2d::math::Vector2;
use bevy_xpbd_2d::prelude::{Collider, Sensor};
use rand::Rng;

use crate::components::{DamageOnTouch, Enemy, FollowPlayer, GainXPOnTouch, Health, MoveSpeed};
use crate::constants::{BALL_DIAMETER, ENEMY_STARTING_POSITION, XP_DIAMETER};
use crate::physics::layers::GameLayer;

const XP_COLOR: Color = Color::rgb(0.0, 1.0, 0.1);

#[derive(Component)]
pub struct Handles {
    pub knight_layout_handle: Handle<TextureAtlasLayout>,

}

pub fn setup_assets(mut commands: Commands,
                    asset_server: Res<AssetServer>,
                    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = TextureAtlasLayout::from_grid(Vec2::new(96.0, 64.0), 8, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);


    commands.spawn(Handles {
        knight_layout_handle: texture_atlas_layout,

    });
}

#[derive(Bundle)]
struct PlayerBundle {}

#[derive(Bundle)]
struct PhysicalBundle {}

struct EnemyBundle {
    sprite_bundle: SpriteSheetBundle,

}

pub fn spawn_enemy(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<&Handles>,
) {
    let handles = query.single();
    let image_path = "sprites/knight/noBKG_KnightRun_strip.png";
    let texture = asset_server.load(image_path);

    let animation_indices = AnimationIndices { first: 0, last: 7 };
    let mut spawned = commands.spawn((
        SpriteSheetBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(9.6_f32, 6.4_f32)),
                anchor: Anchor::Center,
                ..default()
            },
            transform: Transform::from_translation(ENEMY_STARTING_POSITION)
                .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.0)),
            atlas: TextureAtlas {
                layout: handles.knight_layout_handle.clone(),
                index: 0,
            },
            texture,
            ..default()
        },
        Friction::ZERO,
        Restitution::new(1.0),
        LinearVelocity(Vector2::ZERO),
        Name::new("Enemy"),
        LockedAxes::ROTATION_LOCKED,
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Enemy { xp: 1 }, FollowPlayer
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
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    _amount: u16,
    position: Vec2,
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


    spawned.insert(LinearVelocity(Vec2::new(0.0, 0.0)));
    spawned.insert(Collider::circle(0.5));
    spawned.insert(RigidBody::Dynamic);
    spawned.insert(CollisionLayers::new(GameLayer::XP, [GameLayer::Player]));
    spawned.insert(GainXPOnTouch { value: 1u16 });

    spawned.insert(Sensor);
    spawned.insert(Name::new("xp"));
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}

pub fn flip_sprite(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.layout.fli
        }
    }
}

#[derive(Component)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
}