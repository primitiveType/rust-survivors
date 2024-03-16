use bevy::asset::Handle;
use bevy::audio::AudioSource;
use bevy::math::Vec2;
use bevy::prelude::{Bundle, ColorMaterial, Component, default, Event, Reflect, Resource, Sprite, SpriteBundle, Transform};
use bevy::sprite::{MaterialMesh2dBundle, SpriteSheetBundle};
use bevy_asepritesheet::prelude::AnimatedSpriteBundle;
use bevy_xpbd_2d::components::{CollisionLayers, Friction, LinearVelocity, Mass, Restitution, RigidBody};
use bevy_xpbd_2d::prelude::Collider;
use serde::Deserialize;
use serde::Serialize;

use crate::constants::{BOTTOM_WALL, LEFT_WALL, RIGHT_WALL, TOP_WALL, WALL_COLOR, WALL_THICKNESS};
use crate::physics::layers::GameLayer;

#[derive(Component, Default)]
pub struct Player {
    pub xp: u16,
    pub level: u16,
}

#[derive(Component, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Gun {
    #[serde(skip)]
    pub last_shot_time: u128,
    pub cooldown: u128,
    pub bullet_size: f32,
    pub pierce: u8,
    pub bullet_speed: f32,
}

impl Default for Gun {
    fn default() -> Self {
        Self {
            cooldown: 1_000,
            ..default()
        }
    }
}

#[derive(Component)]
pub struct Bullet {
    pub damage: f32,
    pub hits: u8,
    pub pierce: u8,
    pub lifetime: u128,
    pub timestamp: u128,
}

impl Default for Bullet {
    fn default() -> Self {
        Bullet {
            damage: 1.0,
            hits: 0,
            pierce: 0,
            lifetime: 5_000,
            timestamp: 0,
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Health {
    pub value: f32,
}


#[derive(Component, Serialize, Deserialize, Clone)]
pub struct FollowPlayer;


#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
pub struct MoveSpeed {
    pub value: f32,
}


#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Enemy {
    pub xp: u16,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct DamageOnTouch {
    pub value: f32,
}

#[derive(Component)]
pub struct GainXPOnTouch {
    pub value: u16,
}

#[derive(Event, Default)]
pub struct CollisionEvent;

#[derive(Resource)]
pub struct CollisionSound(pub Handle<AudioSource>);

// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
pub struct BulletBundle {
    pub sprite_sheet: AnimatedSpriteBundle,
    // pub material: MaterialMesh2dBundle<ColorMaterial>,
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub friction: Friction,
    pub restitution: Restitution,
    pub mask: CollisionLayers,
    pub bullet: Bullet,
    pub mass: Mass,
    pub linear_velocity: LinearVelocity,
}


// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
pub struct WallBundle {
    // You can nest bundles inside of other bundles like this
    // Allowing you to compose their functionality
    pub sprite_bundle: SpriteBundle,
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub friction: Friction,
    pub restitution: Restitution,
    pub mask: CollisionLayers,
}

/// Which side of the arena is this wall located on?
pub enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        // Make sure we haven't messed up our constants
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    // This "builder method" allows us to reuse logic across our wall entities,
    // making our code easier to read and less prone to bugs when we change the logic
    pub fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                    // This is used to determine the order of our sprites
                    translation: location.position().extend(0.0),
                    // The z-scale of 2D objects must always be 1.0,
                    // or their ordering will be affected in surprising ways.
                    // See https://github.com/bevyengine/bevy/issues/4149
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider::rectangle(1.0, 1.0),
            rigid_body: RigidBody::Static,
            friction: Friction::ZERO,
            restitution: Restitution::new(1.0),
            mask: CollisionLayers::new(GameLayer::Ground, [GameLayer::Ball, GameLayer::Player, GameLayer::Enemy]),
        }
    }
}


#[derive(Component)]
pub struct HealthUi;
