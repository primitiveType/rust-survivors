use bevy::prelude::*;
use crate::AppState;

#[derive(Component)]
pub struct LevelUpUiRoot;

#[derive(Component, Copy, Clone)]
pub enum ButtonAction{
    OptionOne = 0,
    OptionTwo = 1,
    OptionThree = 2,
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // commands.spawn_bundle(UiCameraBundle::default());

    // UI root node
    commands.spawn(NodeBundle {
        style: Style {
            // size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,

            ..default()
        },
        visibility: Visibility::Hidden, // Start with the UI hidden
        ..default()
    }).insert(LevelUpUiRoot)
        .with_children(|parent| {
            // Button one
            let mut binding = parent.spawn(button());
            let button1 = binding.insert(ButtonAction::OptionOne);
            button1.with_children(|button| { button.spawn(button_text(&asset_server, "Option 1")); });
            // Button two
            let mut binding = parent.spawn(button());
            let button1 = binding.insert(ButtonAction::OptionTwo);
            button1.with_children(|parent| { parent.spawn(button_text(&asset_server, "Option 2")); });
            // Button three
            let mut binding = parent.spawn(button());
            let button1 = binding.insert(ButtonAction::OptionThree);
            button1.with_children(|parent| { parent.spawn(button_text(&asset_server, "Option 3")); });
        });
}

fn button() -> ButtonBundle {
    ButtonBundle {
        style: Style {
            // size: Size::new(Val::Px(150.0), Val::Px(65.0)),
            // margin: UiRoot::(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },

        background_color: Color::rgb(0.15, 0.15, 0.15).into(),
        ..default()
    }
}

fn button_text(_asset_server: &Res<AssetServer>, text: &str) -> TextBundle {
    TextBundle {
        text: Text::from_section(
            text,
            TextStyle {
                // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font: Default::default(),
                font_size: 30.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    }
}

pub fn button_system(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &ButtonAction), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut color, action) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::rgb(0.25, 0.25, 0.25).into();
                button_clicked(action, &mut next_state);
            }
            Interaction::Hovered => {
                *color = Color::rgb(0.35, 0.35, 0.35).into();
            }
            Interaction::None => {
                *color = Color::rgb(0.15, 0.15, 0.15).into();
            }
        }
    }
}

fn button_clicked(action : &ButtonAction, next_state: &mut ResMut<NextState<AppState>>){
    println!("Option {:?} clicked", *action as u8);

    next_state.set(AppState::InGame)
}

pub fn toggle_level_ui_system(
    mut query: Query<&mut Visibility, With<LevelUpUiRoot>>,
) {
    for mut visibility in query.iter_mut() {
        if *visibility == Visibility::Hidden {
            *visibility = Visibility::Inherited;
        } else if *visibility == Visibility::Inherited {
            *visibility = Visibility::Hidden;
        }
    }
}