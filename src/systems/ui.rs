use bevy::prelude::*;
use bevy_asepritesheet::core::SpriteAnimController;
use bevy_egui::{egui, EguiContexts};
use bevy_egui::egui::emath;

use crate::AppState;
use crate::components::{AbilityLevel, Health, HealthUi, Player, XP};
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct LevelUpUiRoot;



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
            // let mut binding = parent.spawn(button());
            // let button1 = binding.insert(ButtonAction::OptionOne);
            // button1.with_children(|button| { button.spawn(button_text(&asset_server, "Option 1")); });
            // // Button two
            // let mut binding = parent.spawn(button());
            // let button1 = binding.insert(ButtonAction::OptionTwo);
            // button1.with_children(|parent| { parent.spawn(button_text(&asset_server, "Option 2")); });
            // // Button three
            // let mut binding = parent.spawn(button());
            // let button1 = binding.insert(ButtonAction::OptionThree);
            // button1.with_children(|parent| { parent.spawn(button_text(&asset_server, "Option 3")); });
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
    player_query: Query<(&Player, Entity)>,
    mut next_state: ResMut<NextState<AppState>>,
    commands: Commands,
    choices: Query<&LevelUpChoice>,
    mut abilities: Query<&mut AbilityLevel>,
    mut contexts: EguiContexts,
) {
    egui::CentralPanel::default().frame(egui::Frame {
        fill: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200), // Set background to transparent

        ..Default::default() // Use default settings for other frame properties
    })
        .show(contexts.ctx_mut(), |ui| {
            let screen_size = ui.available_size();
            let button_height = screen_size.y * 0.15;
            let button_width = screen_size.x * 0.5;


            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                for choice in choices.iter() {
                    let mut ability = abilities.get_mut(choice.entity_to_level).unwrap();
                    if ui.add(egui::Button::new(ability.description.to_string())//.fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 255))
                        // .image(egui::TextureId::User(i), [20.0, 20.0]) // Dummy image, replace with actual TextureId
                        .min_size(emath::Vec2::new(button_width, button_height)))
                        .clicked() {
                        // Handle button click
                        println!("Option {} clicked", ability.description);
                        ability.level += 1;
                        next_state.set(AppState::InGame);

                        break;
                    }
                }
            });


        });
}

#[derive(Component, Debug, Serialize, Deserialize)]
pub struct LevelUpChoice {
    // pub description: String,
    pub entity_to_level: Entity,
}

pub fn prepare_level_up(abilities: Query<(Entity, &AbilityLevel, &Name)>,
                        mut commands: Commands,
) {
    //randomly choose abilities to level
    //player may or may not have them already
    // commands.entity(player_query.single_mut()).
    for (entity, ability, name) in abilities.iter() {
        commands.spawn(LevelUpChoice {entity_to_level: entity });
    }
}

pub fn cleanup_level_up(
    mut commands: Commands,
    choices: Query<(Entity, &LevelUpChoice)>,
) {
    for (entity, choice) in choices.iter() {
        commands.entity(entity).despawn();
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
