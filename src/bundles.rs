use bevy::asset::{Assets, AssetServer, ErasedAssetLoader, Handle};
use bevy::core::Name;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Bundle, Circle, Color, ColorMaterial, Commands, Component, default, Deref, DerefMut, Mesh, Query, Res, ResMut, TextureAtlasLayout, Time, Timer, TimerMode, Transform};
use bevy::sprite::{Anchor, MaterialMesh2dBundle, Sprite, SpriteSheetBundle, TextureAtlas};
use bevy_xpbd_2d::components::{CollisionLayers, Friction, LinearVelocity, LockedAxes, Mass, Restitution, RigidBody};
use bevy_xpbd_2d::math::Vector2;
use bevy_xpbd_2d::prelude::{Collider, Sensor};
use rand::Rng;

use crate::components::{DamageOnTouch, Enemy, FollowPlayer, GainXPOnTouch, Health, MoveSpeed, Player};
use crate::constants::{BALL_DIAMETER, ENEMY_STARTING_POSITION, PADDLE_COLOR, PADDLE_SIZE, XP_DIAMETER};
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
pub struct PlayerBundle {
    pub sprite: SpriteSheetBundle,
    pub name: Name,
    pub player: Player,
    pub health: Health,
    pub physical: PhysicalBundle,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            physical: PhysicalBundle {
                collision_layers: CollisionLayers::new(GameLayer::Player, [GameLayer::Ball, GameLayer::Ground, GameLayer::Enemy, GameLayer::XP]),

                ..default()
            },

            sprite: SpriteSheetBundle {
                sprite: Sprite {
                    color: PADDLE_COLOR,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(0.0, -250.0, 0.0),
                    scale: PADDLE_SIZE,
                    ..default()
                },
                ..default()
            },

            name: Name::new("Player"),
            player: Default::default(),
            health: Health { value: 100.0 },
        }
    }
}

#[derive(Bundle)]
pub struct PhysicalBundle {
    pub mass: Mass,
    pub collider: Collider,
    pub friction: Friction,
    pub restitution: Restitution,
    pub linear_velocity: LinearVelocity,
    pub collision_layers: CollisionLayers,
    pub locked_axes: LockedAxes,
    pub rigid_body: RigidBody,
}

impl Default for PhysicalBundle {
    fn default() -> Self {
        Self {
            mass: Mass(10.0),
            collider: Collider::circle(0.5),
            friction: Friction::ZERO,
            restitution: Restitution::new(1.0),
            linear_velocity: LinearVelocity(Vector2::ZERO),
            collision_layers: CollisionLayers::ALL,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            rigid_body: RigidBody::Dynamic,
        }
    }
}

#[derive(Bundle)]
struct EnemyBundle {
    sprite_bundle: SpriteSheetBundle,
    physical: PhysicalBundle,
    animation_timer: AnimationTimer,
    enemy: Enemy,
    name: Name,
    follow_player: FollowPlayer,
    animation_indices: AnimationIndices,

}

impl Default for EnemyBundle {
    fn default() -> Self {
        Self {
            sprite_bundle: SpriteSheetBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(9.6_f32, 6.4_f32)),
                    anchor: Anchor::Center,
                    ..default()
                },
                transform: Transform::from_translation(ENEMY_STARTING_POSITION)
                    .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.0)),
                ..default()
            },
            physical: PhysicalBundle {
                ..default()
            },
            name: Name::new("Enemy"),

            animation_timer: AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),

            enemy: Enemy { xp: 1 },
            follow_player: FollowPlayer,
            animation_indices: AnimationIndices {first: 0,  last: 7},
        }
    }
}

impl EnemyBundle {
    fn with_sprite(path: String,
                   asset_server: Res<AssetServer>,
                   layout: Handle<TextureAtlasLayout>,
    ) -> Self {
        let texture = asset_server.load(path);
        let animation_indices = AnimationIndices { first: 0, last: 7 };

        let enemy = Self {
            sprite_bundle: SpriteSheetBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(9.6_f32, 6.4_f32)),
                    anchor: Anchor::Center,
                    ..default()
                },
                transform: Transform::from_translation(ENEMY_STARTING_POSITION)
                    .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.0)),
                atlas: TextureAtlas {
                    layout,
                    index: 0,
                },
                texture,
                ..default()
            },
            physical: PhysicalBundle {
                ..default()
            },
            name: Name::new("Enemy"),
            animation_indices,
            animation_timer: AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),

            enemy: Enemy { xp: 1 },
            follow_player: FollowPlayer,
        };

        enemy
    }
}

pub fn spawn_enemy(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<&Handles>,
) {
    let handles = query.single();


    let knight_string: String = "sprites/knight/noBKG_KnightRun_strip.png".to_string();
    let mut spawned = commands.spawn(
        EnemyBundle::with_sprite(knight_string, asset_server, handles.knight_layout_handle.clone())
    );


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
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &LinearVelocity)>,
) {
    for (mut timer, mut atlas, velocity) in &mut query {
        if timer.just_finished() {
            atlas.flip_x = velocity.x < 0.0;
        }
    }
}

#[derive(Component)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
}