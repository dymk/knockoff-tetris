use std::f32::consts::TAU;

use bevy::prelude::*;
use lazy_static::lazy_static;

use crate::{CELL_SIDE_LEN, GRID_CELLS};

use super::block_definition::BlockDefinition;

#[derive(Component)]
pub struct AbsolutePositionedPiece {
    pub pos: IVec2,
    pub rot: i32,
    // xxx - construct with around_corner rather than whole block definition
    pub def: &'static BlockDefinition,
}

#[derive(Component)]
pub struct RelativePositionedCell {
    pub pos: IVec2,
    pub def: &'static BlockDefinition,
}

#[derive(Component)]
pub struct AbsolutePositionedCell {
    pub pos: IVec2,
    pub rot: i32,
}

pub struct CellPositioningPlugin;
impl Plugin for CellPositioningPlugin {
    fn build(&self, app: &mut App) {
        // xxx - wrap all this in a system set so it can be made to run after new block positions are calculated
        app.add_system(set_relative_positioned_cell)
            .add_system(set_absolute_positioned_cell)
            .add_system(set_absolute_positioned_piece);
    }
}

lazy_static! {
    static ref SCREEN_DIMS: Vec3 =
        Vec3::new(GRID_CELLS.width as f32, GRID_CELLS.height as f32, 0.) * CELL_SIDE_LEN;
    static ref SHIFT_TO_CORNER: Vec3 = -*SCREEN_DIMS / 2.;
    // static ref HALF_CELL: Vec3 = Vec3::new(CELL_SIDE_LEN / 2., CELL_SIDE_LEN / 2., 0.);
    static ref HALF_CELL: Vec3 = Vec3::new(CELL_SIDE_LEN / 2., CELL_SIDE_LEN / 2., 0.);
}

fn set_absolute_positioned_piece(
    mut query: Query<(&mut Transform, &AbsolutePositionedPiece), Changed<AbsolutePositionedPiece>>,
) {
    for (mut t, p) in query.iter_mut() {
        let maybe_half_cell = if p.def.around_corner {
            *HALF_CELL
        } else {
            Vec3::ZERO
        };

        // if rotating around a corner, shift half a cell so the corner is at (0, 0)
        let mut mat = Transform {
            translation: maybe_half_cell,
            ..default()
        }
        .compute_matrix();

        // rotate the piece
        mat = Transform {
            // xxx - this is wrong, some pieces might not have 4 rotations, need to take into account def.rotations.len
            rotation: Quat::from_rotation_z(-TAU * (p.rot as f32 / 4.)),
            ..default()
        }
        .compute_matrix()
        .mul_mat4(&mat);

        let corner_to_position = Vec3::new(p.pos.x as f32, p.pos.y as f32, 0.) * CELL_SIDE_LEN;
        // finally, shift the whole thing from the center of the screen to the bottom right corner,
        // then apply an offset to shift it to the right cell location, and add half a cell
        // of offset, undoing the half-shift from the corner if needed
        mat = Transform {
            translation: *SHIFT_TO_CORNER + *HALF_CELL + corner_to_position - maybe_half_cell,
            ..default()
        }
        .compute_matrix()
        .mul_mat4(&mat);

        *t = Transform::from_matrix(mat);
    }
}

fn set_relative_positioned_cell(
    mut query: Query<(&mut Transform, &RelativePositionedCell), Changed<RelativePositionedCell>>,
) {
    for (mut t, p) in query.iter_mut() {
        *t = Transform {
            translation: Vec3::new(p.pos.x as f32, p.pos.y as f32, 0.) * CELL_SIDE_LEN,
            ..default()
        };
    }
}
fn set_absolute_positioned_cell(
    mut query: Query<(&mut Transform, &AbsolutePositionedCell), Changed<AbsolutePositionedCell>>,
) {
    for (mut t, p) in query.iter_mut() {
        println!("setting abs position to {}", p.pos);
        let corner_to_position = Vec3::new(p.pos.x as f32, p.pos.y as f32, 0.) * CELL_SIDE_LEN;
        let translation = *SHIFT_TO_CORNER + corner_to_position + *HALF_CELL;
        let rotation = Quat::from_rotation_z(-TAU * (p.rot as f32 / 4.));
        *t = Transform {
            translation,
            rotation,
            ..default()
        };
    }
}
