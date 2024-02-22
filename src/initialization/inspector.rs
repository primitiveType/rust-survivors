use bevy::app::App;
use bevy::prelude::*;
use bevy_editor_pls::prelude::*;

pub(crate) fn add_inspector(app: &mut App) -> &mut App {
    app
        .add_plugins(EditorPlugin::default())
}