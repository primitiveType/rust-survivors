use std::backtrace::Backtrace;
use bevy::prelude::{Deref, DerefMut, Resource};
use bevy_ggrs::Session;
use rand::RngCore;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;


#[derive(Resource, Clone, Debug, Deref, DerefMut)]
pub struct SessionRng {
    rng: Xoshiro256PlusPlus,
}

impl SessionRng{
    pub fn get_rng(&mut self) -> &mut Xoshiro256PlusPlus{
        self.rng = Xoshiro256PlusPlus::seed_from_u64(0);
        // let bt = Backtrace::capture();
        // println!("{:?}", bt);
        // println!("Got rng! {} at {bt}", self.rng.next_u32());
        &mut self.rng
    }

    pub fn new (rng : Xoshiro256PlusPlus) -> Self{
        Self{rng}
    }
}