use bevy::prelude::*;
use bevy_asepritesheet::core::SpriteAnimController;

use crate::AppState;
use crate::components::{Health, HealthUi, Player, XP};
use crate::initialization::load_prefabs::load_gun;

#[derive(Component)]
pub struct LevelUpUiRoot;


#[derive(Component, Copy, Clone)]
pub enum ButtonAction {
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
    mut player_query: Query<(&Player, Entity)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    for (interaction, mut color, action) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::rgb(0.25, 0.25, 0.25).into();
                button_clicked(action, &mut next_state, &mut commands, &mut player_query);
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

fn button_clicked(action: &ButtonAction,
                  next_state: &mut ResMut<NextState<AppState>>,
                  commands: &mut Commands,
                  player_query: &mut Query<(&Player, Entity)>) {
    println!("Option {:?} clicked", *action as u8);

    let (_player, player_entity) = player_query.single_mut();
    next_state.set(AppState::InGame);


    let mut gun_spawn = commands.spawn((load_gun(*action as usize), SpatialBundle { ..default() }));

    gun_spawn.set_parent(player_entity);
    //TODO: set translation to local zero :(
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


pub fn pause_animations(mut animation_timers: ResMut<SpriteAnimController>) {
    animation_timers.is_active = false;
}

pub fn resume_animations(mut animation_timers: ResMut<SpriteAnimController>) {
    animation_timers.is_active = true;
}


pub fn update_player_health_ui(player_query: Query<(&Health, &Player)>, player_xp_query: Query<(&XP, &Player)>, mut query: Query<&mut Text, With<HealthUi>>) {
    let mut text = query.single_mut();
    let (player_health, _) = player_query.single();
    let (xp, _) = player_xp_query.single();
    text.sections[1].value = player_health.value.to_string();
    text.sections[3].value = xp.amount.to_string();
}
