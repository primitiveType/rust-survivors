use bevy::prelude::{Commands, Entity, Query, Without};


use crate::{Health, Player};

pub fn die_at_zero_health(mut query: Query<(Entity, &Health), Without<Player>>,
                          mut _commands: Commands,
) {
    for (entity, health) in query.iter() {
        if health.value <= 0.0
        {
            _commands.entity(entity).despawn();
            println!("entity died.");
        }
    }
}