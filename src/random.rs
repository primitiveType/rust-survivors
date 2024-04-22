use bevy::prelude::{Deref, DerefMut, Resource};
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;


#[derive(Resource, Clone, Debug, Deref, DerefMut)]
pub struct SessionRng {
    pub rng: Xoshiro256PlusPlus,
}