use bevy::{ecs::schedule::ShouldRun, prelude::*};

pub struct SkateTimer(pub Timer);

pub fn skate_timer_present(t: Option<Res<SkateTimer>>) -> ShouldRun {
    if t.is_none() {
        ShouldRun::No
    } else {
        ShouldRun::Yes
    }
}

pub fn skate_timer_absent(t: Option<Res<SkateTimer>>) -> ShouldRun {
    if t.is_none() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}
