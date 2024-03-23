use bevy::app::App;

use crate::animation::AnimatorController;

pub(crate) fn register_types(app: &mut App) -> &mut App {
    app.register_type::<AnimatorController>();

    app
}