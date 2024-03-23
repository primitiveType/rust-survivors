
pub mod game_layer {
    use bevy_rapier2d::prelude::Group;

    pub const PLAYER: Group = Group::GROUP_1; 
    pub const ENEMY: Group = Group::GROUP_2;
    pub const GROUND: Group = Group::GROUP_4;
    pub const XP: Group = Group::GROUP_5;
}
