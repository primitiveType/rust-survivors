use std::fmt;

use bevy::asset::{Assets, Handle};
use bevy::prelude::Changed;
use bevy::prelude::Component;
use bevy::prelude::Image;
use bevy::prelude::Query;
use bevy::prelude::Reflect;
use bevy::prelude::Res;
use bevy::prelude::Sprite;
use bevy_asepritesheet::prelude::{AnimHandle, SpriteAnimator, Spritesheet};
use bevy_rapier2d::dynamics::Velocity;
use serde::{Deserialize, Serialize};

use crate::initialization::load_prefabs::Atlases;

pub fn set_spritesheet_from_animation_info(
    atlases: Res<Atlases>,
    mut query: Query<(&AnimatorController, &mut SpriteAnimator, &Handle<Image>), Changed<AnimatorController>>,
    sprite_assets: Res<Assets<Spritesheet>>,
) {
    for (animator_controller, mut animator, _image) in &mut query.iter_mut() {

        //we have the entities spritesheet name, and animation name 
        //get the spritesheet asset handle
        if let Some(spritesheet) = atlases.sprite_sheets.get(&animator_controller.name) {
            let mut anim_handle = AnimHandle::from_index(0);
            // Attempt to get the spritesheet asset so we can get animations by name
            if let Some(asset) = sprite_assets.get(&spritesheet.clone()) {
                anim_handle = asset.get_anim_handle(&animator_controller.state.to_string());
            } else {
                // The asset is not loaded yet, you might handle this case accordingly
                println!("Animation not loaded yet : {}", animator_controller.state);
            }
            animator.set_anim(anim_handle);
        } else {
            println!("Failed to find Spritesheet {}!", animator_controller.name);
        }
    }
}


#[derive(Component, Deserialize, Serialize, Debug, Clone)]
pub struct SpritePath(pub String);


pub fn update_animation_state(
    mut query: Query<(&mut AnimatorController, &Velocity)>, )
{
    for (mut animator, velocity) in &mut query.iter_mut() {
        if velocity.linvel.length() == 0.0 {
            animator.state = AnimationState::Idle;
        } else if animator.state != AnimationState::Walk {//looks stupid, but necessary to not trigger a change.
            animator.state = AnimationState::Walk;
        }
    }
}


pub fn flip_sprite(
    mut query: Query<(&mut Sprite, &Velocity)>,
) {
    for (mut atlas, velocity) in &mut query {
        if velocity.linvel.x == 0.0 {
            continue;
        }
        atlas.flip_x = velocity.linvel.x < 0.0;
    }
}


#[derive(Component, Reflect)]
pub struct AnimatorController {
    pub state: AnimationState,
    pub name: String,
}

#[derive(Debug, Reflect, PartialEq)]
pub enum AnimationState {
    Walk,
    Idle,
}

impl fmt::Display for AnimationState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}