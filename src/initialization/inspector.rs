use bevy::app::App;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::utils::default;
use bevy::window::Window;
use bevy_editor_pls::EditorWindowPlacement;

use bevy_editor_pls::prelude::*;

pub(crate) fn add_inspector(app: &mut App) -> &mut App {
    app
        .add_plugins(EditorPlugin { window: EditorWindowPlacement::New(Window{
            ..default()
        }) })
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
}