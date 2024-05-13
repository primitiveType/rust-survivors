use bevy::prelude::*;
use bevy_asepritesheet::core::SpriteAnimController;
use bevy_egui::egui::{emath, TextureOptions};
use bevy_egui::{egui, EguiContexts};
use egui::{Color32, SizeHint, TextureFilter};
use rand::seq::IteratorRandom;

use crate::components::{AbilityLevel, Ammo, ApplyColdOnTouch, Chambered, Cooldown, Health, HealthUi, Lifetime, Player, XP};
use crate::AppState;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct LevelUpUiRoot;

#[derive(Component)]
pub struct FadeTextWithLifetime {}

pub fn fade_text(mut query: Query<(&mut Text, &Lifetime), With<FadeTextWithLifetime>>) {
    for (mut text, lifetime) in query.iter_mut() {
        let alpha = 1.0 - lifetime.timer.fraction();
        text.sections[0].style.color.set_a(alpha);
    }
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

pub fn show_bullets(
    ammo_query: Query<(&Ammo, Option<&Children>)>,
    bullet_query: Query<(&Chambered, Option<&ApplyColdOnTouch>)>,
    cooldown_query: Query<(&Cooldown, &Name, &AbilityLevel)>,
    mut contexts: EguiContexts,
) {
    let panel = egui::panel::SidePanel::left("ammo panel").frame(egui::Frame {
        fill: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 0), // Set background to transparent

        ..Default::default() // Use default settings for other frame properties
    });


    let mut new_panel = panel.show(contexts.ctx_mut(), |ui| {
        for (cd, name, level) in cooldown_query.iter() {
            if (level.level == 0) {
                continue;
            }
            let fraction = cd.timer.fraction();
            ui.label(format!("{0} : {1:.2}/{2:.2}", name, cd.timer.elapsed().as_secs_f32(), cd.display_seconds()));
            ui.add(egui::widgets::ProgressBar::new(fraction).show_percentage());
        }
        let label_height = 8;


        let (clip, maybe_bullets) = ammo_query.single();
        // Add a flexible space to push the next elements to the bottom
        let bullet_height = 20.0;

        if let Some(bullets) = maybe_bullets
        {
            let num_bullets: usize = bullets.len();
            ui.add_space(ui.available_size().y - (num_bullets as f32 * (bullet_height + ui.style().spacing.item_spacing.y)));
            for bullet in bullets.iter() {
                let Ok((_, cold)) = bullet_query.get(*bullet) else { todo!() };
                let handle = egui::include_image!("E:/Unity Projects/rust-survivors/assets/sprites/ui-bullet.png");

                let mut tint =egui::Color32::from_rgb(255, 255, 255);

                if cold.is_some() {
                    tint = egui::Color32::from_rgb(0, 0, 255);
                }

                ui.add(
                    egui::Image::new(handle)
                        .texture_options(TextureOptions {
                            magnification: TextureFilter::Nearest,
                            minification: TextureFilter::Nearest,
                            wrap_mode: Default::default(),
                        })
                        .max_height(bullet_height)
                        .tint(tint)
                );
            }
        }

    });
}

pub fn button_system(
    player_query: Query<(&Player, Entity)>,
    mut next_state: ResMut<NextState<AppState>>,
    commands: Commands,
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
                        info!("Option {} clicked", ability.description);
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

    for (entity, ability, name) in abilities.iter().choose_multiple(&mut rng, num_choices) {
        commands.spawn(LevelUpChoice {
            entity_to_level: entity,
        });
    }
}

pub fn cleanup_level_up(mut commands: Commands, choices: Query<(Entity, &LevelUpChoice)>) {
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

pub fn update_player_health_ui(
    player_query: Query<(&Health, &Player)>,
    player_xp_query: Query<(&XP, &Player)>,
    mut query: Query<&mut Text, With<HealthUi>>,
) {
    let mut text = query.single_mut();
    let (player_health, _) = player_query.single();
    let (xp, _) = player_xp_query.single();
    text.sections[1].value = player_health.value.to_string();
    text.sections[3].value = xp.amount.to_string();
}
