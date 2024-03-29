use std::env::Args;
use bevy::prelude::EventWriter;
use spew::prelude::SpawnEvent;
use crate::bundles::Object;

pub type SpawnerOf<'w, EnumType, ArgsType> = EventWriter<'w, SpawnEvent<EnumType, ArgsType>>;
pub type Spawner<'w, ArgsType> = EventWriter<'w, SpawnEvent<Object, ArgsType>>;

impl<'w, Key: Eq + Send + Sync, Value: Send + Sync> Spawn<Key, Value> for EventWriter<'w, SpawnEvent<Key, Value>> {
    fn spawn(&mut self, object : Key, args : Value) {
        self.send(SpawnEvent::with_data(object, args));
    }
}

pub trait Spawn<Object, Args> {
    fn spawn(&mut self, object : Object, args : Args);
}