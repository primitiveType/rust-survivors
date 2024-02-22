use bevy::math::{Vec2, Vec3};
pub(crate) fn to_vec2(vec3 : Vec3) -> Vec2{
    Vec2::new(vec3.x, vec3.y)
}