use bevy_xpbd_2d::prelude::PhysicsLayer;

#[derive(PhysicsLayer)]
pub enum GameLayer {
    Player, // Layer 0
    Enemy, // Layer 1
    Ball,  // Layer 2
    Ground, // Layer 3
    XP,
}