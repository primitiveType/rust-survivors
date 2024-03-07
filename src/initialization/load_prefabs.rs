use std::fs;
use std::fs::DirEntry;

use bevy::prelude::{Bundle, SpatialBundle};
use bevy::reflect::erased_serde::__private::serde::Serializer;
use bevy_inspector_egui::__macro_exports::bevy_reflect::erased_serde::__private::serde;
use serde::Deserialize;
use serde::Serialize;

use crate::components::Gun;

#[derive(Bundle, Clone, Debug, Default, Serialize, Deserialize)]
pub struct GunBundle {
    pub gun: Gun,

    #[serde(skip)]
    pub spatial_bundle: SpatialBundle,

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

pub fn load_gun_test() {
    load_gun(999);
}