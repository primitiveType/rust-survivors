use std::time::Duration;
use bevy::prelude::*;
use bevy::prelude::Component;
use bevy::prelude::Event;
use bevy::prelude::Reflect;
use bevy::prelude::TimerMode::Once;
use bevy::time::TimerMode::Repeating;
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

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Cooldown {
    #[serde(skip)]
    pub timer: Timer,
}

#[derive(Component, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Flask {
    pub bullet_size: f32,
}

#[derive(Component, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct FireBallGun {
    pub bullet_size: f32,
    pub pierce: u8,
    pub bullet_speed: f32,
}

#[derive(Component, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct AttackSpeed {
    pub percent: f32,
}

impl Default for Cooldown {
    fn default() -> Self {
        Self {
            timer: bevy::prelude::Timer::new(Duration::from_secs(2_u64), Repeating),
            ..default()
        }
    }
}

impl Cooldown {
    pub fn with_cooldown(ms: u64) -> Self {
        Self {
            timer: bevy::prelude::Timer::new(Duration::from_millis(ms), Repeating),
        }
    }
}

#[derive(Component)]
pub struct Bullet {
    pub hits: u8,
    pub pierce: u8,
}

#[derive(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

impl Lifetime {
    pub fn from_seconds(seconds: f32) -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f32(seconds), Once),
        }
    }
}

#[derive(Component)]
pub struct Expired {}

impl Default for Bullet {
    fn default() -> Self {
        Bullet {
            hits: 0,
            pierce: 0,
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
    #[serde(skip)]

    pub count_triggers: u32,
}

impl Default for DamageOnTouch {
    fn default() -> Self {
        Self {
            value: 1.0,
            count_triggers: 0,
        }
    }
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
    pub lifetime: Lifetime,
}

#[derive(Bundle)]
pub struct FlaskProjectileBundle {
    pub sprite_sheet: AnimatedSpriteBundle,
    pub physical: PhysicalBundle,
    pub name: Name,
    pub sensor: Sensor,
    pub damage: DamageOnTouch,
    pub lifetime: Lifetime,
}


#[derive(Component)]
pub struct HealthUi;
