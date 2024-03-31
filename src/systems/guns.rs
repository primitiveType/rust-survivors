use std::time::Duration;

use bevy::asset::{Assets, Handle};
use bevy::ecs::query::QueryEntityError;
use bevy::math::{Vec3, Vec3Swizzles};
use bevy::prelude::{Color, Commands, Component, default, Entity, EventReader, GlobalTransform, In, Mut, Query, Res, ResMut, SpriteSheetBundle, Text, Text2dBundle, TextStyle, Time, Transform, Vec2, With};
use bevy_asepritesheet::animator::{AnimatedSpriteBundle, AnimFinishEvent, SpriteAnimator};
use bevy_asepritesheet::prelude::{AnimEventSender, AnimHandle, Spritesheet};
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::geometry::{Collider, CollisionGroups, Restitution};
use bevy_rapier2d::na::clamp;
use bevy_rapier2d::pipeline::CollisionEvent;
use bevy_rapier2d::plugin::RapierContext;
use bevy_rapier2d::prelude::{QueryFilter, Velocity};
use rand::Rng;

use crate::bundles::{Object, PhysicalBundle};
use crate::components::{Cooldown, Bullet, BulletBundle, DamageOnTouch, Enemy, FireBallGun, Health, AttackSpeed, Flask, FlaskProjectileBundle, Lifetime, Expired, AbilityLevel};
use crate::extensions::spew_extensions::{Spawn, Spawner};
use crate::extensions::vectors::to_vec2;
use crate::initialization::load_prefabs::Atlases;
use crate::Name;
use crate::physics::layers::game_layer;
use crate::systems::ui::FadeTextWithLifetime;

//adding a new ability
//1.. add system that does the ability thing. It should require an AbilityLevel component
//2.. add level 0 bundle on player in spawn_player
//3.. add system for updating the description of the ability when its level changes
//4.. add all 4 systems to update loop.
pub fn advance_cooldowns(
    mut query: Query<&mut Cooldown>,
    mut cdr_query: Query<&mut AttackSpeed>,
    time: Res<Time>,
) {//assumes only player needs concept of abilities and CDR, which might change.
    for mut ability in query.iter_mut() {
        let mut total_cdr = 100.0f32;
        for cdr in cdr_query.iter_mut() {
            total_cdr += cdr.percent;
        }

        let multiplier = total_cdr * 0.01f32;//convert percentage to multiplier
        let delta_seconds = time.delta().as_secs_f32();
        let multiplied_delta = delta_seconds * multiplier;

        let duration = Duration::from_secs_f32(multiplied_delta);
        //this idea of advancing the timer will make less sense if we
        //display the timer for the user. If that happens, we will have to
        //track the timer duration and update it based on stats when they change.
        ability.timer.tick(duration);
    }
}

pub fn flask_weapon(
    mut query: Query<(&mut Cooldown, &GlobalTransform, &Flask, &AbilityLevel)>,
    mut spawner: Spawner<FlaskSpawnData>,
) {
    for (ability, transform, flask, level) in query.iter_mut() {
        if level.level == 0 {
            continue;
        }
        if ability.timer.just_finished() {
            let translation = transform.translation();

            let mut rng = rand::thread_rng();
            let value = rng.gen_range(0.0..1.0);
            let angle = value * 2.0 * std::f32::consts::PI;
            // Calculate the direction vector from the angle
            let mut direction = Vec2::new(angle.cos(), angle.sin());


            let distance = Vec2::splat(rng.gen_range(50.0..400.0));
            direction *= distance;

            let mut spawn_data = FlaskSpawnData::get_data_for_level(level.level);
            spawn_data.position = translation.xy() + direction;
            spawner.spawn(Object::Flask, spawn_data);
            // spawn_flask_projectile(&mut commands, flask, direction, &atlases);
        }
    }
}


pub fn fireball_gun(
    mut query: Query<(&mut Cooldown, &GlobalTransform, &FireBallGun, &AbilityLevel)>,
    mut spawner: Spawner<FireballSpawnData>,
    rapier_context: Res<RapierContext>,
) {
    for (ability, transform, gun, level) in query.iter_mut() {
        if level.level == 0 {
            continue;
        }
        if ability.timer.just_finished() {
            let translation = transform.translation();
            if let Some((entity, projection)) = rapier_context.project_point(
                to_vec2(translation),
                true,
                QueryFilter {
                    flags: Default::default(),
                    groups: Some(CollisionGroups::new(game_layer::ENEMY, game_layer::ENEMY)),//is this filter correct?
                    exclude_collider: None,
                    exclude_rigid_body: None,
                    predicate: None,
                },
            ) {
                // The collider closest to the point has this `handle`.
                // println!("Projected point on entity {:?}. Point projection: {}", entity, projection.point);
                // println!("Point was inside of the collider shape: {}", projection.is_inside);

                let mut delta = projection.point - to_vec2(translation);
                delta = delta.normalize();

                let mut spawn_data = FireballSpawnData::get_data_for_level(level.level);
                spawn_data.position = translation;
                spawn_data.direction = delta;
                spawner.spawn(Object::Fireball, spawn_data);
                // spawn_fireball(&mut commands, &gun, translation, delta, &atlases);
            }
        }
    }
}

pub fn deal_damage_on_collide(
    mut collision_events: EventReader<CollisionEvent>,
    mut health_query: Query<(Entity, &mut Health, &Transform)>,
    enemy_query: Query<(Entity, &Enemy)>,//HACK do something smarter.
    mut damage_query: Query<(Entity, &mut DamageOnTouch)>,
    mut spawner: Spawner<DamageTextSpawnData>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _flags) => {
                {//entity 2 damages entity 1 if it can
                    let entity1_health = health_query.get_mut(*entity1);
                    let entity2_damage = damage_query.get_mut(*entity2);

                    let enemy1 = enemy_query.get(*entity1);
                    let enemy2 = enemy_query.get(*entity2);
                    if enemy1.is_ok() && enemy2.is_ok() {
                        continue;
                    }
                    try_deal_damage(entity2_damage, entity1_health, &mut spawner);
                }
                {//entity 1 damages entity 2 if it can
                    let entity2_health = health_query.get_mut(*entity2);
                    let entity1_damage = damage_query.get_mut(*entity1);
                    try_deal_damage(entity1_damage, entity2_health, &mut spawner);
                }
            }
            _ => {}
        }
    }
}

fn try_deal_damage(entity1_damage: Result<(Entity, Mut<DamageOnTouch>), QueryEntityError>, entity2_health: Result<(Entity, Mut<Health>, &Transform), QueryEntityError>, spawner: &mut Spawner<DamageTextSpawnData>) {
    match (entity1_damage, entity2_health) {
        (Ok((_, mut damage)), Ok((_, mut health, transform))) => {
            health.value -= damage.value;
            damage.count_triggers += 1;
            spawner.spawn(Object::DamageNumber, DamageTextSpawnData { position: transform.translation.xy(), amount: damage.value as u32 })
        }
        _ => {}
    }
}


pub fn expire_bullets_on_hit(mut bullets: Query<(&mut Bullet, Entity, &Transform, &DamageOnTouch)>,
                             mut commands: Commands,
) {
    for (bullet, entity, transform, damage) in bullets.iter_mut() {
        if damage.count_triggers > bullet.pierce.into()
        {
            commands.entity(entity).insert(Expired {});
        }
    }
}

pub fn expired_bullets_explode(mut bullets: Query<(Entity, &Bullet, &Transform), With<Expired>>,
                               mut commands: Commands,
                               atlases: Res<Atlases>,
                               sprite_assets: Res<Assets<Spritesheet>>, ) {
    for (bullet, entity, transform) in bullets.iter_mut() {
        spawn_particle(transform.translation, &mut commands, "bullets".to_string(), FIREBALL_EXPLODE_ANIMATION, &atlases, &sprite_assets);
    }
}

pub fn expire_entities(mut lifetimes: Query<(Entity, &mut Lifetime)>,
                       mut commands: Commands,
                       time: Res<Time>) {
    for (entity, mut lifetime) in lifetimes.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).insert(Expired {});
        }
    }
}

pub fn destroy_expired_entities(mut lifetimes: Query<(Entity, &Expired)>,
                                mut commands: Commands,
) {
    for (entity, _) in lifetimes.iter_mut() {
        commands.entity(entity).despawn();
    }
}

const FIREBALL_EXPLODE_ANIMATION: &str = "Fireball_explode";

fn spawn_particle(position: Vec3, commands: &mut Commands, sprite_sheet: String, animation: &str, atlases: &Res<Atlases>, sprite_assets: &Res<Assets<Spritesheet>>) {
    let spritesheet = atlases.sprite_sheets.get(&sprite_sheet).expect("failed to find explode animation!").clone();
    let mut anim_handle = AnimHandle::from_index(0);
    // Attempt to get the asset using the handle
    if let Some(asset) = sprite_assets.get(&spritesheet) {
        // Now you have access to the asset (`T`) here
        // Do something with the asset
        anim_handle = asset.get_anim_handle(animation);
    } else {
        // The asset is not loaded yet, you might handle this case accordingly
        println!("Asset not loaded yet");
    }
    commands.spawn((
        AnimEventSender,
        AnimatedSpriteBundle {
            animator: SpriteAnimator::from_anim(anim_handle),
            spritesheet,
            sprite_bundle: SpriteSheetBundle
            {
                transform: Transform::from_translation(position),
                ..default()
            },

            ..Default::default()
        }));
}

pub fn destroy_explosions(
    mut commands: Commands,
    mut events: EventReader<AnimFinishEvent>,
    spritesheet_assets: Res<Assets<Spritesheet>>,
    animated_sprite_query: Query<&Handle<Spritesheet>, With<SpriteAnimator>>,
) {
    for event in events.read() {
        // get the spritesheet handle off the animated sprite entity
        if let Ok(sheet_handle) = animated_sprite_query.get(event.entity) {
            if let Some(anim_sheet) = spritesheet_assets.get(sheet_handle) {
                // get the animation reference from the spritesheet
                if let Ok(anim) = anim_sheet.get_anim(&event.anim) {
                    if anim.name == FIREBALL_EXPLODE_ANIMATION {
                        commands.entity(event.entity).despawn();
                    }
                }
            }
        }
    }
}

pub struct DamageTextSpawnData {
    position: Vec2,
    amount: u32,
    //type
}

pub struct FlaskSpawnData {
    gun: Flask,
    position: Vec2,
    pub scale: f32,
    pub cooldown: Cooldown,
}

impl LevelableData for FlaskSpawnData {
    fn get_data_for_level(level: u8) -> Self {
        Self {
            gun: Flask {},
            position: Default::default(),
            scale: 1.0 * level as f32,
            cooldown: Cooldown::from_seconds(clamp(10.0 - (0.5 * level as f32), 0.5, 100.0)),
        }
    }
}

pub trait LevelableData {
    fn get_data_for_level(level: u8) -> Self;
}

pub const BACKGROUND_PROJECTILE_LAYER: f32 = -1.0;
pub const DAMAGE_TEXT_LAYER: f32 = 10.0;



pub fn spawn_damage_text(In(data): In<DamageTextSpawnData>,
                         mut commands: Commands, ) {
    commands.spawn((Text2dBundle {
        text: Text::from_section(
            data.amount.to_string(),
            TextStyle {
                font: Default::default(),
                /* Load or reference your font here */
                font_size: 40.0,
                color: Color::WHITE,
            },
        ),
        transform: Transform::from_translation((data.position + Vec2::new(0.0, 50.0)).extend(DAMAGE_TEXT_LAYER)), // Offset above the enemy
        ..Default::default()
    },
                    Lifetime::from_seconds(1.0),
                    FadeTextWithLifetime {},
    ));
}

pub fn spawn_flask_projectile(
    In(data): In<FlaskSpawnData>,
    mut commands: Commands,
    atlases: Res<Atlases>,
) {
    let bundle = FlaskProjectileBundle {
        sprite_sheet: AnimatedSpriteBundle {
            animator: SpriteAnimator::from_anim(AnimHandle::from_index(0)),
            spritesheet: atlases.sprite_sheets.get("bullets").expect("failed to find asset for bullet!").clone(),
            sprite_bundle: SpriteSheetBundle {
                transform: Transform::from_translation(data.position.extend(BACKGROUND_PROJECTILE_LAYER)).with_scale(Vec2::splat(data.scale).extend(1.0)),
                ..default()
            },

            ..Default::default()
        },
        physical: PhysicalBundle {
            collider: Collider::ball(
                10.0),
            restitution: Restitution::new(1.0),
            velocity: Velocity { linvel: Vec2::ZERO, angvel: 0.0 },
            collision_layers: CollisionGroups::new(game_layer::PLAYER, game_layer::GROUND | game_layer::ENEMY),
            rigid_body: RigidBody::Dynamic,

            ..default()
        },
        name: Name::new("flask"),
        sensor: Default::default(),
        damage: DamageOnTouch { value: 100.0, ..default() },
        lifetime: Lifetime::from_seconds(2.0),
    };
    commands.spawn(bundle);
}

pub struct FireballSpawnData {
    gun: FireBallGun,
    position: Vec3,
    direction: Vec2,
    pub bullet_size: f32,
    pub pierce: u8,
    pub bullet_speed: f32,
}

impl LevelableData for FireballSpawnData {
    fn get_data_for_level(level: u8) -> Self {
        Self {
            gun: FireBallGun {},
            position: Default::default(),
            direction: Default::default(),
            bullet_size: 50_000.0 + (level as f32 * 1_000_f32),
            pierce: 0,
            bullet_speed: 400.0 + (level as f32 * 10.0),
        }
    }
}

pub fn spawn_fireball(
    In(data): In<FireballSpawnData>,
    atlases: ResMut<Atlases>,
    mut commands: Commands,
) {
    let speed = data.bullet_speed;
    let bundle = BulletBundle {
        sprite_sheet: AnimatedSpriteBundle {
            animator: SpriteAnimator::from_anim(AnimHandle::from_index(0)),
            spritesheet: atlases.sprite_sheets.get("bullets").expect("failed to find asset for bullet!").clone(),
            sprite_bundle: SpriteSheetBundle {
                transform: Transform::from_translation(data.position).with_scale(Vec3::new(2.0, 2.0, 0.0)),
                ..default()
            },

            ..Default::default()
        },
        physical: PhysicalBundle {
            collider: Collider::ball(
                0.5),
            restitution: Restitution::new(1.0),
            velocity: Velocity { linvel: data.direction * speed, angvel: 0.0 },
            collision_layers: CollisionGroups::new(game_layer::PLAYER, game_layer::GROUND | game_layer::ENEMY),
            rigid_body: RigidBody::Dynamic,

            ..default()
        },
        bullet: Bullet
        { pierce: data.pierce, ..default() },

        name: Name::new("bullets"),
        sensor: Default::default(),
        damage: DamageOnTouch { value: 5.0, ..default() },
        lifetime: Lifetime::from_seconds(2.0),
    };
    commands.spawn(bundle);
}