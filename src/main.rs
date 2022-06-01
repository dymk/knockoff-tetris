mod components;
mod tetris_block;

use bevy::prelude::*;

use tetris_block::*;

pub struct Dims {
    width: i32,
    height: i32,
}
pub const GRID_CELLS: Dims = Dims {
    width: 8,
    height: 12,
};
pub const CELL_SIDE_LEN: f32 = 40.;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: GRID_CELLS.width as f32 * CELL_SIDE_LEN,
            height: GRID_CELLS.height as f32 * CELL_SIDE_LEN,
            title: "Knockoff Tetris".to_string(),
            resizable: false,
            decorations: true,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.5)))
        .add_startup_system(setup_camera)
        .add_plugins(DefaultPlugins)
        .add_plugin(TetrisBlockPlugin)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
