use std::fmt;

use bevy::asset::{Assets, AssetServer, Handle};
use bevy::core::Name;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Bundle, Changed, Circle, Color, ColorMaterial, Commands, Component, default, Deref, DerefMut, Entity, Image, Mesh, Query, Reflect, Res, ResMut, TextureAtlasLayout, Time, Timer, TimerMode, Transform};
use bevy::sprite::{Anchor, MaterialMesh2dBundle, Sprite, SpriteSheetBundle, TextureAtlas};
use bevy_xpbd_2d::components::{CollisionLayers, Friction, LinearVelocity, LockedAxes, Mass, Restitution, RigidBody};
use bevy_xpbd_2d::math::Vector2;
use bevy_xpbd_2d::prelude::{Collider, Sensor};
use serde::{Deserialize, Serialize};

use crate::bundles::AnimationState::Run;
use crate::components::{DamageOnTouch, Enemy, FollowPlayer, GainXPOnTouch, Health, MoveSpeed, Player};
use crate::constants::{ENEMY_STARTING_POSITION, PADDLE_COLOR, PADDLE_SIZE, XP_DIAMETER};
use crate::initialization::load_prefabs::{Animations, Atlases, load_enemy, load_enemy_data_from_path};
use crate::physics::layers::GameLayer;

const XP_COLOR: Color = Color::rgb(0.0, 1.0, 0.1);

#[derive(Component)]
pub struct Handles {
    pub knight_layout_handle: Handle<TextureAtlasLayout>,

}

pub fn setup_assets(mut commands: Commands,
                    _asset_server: Res<AssetServer>,
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
pub struct EnemyBundle {
    sprite_bundle: SpriteSheetBundle,
    physical: PhysicalBundle,
    sensor: Sensor,
    animation_timer: AnimationTimer,
    enemy_data: EnemyData,
    animator: Animator,
}

#[derive(Component, Deserialize, Serialize, Debug, Clone)]
pub struct SpritePath(String);

#[derive(Deserialize, Serialize, Bundle, Clone)]
pub struct EnemyData {
    pub sprite_path: SpritePath,
    enemy: Enemy,
    pub name: Name,
    follow_player: FollowPlayer,
    move_speed: MoveSpeed,
    health: Health,
    touch_damage: DamageOnTouch,
}


impl EnemyBundle {
    pub fn from_path(
        path: &str,
        asset_server: Res<AssetServer>,
        atlases: ResMut<Atlases>,
    ) -> Self {
        let enemy_data = load_enemy_data_from_path(path);
        let image = asset_server.load(&enemy_data.sprite_path.0);
        let str = enemy_data.name.as_str();
        println!("{str}{}", atlases.map.keys().last().unwrap());
        let layout = atlases.map[str].clone();
        Self {
            enemy_data: enemy_data.clone(),
            sprite_bundle: Self::get_default_sprite_sheet_bundle(image, layout),
            animator: Animator {
                state: Run,
                name: enemy_data.name.to_string(),
            },
            ..default()
        }
    }

    fn get_default_sprite_sheet_bundle(image: Handle<Image>, layout: Handle<TextureAtlasLayout>) -> SpriteSheetBundle {
        SpriteSheetBundle {
            texture: image,
            atlas: TextureAtlas {
                layout,
                index: 0,
            },
            sprite: Sprite {
                anchor: Anchor::Center,

                ..default()
            },
            transform: Transform::from_translation(ENEMY_STARTING_POSITION)
                .with_scale(Vec2::splat(4.0).extend(1.0)),
            ..default()
        }
    }
}

impl Default for EnemyBundle {
    fn default() -> Self {
        Self {
            sprite_bundle: Self::get_default_sprite_sheet_bundle(Handle::default(), Handle::default()),
            physical: PhysicalBundle {
                ..default()
            },
            enemy_data: EnemyData {
                sprite_path: SpritePath("".to_string()),
                name: Name::new("Enemy"),
                enemy: Enemy { xp: 1 },
                follow_player: FollowPlayer,
                move_speed: MoveSpeed { value: 1.0 },
                health: Health { value: 5.0 },
                touch_damage: DamageOnTouch { value: 1.0 },
            },
            sensor: Default::default(),
            animation_timer: AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            animator: Animator { state: Run, name: "default".to_string() },
        }
    }
}

// impl EnemyBundle {
//     fn with_sprite(path: String,
//                    asset_server: Res<AssetServer>,
//                    layout: Handle<TextureAtlasLayout>,
//     ) -> Self {
//         let texture = asset_server.load(path);
//         let animation_indices = AnimationIndices { first: 0, last: 7 };
//
//         let enemy = Self {
//             sprite_bundle: SpriteSheetBundle {
//                 sprite: Sprite {
//                     custom_size: Some(Vec2::new(9.6_f32, 6.4_f32)),
//                     anchor: Anchor::Center,
//                     ..default()
//                 },
//                 transform: Transform::from_translation(ENEMY_STARTING_POSITION)
//                     .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.0)),
//                 atlas: TextureAtlas {
//                     layout,
//                     index: 0,
//                 },
//                 texture,
//                 ..default()
//             },
//             physical: PhysicalBundle {
//                 ..default()
//             },
//             sensor: Sensor,
//             animation_timer: AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
//             enemy_data: EnemyData {
//                 sprite_path: SpritePath(),
//                 enemy: Enemy {},
//                 name: Default::default(),
//                 follow_player: FollowPlayer,
//                 animation_indices,
//                 move_speed: MoveSpeed {},
//                 health: Health {},
//                 touch_damage: DamageOnTouch {},
//             },
//         };
//
//         enemy
//     }
// }


pub fn spawn_enemy(
    enemy: usize,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlases: ResMut<Atlases>,
) {
    let bundle = load_enemy(enemy, asset_server, atlases);
    commands.spawn(bundle);
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

#[derive(Component, Deref, DerefMut, Serialize, Deserialize, Default, Reflect)]
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

pub fn update_animations(
    animations: Res<Animations>,
    mut entity_commands: Commands,
    mut query: Query<(Entity, &Animator), Changed<Animator>>,
) {
    for (entity, animator) in &mut query.iter_mut() {
        let state = &format!("{}/{}", &animator.name, &animator.state.to_string());
        println!("{}", state);
        entity_commands.entity(entity).insert(animations.map[state]);
    }
}

pub fn flip_sprite(
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &LinearVelocity)>,
) {
    for (timer, mut atlas, velocity) in &mut query {
        if timer.just_finished() {
            atlas.flip_x = velocity.x < 0.0;
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Copy, Reflect)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Reflect)]
pub struct Animator {
    state: AnimationState,
    name: String,
}

#[derive(Debug, Reflect)]
pub enum AnimationState {
    Run,
}

impl fmt::Display for AnimationState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}