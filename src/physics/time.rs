use bevy::prelude::*;
use bevy::prelude::ResMut;

use crate::time;
use crate::time::PhysicsTimeExt;

pub fn pause(mut time: ResMut<time::PhysicsTime>,) {
    time.pause();
}

pub fn unpause(mut time: ResMut<time::PhysicsTime>,) {
    time.resume();
}