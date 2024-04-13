use bevy::asset::{Assets, Handle};
use bevy::ecs::query::QueryEntityError;
use bevy::log::tracing_subscriber::fmt::time;
use bevy::math::{Vec3, Vec3Swizzles};
use bevy::prelude::{
    default, BuildChildren, Color, Commands, Component, Entity, EventReader, GlobalTransform, In,
    Mut, Query, Res, ResMut, Sprite, SpriteSheetBundle, Text, Text2dBundle, TextStyle, Time,
    Transform, Vec2, With, Without,
};
use bevy::time::TimerMode::Once;
use bevy::time::{Timer, TimerMode};
use bevy_asepritesheet::animator::{AnimFinishEvent, AnimatedSpriteBundle, SpriteAnimator};
use bevy_asepritesheet::prelude::{AnimEventSender, AnimHandle, Spritesheet};
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::geometry::{Collider, CollisionGroups, Restitution};
use bevy_rapier2d::na::clamp;
use bevy_rapier2d::parry::math::DEFAULT_EPSILON;
use bevy_rapier2d::pipeline::CollisionEvent;
use bevy_rapier2d::plugin::RapierContext;
use bevy_rapier2d::prelude::{QueryFilter, Velocity};
use rand::Rng;
use std::time::Duration;
use temporary_component_derive::*;

use crate::bundles::{DestroyAfterDeathAnimation, Object, PhysicalBundle};
use crate::components::{AbilityLevel, ApplyColdOnTouch, AttackSpeed, Bullet, BulletBundle, Cold, Cooldown, DamageOnTouch, Enemy, Expired, FireBallGun, Flask, FlaskProjectileBundle, Health, IceBallGun, Lifetime, MoveSpeed, TemporaryComponent};
use crate::constants::{BACKGROUND_PROJECTILE_LAYER, DAMAGE_TEXT_LAYER, PIXEL_SCALE};
use crate::extensions::spew_extensions::{Spawn, Spawner};
use crate::extensions::vectors::to_vec2;
use crate::initialization::load_prefabs::Atlases;
use crate::physics::layers::game_layer;
use crate::systems::ui::FadeTextWithLifetime;
use crate::Name;

//adding a new ability
//1.. add system that does the ability thing. It should require an AbilityLevel component
//2.. add level 0 bundle on player in spawn_player
//3.. add system for updating the description of the ability when its level changes
//4.. add all 4 systems to update loop.
pub fn advance_cooldowns(
    mut query: Query<&mut Cooldown>,
    mut cdr_query: Query<&mut AttackSpeed>,
    time: Res<Time>,
) {
    //assumes only player needs concept of abilities and CDR, which might change.
    for mut ability in query.iter_mut() {
        let mut total_cdr = 100.0f32;
        for cdr in cdr_query.iter_mut() {
            total_cdr += cdr.percent;
        }

        let multiplier = total_cdr * 0.01f32; //convert percentage to multiplier
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

pub fn iceball_gun(
    mut query: Query<(&mut Cooldown, &GlobalTransform, &IceBallGun, &AbilityLevel)>,
    mut spawner: Spawner<IceballSpawnData>,
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
                    groups: Some(CollisionGroups::new(game_layer::ENEMY, game_layer::ENEMY)), //is this filter correct?
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

                let mut spawn_data = IceballSpawnData::get_data_for_level(level.level);
                spawn_data.position = translation;
                spawn_data.direction = delta;
                spawner.spawn(Object::Iceball, spawn_data);
                // spawn_fireball(&mut commands, &gun, translation, delta, &atlases);
            }
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
                    groups: Some(CollisionGroups::new(game_layer::ENEMY, game_layer::ENEMY)), //is this filter correct?
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

pub fn apply_cold_on_collide(
    mut collision_events: EventReader<CollisionEvent>,
    mut enemy_query: Query<(Entity), (With<Enemy>, With<MoveSpeed>)>,
    mut damage_query: Query<(&ApplyColdOnTouch)>,
    mut commands: Commands,
) {
    for collision_event in collision_events.read() {
        //need unique tags for icy, etc. So a given effect only applies once...
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _flags) => {
                {
                    //entity 2 damages entity 1 if it can
                    let slowed_entity = enemy_query.get_mut(*entity1);
                    let slowing_entity = damage_query.get_mut(*entity2);

                    try_slow(slowing_entity, slowed_entity, &mut commands);
                }
                {
                    //entity 1 damages entity 2 if it can
                    let entity2_health = enemy_query.get_mut(*entity2);
                    let entity1_damage = damage_query.get_mut(*entity1);
                    try_slow(entity1_damage, entity2_health, &mut commands);
                }
            }
            _ => {}
        }
    }
}

#[derive(Component, TemporaryComponent)]
pub struct Damaged {
    timer: Timer,
    //consider using trait_queries crate for the common pattern of having a component that needs to be removed
    //when its timer expires. I think statuses will probably all do this.
    //may not work due to remove function being generic though...
}



pub fn process_temporary_component<T>(
    mut damaged: Query<(Entity, &mut T)>,
    time: Res<Time>,
    mut commands: Commands,
) where
    T: TemporaryComponent + Component,
{
    for (entity, mut damaged) in damaged.iter_mut() {
        damaged.advance_timer(time.delta());
        if (damaged.is_finished()) {
            commands.entity(entity).remove::<T>();
        }
    }
}

pub fn deal_damage_on_collide(
    mut collision_events: EventReader<CollisionEvent>,
    mut health_query: Query<(Entity, &mut Health, &Transform), Without<Damaged>>,
    enemy_query: Query<(Entity, &Enemy)>, //HACK do something smarter.
    mut damage_query: Query<(Entity, &mut DamageOnTouch)>,
    mut spawner: Spawner<DamageTextSpawnData>,
    mut commands: Commands,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _flags) => {
                {
                    //entity 2 damages entity 1 if it can
                    let entity1_health = health_query.get_mut(*entity1);
                    let entity2_damage = damage_query.get_mut(*entity2);

                    let enemy1 = enemy_query.get(*entity1);
                    let enemy2 = enemy_query.get(*entity2);
                    if enemy1.is_ok() && enemy2.is_ok() {
                        continue;
                    }
                    try_deal_damage(&mut commands, entity2_damage, entity1_health, &mut spawner);
                }
                {
                    //entity 1 damages entity 2 if it can
                    let entity2_health = health_query.get_mut(*entity2);
                    let entity1_damage = damage_query.get_mut(*entity1);
                    try_deal_damage(&mut commands, entity1_damage, entity2_health, &mut spawner);
                }
            }
            _ => {}
        }
    }
}

pub const DEFAULT_I_FRAMES: f32 = 0.25f32;

fn try_deal_damage(
    commands: &mut Commands,
    entity1_damage: Result<(Entity, Mut<DamageOnTouch>), QueryEntityError>,
    entity2_health: Result<(Entity, Mut<Health>, &Transform), QueryEntityError>,
    spawner: &mut Spawner<DamageTextSpawnData>,
) {
    match (entity1_damage, entity2_health) {
        (Ok((_, mut damage)), Ok((health_entity, mut health, transform))) => {
            health.value -= damage.value;
            damage.count_triggers += 1;
            commands.entity(health_entity).insert(Damaged {
                timer: Timer::from_seconds(DEFAULT_I_FRAMES, Once),
            });
            spawner.spawn(
                Object::DamageNumber,
                DamageTextSpawnData {
                    position: transform.translation.xy(),
                    amount: damage.value as u32,
                },
            )
        }
        _ => {}
    }
}

fn try_slow(
    entity1_damage: Result<&ApplyColdOnTouch, QueryEntityError>,
    entity2_health: Result<Entity, QueryEntityError>,
    mut commands: &mut Commands,
) {
    match (entity1_damage, entity2_health) {
        (Ok((slow)), Ok((entity))) => {
            // let mut child = commands.spawn((ParentMoveSpeedMultiplier { value: -slow.multiplier }, Lifetime::from_seconds(slow.seconds)));
            // child.insert(Name ::new("slow effect"));
            // child.set_parent(entity);
            commands.entity(entity).insert(Cold {
                multiplier: slow.multiplier,
                timer: Timer::from_seconds(slow.seconds, TimerMode::Once),
            });
        }
        _ => {}
    }
}

pub fn expire_bullets_on_hit(
    mut bullets: Query<(&mut Bullet, Entity, &Transform, &DamageOnTouch)>,
    mut commands: Commands,
) {
    for (bullet, entity, transform, damage) in bullets.iter_mut() {
        if damage.count_triggers > bullet.pierce.into() {
            commands.entity(entity).insert(Expired {});
        }
    }
}

pub fn expired_bullets_explode(
    mut bullets: Query<(Entity, &Bullet, &Transform, &Name), With<Expired>>,
    mut commands: Commands,
    atlases: Res<Atlases>,
    sprite_assets: Res<Assets<Spritesheet>>,
    mut spawner: Spawner<ParticleSpawnData>,
) {
    for (bullet, entity, transform, name) in bullets.iter_mut() {
        spawner.spawn(
            Object::Particle,
            ParticleSpawnData {
                position: transform.translation.xy(),
                scale: Vec2::splat(1.0),
                sprite_sheet: name.to_string(),
                color: Color::rgb(0.9, 0.3, 0.0),
                animation: "Dead".to_string(),
                parent: None,
                lifetime: Lifetime::from_seconds(1.0),
            },
        );
        // spawn_particle(transform.translation, &mut commands, name.to_string(), "Dead", &atlases, &sprite_assets);
    }
}

pub fn expire_entities(
    mut lifetimes: Query<(Entity, &mut Lifetime)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in lifetimes.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).insert(Expired {});
        }
    }
}

pub fn destroy_expired_entities(mut lifetimes: Query<(Entity, &Expired)>, mut commands: Commands) {
    for (entity, _) in lifetimes.iter_mut() {
        commands.entity(entity).despawn();
    }
}

#[derive(Default)]
pub struct ParticleSpawnData {
    pub scale: Vec2,
    pub position: Vec2,
    pub sprite_sheet: String,
    pub color: Color,
    pub animation: String,
    pub parent: Option<Entity>,
    pub lifetime: Lifetime,
}

pub const PARTILE_STATUS_LAYER: f32 = 4.0;

pub fn spawn_particle(
    In(data): In<ParticleSpawnData>,
    mut commands: Commands,
    atlases: Res<Atlases>,
    sprite_assets: Res<Assets<Spritesheet>>,
) {
    let spritesheet = atlases
        .sprite_sheets
        .get(&data.sprite_sheet)
        .expect("failed to find particle animation!")
        .clone();
    let mut anim_handle = AnimHandle::from_index(0);
    // Attempt to get the asset using the handle
    if let Some(asset) = sprite_assets.get(&spritesheet) {
        // Now you have access to the asset (`T`) here
        // Do something with the asset
        anim_handle = asset.get_anim_handle(data.animation);
    } else {
        // The asset is not loaded yet, you might handle this case accordingly
        println!("Asset not loaded yet");
    }
    let parent_exists =
        data.parent.is_some() && commands.get_entity(data.parent.unwrap()).is_some();
    let mut spawned = commands.spawn((
        data.lifetime,
        AnimEventSender,
        // Cold{ multiplier: 0.0, timer: Timer::from_seconds(100.0, TimerMode::Repeating) },
        AnimatedSpriteBundle {
            animator: SpriteAnimator::from_anim(anim_handle),
            spritesheet,
            sprite_bundle: SpriteSheetBundle {
                transform: Transform::from_translation(data.position.extend(PARTILE_STATUS_LAYER))
                    .with_scale(data.scale.extend(1.0)),
                sprite: Sprite {
                    color: data.color,
                    ..default()
                },
                ..default()
            },

            ..Default::default()
        },
    ));
    if parent_exists {
        spawned.set_parent(data.parent.unwrap());
    }
}

pub fn destroy_after_death_anim(
    mut commands: Commands,
    mut events: EventReader<AnimFinishEvent>,
    spritesheet_assets: Res<Assets<Spritesheet>>,
    animated_sprite_query: Query<
        &Handle<Spritesheet>,
        (With<SpriteAnimator>, With<DestroyAfterDeathAnimation>),
    >,
) {
    for event in events.read() {
        // get the spritesheet handle off the animated sprite entity
        if let Ok(sheet_handle) = animated_sprite_query.get(event.entity) {
            if let Some(anim_sheet) = spritesheet_assets.get(sheet_handle) {
                // get the animation reference from the spritesheet
                if let Ok(anim) = anim_sheet.get_anim(&event.anim) {
                    if anim.name == "Dead" {
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
    pub cooldown: f32,
    pub damage: f32,
    duration_seconds: f32,
}

impl LevelableData for FlaskSpawnData {
    fn get_data_for_level(level: u8) -> Self {
        Self {
            gun: Flask {},
            position: Default::default(),
            scale: 5.0 + (level as f32),
            cooldown: clamp(10.0 - (1.25 * level as f32), 0.5, 100.0),
            damage: 1.0,
            duration_seconds: 2.0,
        }
    }
}

pub trait LevelableData {
    fn get_data_for_level(level: u8) -> Self;
}

pub fn spawn_damage_text(In(data): In<DamageTextSpawnData>, mut commands: Commands) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                data.amount.to_string(),
                TextStyle {
                    font: Default::default(),
                    /* Load or reference your font here */
                    font_size: 40.0,
                    color: Color::WHITE,
                },
            ),
            transform: Transform::from_translation(
                (data.position + Vec2::new(0.0, 50.0)).extend(DAMAGE_TEXT_LAYER),
            ), // Offset above the enemy
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
            spritesheet: atlases
                .sprite_sheets
                .get("fireball")
                .expect("failed to find asset for bullet!")
                .clone(),
            sprite_bundle: SpriteSheetBundle {
                transform: Transform::from_translation(
                    data.position.extend(BACKGROUND_PROJECTILE_LAYER),
                )
                .with_scale(Vec2::splat(data.scale).extend(1.0)),
                ..default()
            },

            ..Default::default()
        },
        physical: PhysicalBundle {
            collider: Collider::ball(10.0),
            restitution: Restitution::new(1.0),
            velocity: Velocity {
                linvel: Vec2::ZERO,
                angvel: 0.0,
            },
            collision_layers: CollisionGroups::new(
                game_layer::PLAYER,
                game_layer::GROUND | game_layer::ENEMY,
            ),
            rigid_body: RigidBody::Dynamic,

            ..default()
        },
        name: Name::new("flask"),
        sensor: Default::default(),
        damage: DamageOnTouch {
            value: data.damage,
            ..default()
        },
        lifetime: Lifetime::from_seconds(data.duration_seconds),
    };
    commands.spawn(bundle);
}

pub struct FireballSpawnData {
    pub damage: f32,
    position: Vec3,
    direction: Vec2,
    pub bullet_size: f32,
    pub pierce: u8,
    pub bullet_speed: f32,
}

pub struct IceballSpawnData {
    pub damage: f32,
    pub slow_amount: f32,
    pub slow_seconds: f32,
    pub bullet_lifetime_seconds: f32,
    position: Vec3,
    direction: Vec2,
    pub bullet_size: f32,
    pub pierce: u8,
    pub bullet_speed: f32,
}

impl LevelableData for IceballSpawnData {
    fn get_data_for_level(level: u8) -> Self {
        Self {
            damage: 0.0,
            slow_amount: 0.4,
            slow_seconds: 1.0,
            bullet_lifetime_seconds: 2.0,
            position: Default::default(),
            direction: Default::default(),
            bullet_size: 1.0,
            pierce: level.clamp(1, 255) - 1,
            bullet_speed: 400.0 + (level as f32 * 10.0),
        }
    }
}

impl LevelableData for FireballSpawnData {
    fn get_data_for_level(mut level: u8) -> Self {
        level = level - 1;
        Self {
            damage: 1.0 + (level as f32).floor(),
            position: Default::default(),
            direction: Default::default(),
            bullet_size: 1.0 + (level as f32 * 0.1_f32),
            pierce: (level as f32 * 0.25).floor() as u8,
            bullet_speed: 400.0 + (level as f32 * 10.0),
        }
    }
}

pub fn spawn_iceball(
    In(data): In<IceballSpawnData>,
    atlases: ResMut<Atlases>,
    mut commands: Commands,
) {
    let speed = data.bullet_speed;
    let base_size = 2.0;

    let bundle = BulletBundle {
        sprite_sheet: AnimatedSpriteBundle {
            animator: SpriteAnimator::from_anim(AnimHandle::from_index(0)),
            spritesheet: atlases
                .sprite_sheets
                .get("snowball")
                .expect("failed to find asset for bullet!")
                .clone(),
            sprite_bundle: SpriteSheetBundle {
                transform: Transform::from_translation(data.position).with_scale(Vec3::new(
                    base_size * data.bullet_size,
                    base_size * data.bullet_size,
                    0.0,
                )),
                ..default()
            },

            ..Default::default()
        },
        physical: PhysicalBundle {
            collider: Collider::ball(data.bullet_size),
            restitution: Restitution::new(1.0),
            velocity: Velocity {
                linvel: data.direction * speed,
                angvel: 0.0,
            },
            collision_layers: CollisionGroups::new(
                game_layer::PLAYER,
                game_layer::GROUND | game_layer::ENEMY,
            ),
            rigid_body: RigidBody::Dynamic,

            ..default()
        },
        bullet: Bullet {
            pierce: data.pierce,
            ..default()
        },

        name: Name::new("fireball"),
        sensor: Default::default(),
        damage: DamageOnTouch {
            value: data.damage,
            ..default()
        },
        lifetime: Lifetime::from_seconds(data.bullet_lifetime_seconds),
    };
    let mut bullet = commands.spawn(bundle);

    bullet.insert(ApplyColdOnTouch {
        multiplier: data.slow_amount,
        seconds: data.slow_seconds,
    });
}

pub fn spawn_fireball(
    In(data): In<FireballSpawnData>,
    atlases: ResMut<Atlases>,
    mut commands: Commands,
) {
    let base_size = 2.0;
    let sprite = atlases
        .sprite_sheets
        .get("fireball")
        .expect("failed to find asset for bullet!")
        .clone();
    let speed = data.bullet_speed;
    let bundle = BulletBundle {
        sprite_sheet: AnimatedSpriteBundle {
            animator: SpriteAnimator::from_anim(AnimHandle::from_index(0)),
            spritesheet: sprite,
            sprite_bundle: SpriteSheetBundle {
                transform: Transform::from_translation(data.position).with_scale(Vec3::new(
                    base_size * data.bullet_size,
                    base_size * data.bullet_size,
                    1.0,
                )),
                ..default()
            },

            ..Default::default()
        },
        physical: PhysicalBundle {
            collider: Collider::ball(PIXEL_SCALE * data.bullet_size),
            restitution: Restitution::new(1.0),
            velocity: Velocity {
                linvel: data.direction * speed,
                angvel: 0.0,
            },
            collision_layers: CollisionGroups::new(
                game_layer::PLAYER,
                game_layer::GROUND | game_layer::ENEMY,
            ),
            rigid_body: RigidBody::Dynamic,

            ..default()
        },
        bullet: Bullet {
            pierce: data.pierce,
            ..default()
        },

        name: Name::new("fireball"),
        sensor: Default::default(),
        damage: DamageOnTouch {
            value: data.damage,
            ..default()
        },
        lifetime: Lifetime::from_seconds(2.0),
    };
    commands.spawn(bundle);
}
