use std::time::Duration;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

pub const DEFAULT_TIMESTEP: Duration = Duration::from_micros(15625);
pub const MAX_PHYSICS_EXEC_TIME: Duration = Duration::from_micros(15625);

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(PhysicsSchedule)
            .register_type::<PhysicsTime>()
            .init_resource::<PhysicsTime>()
            .add_systems(PreUpdate, run_physics_schedule);
    }
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PhysicsSchedule;

pub type PhysicsTime = Time<PhysicsTimeInner>;

pub trait PhysicsTimeExt {
    fn pause(&mut self);
    fn resume(&mut self);
    fn _step(&mut self);
    fn _run(&mut self, speed: f32);
}

impl PhysicsTimeExt for PhysicsTime {
    fn pause(&mut self) {
        self.context_mut().set_mode(PhysicsTimeMode::Paused);
    }

    fn resume(&mut self) {
        let old_mode = self.context().old_mode;
        self.context_mut().set_mode(old_mode);
    }

    fn _step(&mut self) {
        self.context_mut().set_mode(PhysicsTimeMode::OneTick);
    }

    fn _run(&mut self, speed: f32) {
        self.context_mut()
            .set_mode(PhysicsTimeMode::Running { speed });
    }
}

#[derive(Debug, Copy, Clone, Reflect)]
#[reflect(Default)]
pub struct PhysicsTimeInner {
    pub mode: PhysicsTimeMode,
    old_mode: PhysicsTimeMode,
    pub timestep: Duration,
    pub overstep: Duration,
}

impl PhysicsTimeInner {
    pub fn set_mode(&mut self, mode: PhysicsTimeMode) {
        if let PhysicsTimeMode::Running { .. } = mode {
            self.old_mode = mode;
        }
        self.mode = mode;
    }
}

impl Default for PhysicsTimeInner {
    fn default() -> Self {
        Self {
            mode: PhysicsTimeMode::default(),
            old_mode: PhysicsTimeMode::default(),
            timestep: DEFAULT_TIMESTEP,
            overstep: Duration::ZERO,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum PhysicsTimeMode {
    Paused,
    OneTick,
    Running { speed: f32 },
}

impl Default for PhysicsTimeMode {
    fn default() -> Self {
        Self::Running { speed: 1. }
    }
}

fn accumulate_time(time: &mut PhysicsTime, delta: Duration) {
    let context = time.context_mut();
    match context.mode {
        PhysicsTimeMode::Paused => (),
        PhysicsTimeMode::OneTick => (),
        PhysicsTimeMode::Running { speed } => {
            if speed == std::f32::INFINITY {
                context.overstep = Duration::MAX;
            } else {
                context.overstep = context.overstep.saturating_add(delta.mul_f32(speed));
            }
        }
    }
}

fn expend_time(time: &mut PhysicsTime) -> bool {
    let context = time.context_mut();
    let result = match context.mode {
        PhysicsTimeMode::Paused => false,
        PhysicsTimeMode::OneTick => {
            context.mode = PhysicsTimeMode::Paused;
            context.overstep = Duration::ZERO;
            true
        }
        PhysicsTimeMode::Running { speed: _ } => {
            if let Some(new_value) = context.overstep.checked_sub(context.timestep) {
                context.overstep = new_value;
                true
            } else {
                false
            }
        }
    };

    if result {
        let timestep = context.timestep;
        time.advance_by(timestep);
    }
    result
}

fn limit_overstep(time: &mut PhysicsTime) {
    let context = time.context_mut();
    context.overstep = context.overstep.min(context.timestep * 3);
}

pub fn run_physics_schedule(world: &mut World) {
    let delta = world.resource::<Time<Virtual>>().delta();
    accumulate_time(&mut world.resource_mut::<PhysicsTime>(), delta);

    let time = std::time::Instant::now();
    world.schedule_scope(PhysicsSchedule, |world, schedule| {
        while expend_time(&mut world.resource_mut::<PhysicsTime>()) {
            schedule.run(world);
            if time.elapsed() >= MAX_PHYSICS_EXEC_TIME {
                break;
            }
        }
        limit_overstep(&mut world.resource_mut::<PhysicsTime>());
    });
}
