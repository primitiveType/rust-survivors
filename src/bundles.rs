use bevy::asset::Assets;
use bevy::core::Name;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Bundle, Circle, Color, ColorMaterial, Commands, default, In, Mesh, ResMut, SpatialBundle, Transform};
use bevy::sprite::{MaterialMesh2dBundle, SpriteSheetBundle};
use bevy_asepritesheet::prelude::AnimatedSpriteBundle;
use bevy_prng::WyRand;
use bevy_rand::prelude::GlobalEntropy;
use bevy_rapier2d::dynamics::{LockedAxes, RigidBody, Velocity};
use bevy_rapier2d::geometry::{ActiveEvents, Collider, CollisionGroups, Restitution, Sensor};
use bevy_rapier2d::parry::transformation::utils::transform;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::animation::{AnimationState, AnimatorController, SpritePath};
use crate::animation::AnimationState::Walk;
use crate::components::{AbilityLevel, BaseMoveSpeed, DamageOnTouch, Enemy, FollowPlayer, GainXPOnTouch, Health, MoveSpeed, Player, XP};
use crate::constants::{PLAYER_SPEED, XP_DIAMETER};
use crate::initialization::load_prefabs::{Atlases, Enemies, load_enemy_data_from_path};
use crate::physics::layers::game_layer;
use crate::systems::animation::AnimationState::Idle;

const XP_COLOR: Color = Color::rgb(0.0, 1.0, 0.1);

#[derive(Debug, Eq, PartialEq)]
pub enum Object {
    Cube,
    Player,
    Enemy,
    Fireball,
    Flask,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub sprite: AnimatedSpriteBundle,
    pub name: Name,
    pub player: Player,
    pub health: Health,
    pub physical: PhysicalBundle,
    pub animator: AnimatorController,
    pub xp: XP,
    move_speed: MoveSpeed,
    pub base_speed: BaseMoveSpeed,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            physical: PhysicalBundle {
                collision_layers: CollisionGroups::new(game_layer::PLAYER, game_layer::GROUND | game_layer::ENEMY | game_layer::XP),

                ..default()
            },

            sprite: AnimatedSpriteBundle {
                sprite_bundle: Default::default(),
                spritesheet: Default::default(),
                animator: Default::default(),
                needs_img: Default::default(),
                needs_atlas: Default::default(),
            },

            name: Name::new("Player"),
            player: Default::default(),
            health: Health { value: 100.0 },
            animator: AnimatorController { state: AnimationState::Walk, name: "default".to_string() },
            xp: XP { amount: 0 },
            move_speed: MoveSpeed { value: 0.0 },
            base_speed: BaseMoveSpeed { value: PLAYER_SPEED },
        }
    }
}

impl PlayerBundle {
    pub fn with_sprite(atlases: ResMut<Atlases>) -> Self {
        Self {
            physical: PhysicalBundle {
                collider: Collider::ball(2.0),
                rigid_body: RigidBody::Dynamic,
                collision_layers: CollisionGroups::new(game_layer::PLAYER, game_layer::GROUND | game_layer::ENEMY | game_layer::XP),
                ..default()
            },
            sprite: AnimatedSpriteBundle {
                spritesheet: atlases.sprite_sheets.get("player").unwrap().clone(),
                sprite_bundle: SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(0.0, -250.0, 0.0),
                        scale: Vec2::splat(4.0).extend(1.0),

                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
            name: Name::new("Player"),
            player: Default::default(),
            health: Health { value: 100.0 },
            animator: AnimatorController { state: Idle, name: "player".to_string() },

            xp: XP { amount: 2 },
            move_speed: MoveSpeed { value: 0.0 },
            base_speed: BaseMoveSpeed { value: PLAYER_SPEED },
        }
    }
}


#[derive(Bundle, Clone)]
pub struct PhysicalBundle {
    // pub mass: Mass,
    pub collider: Collider,
    // pub friction: Friction,
    pub restitution: Restitution,
    pub velocity: Velocity,
    pub collision_layers: CollisionGroups,
    // pub locked_axes: LockedAxes,
    pub rigid_body: RigidBody,
    pub locked_axes: LockedAxes,
    pub active_events: ActiveEvents,
}

impl Default for PhysicalBundle {
    fn default() -> Self {
        Self {
            // mass: Mass(10.0),
            collider: Collider::ball(4.0),
            // friction: Friction::ZERO,
            restitution: Restitution::new(1.0),
            velocity: Velocity { linvel: Vec2::ZERO, angvel: 0.0 },
            collision_layers: Default::default(),
            rigid_body: RigidBody::Dynamic,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            active_events: ActiveEvents::COLLISION_EVENTS,
        }
    }
}

#[derive(Bundle, Clone)]
pub struct EnemyBundle {
    animation_bundle: AnimatedSpriteBundle,
    physical: PhysicalBundle,
    // sensor: Sensor,
    enemy_data: EnemyData,
    animator: AnimatorController,
}

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

pub struct AbilityBundle {
    pub spatial: SpatialBundle,
    pub name: Name,
    pub ability: AbilityLevel,
}

impl EnemyBundle {
    pub fn from_path(
        path: &str,
        atlases: &ResMut<Atlases>,
    ) -> Self {
        let enemy_data = load_enemy_data_from_path(path);

        Self {
            enemy_data: enemy_data.clone(),
            animator: AnimatorController {
                state: Walk,
                name: enemy_data.name.to_string(),
            },
            animation_bundle: AnimatedSpriteBundle {
                spritesheet: atlases.sprite_sheets.get(&enemy_data.name.to_string()).expect(&format!("{} not found!", &enemy_data.name).to_string()).clone(),
                sprite_bundle: SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(0.0, -250.0, 0.0),
                        scale: Vec2::splat(4.0).extend(1.0),

                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
            ..default()
        }
    }
}


impl Default for EnemyBundle {
    fn default() -> Self {
        Self {
            // sprite_bundle: get_default_sprite_sheet_bundle(Handle::default(), Handle::default()),
            physical: PhysicalBundle {
                collision_layers: CollisionGroups::new(game_layer::ENEMY, game_layer::PLAYER | game_layer::ENEMY),
                ..default()
            },
            enemy_data: EnemyData {
                sprite_path: SpritePath("".to_string()),
                name: Name::new("Enemy"),
                enemy: Enemy { xp: 1 },
                follow_player: FollowPlayer,
                move_speed: MoveSpeed { value: 0.1 },
                health: Health { value: 5.0 },
                touch_damage: DamageOnTouch { value: 1.0, ..default() },
            },
            // sensor: Default::default(),
            animator: AnimatorController { state: Walk, name: "default".to_string() },
            animation_bundle: Default::default(),

        }
    }
}

pub struct EnemySpawnData {
    pub enemy_id: String,
    pub player_position: Vec2,
}

pub fn spawn_enemy(
    In(enemy_spawn_data): In<EnemySpawnData>,
    enemies: ResMut<Enemies>,
    _rng: ResMut<GlobalEntropy<WyRand>>,
    mut commands: Commands,
) {
    let mut bundle: EnemyBundle = enemies.datas.get(&enemy_spawn_data.enemy_id).unwrap().clone();

    //get random position outside screen
    let mut rng = rand::thread_rng();
    let value = rng.gen_range(0.0..1.0);
    let angle = value * 2.0 * std::f32::consts::PI;
    // Calculate the direction vector from the angle
    let mut direction = Vec2::new(angle.cos(), angle.sin());
    let distance = Vec2::splat(600.0);
    direction *= distance;
    bundle.animation_bundle.sprite_bundle.transform.translation = (direction + enemy_spawn_data.player_position).extend(0.0);
    // bundle.animation_bundle.sprite_bundle.transform.translation = (direction + enemy_spawn_data.player_position).extend(0.0);
    let mut enemy = commands.spawn(bundle);

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


    spawned.insert(Velocity { linvel: Vec2::ZERO, angvel: 0.0 });
    spawned.insert(Collider::ball(0.5));
    spawned.insert(RigidBody::Dynamic);
    spawned.insert(CollisionGroups::new(game_layer::XP, game_layer::XP | game_layer::PLAYER));
    spawned.insert(GainXPOnTouch { value: 1u16 });

    spawned.insert(Sensor);
    spawned.insert(Name::new("xp"));
}