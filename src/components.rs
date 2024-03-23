use bevy::prelude::*;
use bevy::prelude::Component;
use bevy::prelude::Event;
use bevy::prelude::Reflect;
use bevy_asepritesheet::prelude::AnimatedSpriteBundle;
use bevy_rapier2d::geometry::Sensor;
use serde::Deserialize;
use serde::Serialize;
use crate::bundles::PhysicalBundle;

#[derive(Component, Default)]
pub struct Player {
    // pub xp: u16,
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
    pub hits: u8,
    pub pierce: u8,
    pub lifetime: u128,
    pub timer: Timer,
}

impl Default for Bullet {
    fn default() -> Self {
        Bullet {
            hits: 0,
            pierce: 0,
            lifetime: 5_000,
            timer: Timer::new(Default::default(), TimerMode::Once),
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

#[derive(Component)]
pub struct XP {
    pub amount: u16,
}

#[derive(Component)]
pub struct XPVacuum {}


// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
pub struct BulletBundle {
    pub sprite_sheet: AnimatedSpriteBundle,
    pub bullet: Bullet,
    pub physical: PhysicalBundle,
    pub name: Name,
    pub sensor: Sensor,
    pub damage: DamageOnTouch,
}


#[derive(Component)]
pub struct HealthUi;
