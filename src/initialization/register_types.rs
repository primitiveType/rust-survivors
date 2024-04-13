use bevy::app::App;

use crate::animation::AnimatorController;
use crate::components::{PassiveXPMultiplier, XPMultiplier, XP};

pub(crate) fn register_types(app: &mut App) -> &mut App {
    app.register_type::<AnimatorController>();
    app.register_type::<XP>();
    app.register_type::<XPMultiplier>();
    app.register_type::<PassiveXPMultiplier>();

    app
}
