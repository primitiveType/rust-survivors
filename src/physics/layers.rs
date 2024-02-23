use bevy_xpbd_2d::prelude::PhysicsLayer;

#[derive(PhysicsLayer)]
pub enum GameLayer {
    Player, // Layer 0
    Ball,  // Layer 1
    Ground, // Layer 2
}