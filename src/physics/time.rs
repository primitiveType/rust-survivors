use bevy::prelude::{ResMut, Time};
use bevy_xpbd_2d::prelude::{Physics, PhysicsTime};

pub fn pause(mut time: ResMut<Time<Physics>>) {
    time.pause();
}

pub fn unpause(mut time: ResMut<Time<Physics>>) {
    time.unpause();
}