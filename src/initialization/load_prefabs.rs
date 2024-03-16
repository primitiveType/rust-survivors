use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;

use bevy::asset::{Assets, AssetServer, Handle};
use bevy::math::vec2;
use bevy::prelude::{Bundle, Image, Res, ResMut, Resource, SpatialBundle};
use bevy::sprite::TextureAtlasLayout;
use serde::Deserialize;
use serde::Serialize;
use walkdir::WalkDir;

use crate::bundles::{AnimationIndices, EnemyBundle, EnemyData};
use crate::components::Gun;

//on startup, load all images
//have texture layouts loaded as well, possibly as files next to the images?
//load texture layouts into dictionary
//enemies know their name and can get sprite+layout based on name and action

#[derive(Bundle, Clone, Debug, Default, Serialize, Deserialize)]
pub struct GunBundle {
    pub gun: Gun,
    #[serde(skip)]
    pub spatial_bundle: SpatialBundle,
}

#[derive(Resource)]
pub struct Atlases {
    //assettype/assetname
    //eg. enemy/knight
    pub map: HashMap<String, Handle<TextureAtlasLayout>>,
    pub image_map: HashMap<String, Handle<Image>>,
}

#[derive(Resource)]
pub struct Animations {
    //assettype/assetname/animation
    //eg. enemy/knight/run
    pub map: HashMap<String, AnimationIndices>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AtlasLayout {
    pub cols: usize,
    pub rows: usize,
    pub height_px: u16,
    pub width_px: u16,
}

// fn serialize_spatial_bundle<S>(x: &SpatialBundle, s: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
// {
//     // pub visibility: Visibility,
//     // /// The inherited visibility of the entity.
//     // pub inherited_visibility: InheritedVisibility,
//     // /// The view visibility of the entity.
//     // pub view_visibility: ViewVisibility,
//     // /// The transform of the entity.
//     // pub transform: Transform,
//     // /// The global transform of the entity.
//     // pub global_transform: GlobalTransform,
//     s.ser()
// }

const GUNS_PATH: &str = "E:\\Unity Projects\\rust-survivors\\assets\\prefabs\\guns\\";
const ENEMIES_PATH: &str = "E:\\Unity Projects\\rust-survivors\\assets\\prefabs\\enemies\\";
const SPRITES_PATH: &str = "E:\\Unity Projects\\rust-survivors\\assets\\sprites";


pub fn load_sprites(
    asset_server: ResMut<AssetServer>,
    mut atlases: ResMut<Atlases>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut animations: ResMut<Animations>,
) {
    for entry in WalkDir::new(SPRITES_PATH).into_iter().filter_map(|e| e.ok()) {
        println!("{}", entry.path().display());
        let path = entry.path().to_str().unwrap();
        let extension = entry.path().extension();
        let parent = entry.path().parent().unwrap().file_name().unwrap().to_str().unwrap();

        if extension == Some("layout".as_ref()) {
            continue;
        } else if extension == Some("yml".as_ref()) {
            let asset_path_str = path.to_string();
            //todo...
            let path_without_extensions = entry.path().with_extension("");
            let asset_filename_no_extension = path_without_extensions.file_name().unwrap().to_str().unwrap();

            let indices = load_data_from_path::<AnimationIndices>(path);
            animations.map.insert(format!("{}_{}", parent, asset_filename_no_extension), indices);
            println!("inserted animation {}", asset_path_str);
        } else {
            let tester = entry.path().with_extension("");
            let asset_filename_no_extension = tester.file_name().unwrap().to_str().unwrap();

            let yml_path = entry.path().with_extension("layout");
            if yml_path.exists() {
                let asset_path_str = entry.path().to_str().unwrap().to_string();
                let handle: Handle<Image> = asset_server.load(asset_path_str.clone());
                let atlas = load_data_from_path::<AtlasLayout>(yml_path.to_str().unwrap());
                println!("Loaded atlas {:?}", atlas);
                let layout = TextureAtlasLayout::from_grid(vec2(atlas.width_px as f32,
                                                                atlas.height_px as f32),
                                                           atlas.cols,
                                                           atlas.rows,
                                                           None, None);
                let layout_handle = layouts.add(layout);
                let key = format!("{}_{}", parent, asset_filename_no_extension);
                atlases.map.insert(key.clone(), layout_handle);
                atlases.image_map.insert(key, handle);
            }
        }
    }
}


// pub fn save_gun( ) {
//     let gun_bundle: Gun =
//          Gun { last_shot_time: 0, cooldown: 200 };
//
//     let gun_yaml = serde_yaml::to_string(&gun_bundle).expect("Unable to serialize!");
//     fs::write(PATH, gun_yaml).expect("Unable to write file!");
// }
pub fn load_gun(gun: usize) -> Gun {
    let paths: Vec<DirEntry> = fs::read_dir(GUNS_PATH).unwrap().filter_map(|entry| entry.ok()).collect();

    let paths_count = paths.len();
    let path = &paths[gun % paths_count];
    let file_path = path.path();
    println!("Loaded gun {}", file_path.display());
    let gun_yaml = fs::read_to_string(file_path).expect("failed to load yaml!");
    let gun = serde_yaml::from_str::<Gun>(gun_yaml.as_str()).expect("failed to deserialize gun!");

    gun
}

pub fn _save_enemy(bundle: EnemyData) {
    let enemy_yaml = serde_yaml::to_string(&bundle).expect("Unable to serialize!");
    fs::write(ENEMIES_PATH, enemy_yaml).expect("Unable to write file!");
}

pub fn load_enemy(
    enemy: usize,
    asset_server: Res<AssetServer>,
    atlases: ResMut<Atlases>,
) -> EnemyBundle {
    let file_path = get_enemy_path(enemy);
    EnemyBundle::from_path(file_path.to_str().unwrap(), asset_server, atlases)
}


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