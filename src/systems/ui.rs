use bevy::prelude::*;
use bevy_asepritesheet::core::SpriteAnimController;
use bevy_egui::egui::emath;
use bevy_egui::{egui, EguiContexts};
use rand::seq::IteratorRandom;

use crate::components::{AbilityLevel, Health, HealthUi, Lifetime, Player, XP};
use crate::AppState;
use serde::{Deserialize, Serialize};


#[derive(Component)]
pub struct FadeTextWithLifetime {}

pub fn fade_text(mut query: Query<(&mut Text, &Lifetime), With<FadeTextWithLifetime>>) {
    for (mut text, lifetime) in query.iter_mut() {
        let alpha = 1.0 - lifetime.timer.fraction();
        text.sections[0].style.color.set_a(alpha);
    }
}

pub fn button_system(
    _player_query: Query<(&Player, Entity)>,
    mut next_state: ResMut<NextState<AppState>>,
    _commands: Commands,
    choices: Query<&LevelUpChoice>,
    mut abilities: Query<&mut AbilityLevel>,
    mut contexts: EguiContexts,
) {
    egui::CentralPanel::default()
        .frame(egui::Frame {
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

pub fn prepare_level_up(abilities: Query<(Entity, &AbilityLevel, &Name)>, mut commands: Commands) {
    let num_choices = 3;
    //randomly choose abilities to level
    //player may or may not have them already
    // commands.entity(player_query.single_mut()).
    let mut rng = rand::thread_rng();

    for (entity, _ability, _name) in abilities.iter().choose_multiple(&mut rng, num_choices) {
        commands.spawn(LevelUpChoice {
            entity_to_level: entity,
        });
    }
}

pub fn cleanup_level_up(mut commands: Commands, choices: Query<(Entity, &LevelUpChoice)>) {
    for (entity, _choice) in choices.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn pause_animations(mut animation_timers: ResMut<SpriteAnimController>) {
    animation_timers.is_active = false;
}

pub fn resume_animations(mut animation_timers: ResMut<SpriteAnimController>) {
    animation_timers.is_active = true;
}

pub fn update_player_health_ui(
    _player_query: Query<(&Health, &Player)>,
    _player_xp_query: Query<(&XP, &Player)>,
    _query: Query<&mut Text, With<HealthUi>>,
) {
    // let mut text = query.single_mut();
    // let (player_health, _) = player_query.single();
    // let (xp, _) = player_xp_query.single();
    // text.sections[1].value = player_health.value.to_string();
    // text.sections[3].value = xp.amount.to_string();
}
