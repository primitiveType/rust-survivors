use bevy::app::App;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use bevy_editor_pls::prelude::*;

pub(crate) fn add_inspector(app: &mut App) -> &mut App {
    app
        .add_plugins(EditorPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
}