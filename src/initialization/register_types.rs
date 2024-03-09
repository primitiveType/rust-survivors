use bevy::app::App;

use crate::bundles::{AnimationIndices, AnimationTimer, Animator};

pub(crate) fn register_types(app: &mut App) -> &mut App {
    app.register_type::<AnimationIndices>();
    app.register_type::<AnimationTimer>();
    app.register_type::<Animator>();
    
    app
}