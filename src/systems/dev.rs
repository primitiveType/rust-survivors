use crate::AppState;
use bevy::log::info;
use bevy::prelude::{EventReader, StateTransitionEvent};

/// print when an `AppState` transition happens
/// also serves as an example of how to use `StateTransitionEvent`
pub fn log_transitions(mut transitions: EventReader<StateTransitionEvent<AppState>>) {
    for transition in transitions.read() {
        info!(
            "transition: {:?} => {:?}",
            transition.before, transition.after
        );
    }
}
