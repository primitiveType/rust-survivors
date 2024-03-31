use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;

use bevy::asset::{AssetServer, Handle};
use bevy::prelude::{Bundle, Commands, Res, ResMut, Resource, SpatialBundle};
use bevy_asepritesheet::core::load_spritesheet;
use bevy_asepritesheet::prelude::Spritesheet;
use serde::Deserialize;
use serde::Serialize;

use crate::bundles::{EnemyBundle, EnemyData};
use crate::components::Cooldown;

//on startup, load all images
//have texture layouts loaded as well, possibly as files next to the images?
//load texture layouts into dictionary
//enemies know their name and can get sprite+layout based on name and action

#[derive(Bundle, Clone, Debug, Default, Serialize, Deserialize)]
pub struct GunBundle {
    pub gun: Cooldown,
    #[serde(skip)]
    pub spatial_bundle: SpatialBundle,
}

#[derive(Resource)]
pub struct Atlases {
    //assettype/assetname
    //eg. enemy/knight

    pub sprite_sheets: HashMap<String, Handle<Spritesheet>>,
}

#[derive(Resource)]
pub struct Enemies {
    pub datas: HashMap<String, EnemyBundle>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AtlasLayout {
    pub cols: usize,
    pub rows: usize,
    pub height_px: u16,
    pub width_px: u16,
}


const GUNS_PATH: &str = "E:\\Unity Projects\\rust-survivors\\assets\\prefabs\\guns\\";
const ENEMIES_PATH: &str = "E:\\Unity Projects\\rust-survivors\\assets\\prefabs\\enemies\\";


pub fn load_sprites(mut commands: Commands, asset_server: Res<AssetServer>, mut atlases: ResMut<Atlases>) {
    //todo: cache this and store in what is currently called atlases.
    let sheet_handle = load_spritesheet(
        &mut commands,
        &asset_server,
        "bullets.json",
        bevy::sprite::Anchor::Center,
    );

    atlases.sprite_sheets.insert("bullet".to_string(), sheet_handle);

    let sheet_handle = load_spritesheet(
        &mut commands,
        &asset_server,
        "prototype_char.json",
        bevy::sprite::Anchor::Center,
    );

    atlases.sprite_sheets.insert("player".to_string(), sheet_handle);

    let sheet_handle = load_spritesheet(
        &mut commands,
        &asset_server,
        "Skeleton.json",
        bevy::sprite::Anchor::Center,
    );


    atlases.sprite_sheets.insert("skeleton".to_string(), sheet_handle);

    let sheet_handle = load_spritesheet(
        &mut commands,
        &asset_server,
        "bat-all.json",
        bevy::sprite::Anchor::Center,
    );

    atlases.sprite_sheets.insert("bat".to_string(), sheet_handle);

    // let sheet_handle = load_spritesheet_then(
    //     commands,
    //     asset_server,
    //     sprite_sheet,
    //     bevy::sprite::Anchor::Center,
    //     |sheet| {
    //         let explode = sheet.get_anim_handle(crate::systems::guns::FIREBALL_EXPLODE_ANIMATION);//TODO: i guess its not possible to pass a satring to spawn_particle for the animation. consider using the same animation name
    //         let mut explode_mut = sheet.get_anim_mut(&explode);
    //         explode_mut.unwrap().end_action = AnimEndAction::Stop;
    //     },
    // );
    //
    // atlases.sprite_sheets.insert(FIREBALL_EXPLODE_ANIMATION, sheet_handle);
}


pub fn load_enemy_prefabs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut enemies: ResMut<Enemies>,
    atlases: ResMut<Atlases>,
) {
    let paths: Vec<DirEntry> = fs::read_dir(ENEMIES_PATH).unwrap().filter_map(|entry| entry.ok()).collect();
    for dir in paths.iter() {
        let enemy_name = dir.path().with_extension("").file_name().unwrap().to_str().unwrap().to_string();//are you serious
        enemies.datas.insert(enemy_name, EnemyBundle::from_path(dir.path().to_str().unwrap(), &atlases));
    }
}


pub fn load_gun(gun: usize) -> Cooldown {
    let paths: Vec<DirEntry> = fs::read_dir(GUNS_PATH).unwrap().filter_map(|entry| entry.ok()).collect();

    let paths_count = paths.len();
    let path = &paths[gun % paths_count];
    let file_path = path.path();
    println!("Loaded gun {}", file_path.display());
    let gun_yaml = fs::read_to_string(file_path).expect("failed to load yaml!");
    let gun = serde_yaml::from_str::<Cooldown>(gun_yaml.as_str()).expect("failed to deserialize gun!");

    gun
}

pub fn _save_enemy(bundle: EnemyData) {
    let enemy_yaml = serde_yaml::to_string(&bundle).expect("Unable to serialize!");
    fs::write(ENEMIES_PATH, enemy_yaml).expect("Unable to write file!");
}

// pub fn load_enemy(
//     enemy: usize,
//     atlases: ResMut<Atlases>,
// ) -> EnemyBundle {
//     let file_path = get_enemy_path(enemy);
//     EnemyBundle::from_path(file_path.to_str().unwrap(), atlases)
// }


fn get_enemy_path(index: usize) -> PathBuf {
    let paths: Vec<DirEntry> = fs::read_dir(ENEMIES_PATH).unwrap().filter_map(|entry| entry.ok()).collect();
    let paths_count = paths.len();
    let path = &paths[index % paths_count];
    path.path()
}

pub fn _load_enemy_data(enemy: usize) -> EnemyData {
    let file_path = get_enemy_path(enemy);

    load_enemy_data_from_path(file_path.to_str().unwrap())
}

pub fn load_enemy_data_from_path(path: &str) -> EnemyData {
    load_data_from_path::<EnemyData>(path)
}

pub fn load_data_from_path<T: for<'a> Deserialize<'a>>(path: &str) -> T {
    let enemy_yaml = fs::read_to_string(path).expect("failed to load yaml!");
    let enemy = serde_yaml::from_str::<T>(enemy_yaml.as_str()).unwrap_or_else(|_| panic!("failed to deserialize data at path {}!", path));
    enemy
}

pub fn load_gun_test() {
    // load_gun(999);
}