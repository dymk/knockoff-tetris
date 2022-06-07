use std::f32::consts::TAU;

use bevy::{
    core::Time,
    math::{IVec2, Quat, Vec3},
    prelude::{default, Component, Plugin, Query, Res, Transform},
};

use crate::{CELL_SIDE_LEN, GRID_CELLS};

use super::movable_block::MovableBlock;

pub struct TweeningPositionPlugin;
impl Plugin for TweeningPositionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(tick_cell_position_times);
    }
}

fn tick_cell_position_times(mut cell_positions: Query<&mut TweeningTransform>, time: Res<Time>) {
    for mut cp in cell_positions.iter_mut() {
        cp.tick(time.delta_seconds());
    }
}

#[derive(Component)]
pub struct TweeningTransform {
    start: Transform,
    target: Transform,
    elapsed: f32,
    duration: f32,
}

impl TweeningTransform {
    pub fn new(transform: &Transform, duration: f32) -> TweeningTransform {
        TweeningTransform {
            start: *transform,
            target: *transform,
            elapsed: 0.,
            duration,
        }
    }

    pub fn transform(&self) -> Transform {
        let t = (self.elapsed / self.duration).clamp(0., 1.);

        // xxx - play with different interpolation curves
        // let t = t * t;

        Transform {
            translation: self.start.translation.lerp(self.target.translation, t),
            rotation: self.start.rotation.lerp(self.target.rotation, t),
            scale: self.start.scale.lerp(self.target.scale, t),
        }
    }

    pub fn set_target(&mut self, new: &Transform) {
        if self.target != *new {
            self.start = self.transform();
            self.target = *new;
            self.elapsed = 0.;
        }
    }

    fn tick(&mut self, dur: f32) {
        self.elapsed += dur;
    }
}
