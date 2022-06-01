mod components;
mod tetris_block;

use bevy::{math::XY, prelude::*};

use tetris_block::*;

pub const GRID_CELLS: XY<i32> = XY { x: 8, y: 12 };
pub const CELL_SIDE_LEN: f32 = 40.;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: GRID_CELLS.x as f32 * CELL_SIDE_LEN,
            height: GRID_CELLS.y as f32 * CELL_SIDE_LEN,
            title: "Knockoff Tetris".to_string(),
            resizable: false,
            decorations: true,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_startup_system(setup_camera)
        .add_plugins(DefaultPlugins)
        .add_plugin(TetrisBlockPlugin)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
