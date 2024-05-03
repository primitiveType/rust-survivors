use std::process::Command;
use bevy::ecs::entity;
use bevy::math::{Vec2Swizzles, Vec3Swizzles};
use bevy::prelude::{Camera, GlobalTransform, KeyCode, Query, Res, ResMut, Resource, Transform, Vec2, Window, With};
use bevy::window::PrimaryWindow;
use crate::components::{Player, Reloadable, Reloading};
use bevy::prelude::*;
use bevy::prelude::KeyCode::KeyR;
use bevy::time::Timer;
/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
pub struct AimDirection(pub Vec2);

const RELOAD_KEY : KeyCode = KeyR;
pub fn get_aim_direction(
    player: Query<(&Transform), With<Player>>,
    mut mycoords: ResMut<AimDirection>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();


    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {

        let player_transform = player.single();


        mycoords.0 = (world_position - player_transform.translation.xy()).normalize();
        eprintln!("direction: {}/{}", mycoords.0.x, mycoords.0.y);

    }
}

/// This system prints 'A' key state
pub fn input_reload_gun_system(keyboard_input: Res<ButtonInput<KeyCode>>,
query : Query<(Entity, &Reloadable)>,
    mut commands: Commands
) {
    if keyboard_input.pressed(RELOAD_KEY) {
        info!("Reload started!");
        for (entity, gun) in query.iter() {
         commands.entity(entity).insert(Reloading{ timer: Timer::from_seconds(gun.reload_seconds_per_bullet, TimerMode::Repeating) });
        }
    }
}