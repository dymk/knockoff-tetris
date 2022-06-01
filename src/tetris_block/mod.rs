mod block_shape;
mod board_state;
mod skate_timer;

use bevy::{core::FixedTimestep, ecs::schedule::ShouldRun, prelude::*};
use rand::{thread_rng, Rng};

use crate::{components::ActiveBlock, CELL_SIDE_LEN, GRID_CELLS};

use self::block_shape::{BlockShape, BlockShapeDescriptor};
use self::board_state::{clear_filled_lines, rebuild_board_state, BoardState};
use self::skate_timer::{skate_timer_absent, skate_timer_present, SkateTimer};

#[derive(Component)]
struct TetrisBlock {
    descriptor: BlockShapeDescriptor,
}

#[derive(Component, Debug, Copy, Clone)]
pub struct GridLocation {
    idx: usize,
    loc: IVec2,
}

impl GridLocation {
    pub fn to_translation(&self) -> Vec3 {
        let screen_dims = Vec2::new(GRID_CELLS.x as f32, GRID_CELLS.y as f32) * CELL_SIDE_LEN;
        // offset to apply to move center (0, 0) to the bottom left of the screen
        let offset = -screen_dims / 2.;
        let this = Vec2::new(self.loc.x as f32, self.loc.y as f32) * CELL_SIDE_LEN;
        let shifted = this + offset + Vec2::new(CELL_SIDE_LEN / 2., CELL_SIDE_LEN / 2.);
        Vec3::new(shifted.x, shifted.y, 0.)
    }
}

pub struct TetrisBlockPlugin;
impl Plugin for TetrisBlockPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(BoardState::new());

        // spawning the new blocks must happen in its own stage, so update_block_positions can work with newly spawned entities
        let mut spawn_new_blocks = SystemStage::parallel();
        spawn_new_blocks.add_system_set(
            SystemSet::new()
                .with_run_criteria(no_active_block_exists)
                .with_system(insert_active_block),
        );

        let mut update_block_positions = SystemStage::parallel();
        update_block_positions
            .add_system(handle_block_left_right)
            .add_system(handle_block_rotation)
            .add_system(
                update_child_grid_locations_from_block
                    .after(handle_block_left_right)
                    .after(handle_block_rotation),
            )
            .add_system(
                update_child_transforms_from_grid_locations
                    .after(update_child_grid_locations_from_block),
            )
            // moves the active block down every 1 second
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(1.))
                    .with_system(
                        move_active_block_down.after(update_child_transforms_from_grid_locations),
                    ),
            )
            // checks if the skate timer can be started after block movement
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(skate_timer_absent)
                    .with_system(start_stake_timer.after(move_active_block_down)),
            )
            // system waits for skate timer to fire to finalize block position
            // and remove the ActiveBlock marker component
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(skate_timer_present)
                    .with_system(deactivate_block_after_skate_timer),
            );

        let mut update_board_state = SystemStage::parallel();
        update_board_state
            .add_system(rebuild_board_state)
            .add_system(clear_filled_lines.after(rebuild_board_state));

        app.add_stage_after(CoreStage::Update, "spawn_new_blocks", spawn_new_blocks);
        app.add_stage_after(
            "spawn_new_blocks",
            "update_block_positions",
            update_block_positions,
        );
        app.add_stage_after(
            "update_block_positions",
            "update_board_state",
            update_board_state,
        );
    }
}

fn no_active_block_exists(query: Query<(), With<ActiveBlock>>) -> ShouldRun {
    for _ in query.iter() {
        return ShouldRun::No;
    }
    ShouldRun::Yes
}

const COLORS: &[Color] = &[
    Color::RED,
    Color::GREEN,
    Color::BLUE,
    Color::ORANGE,
    Color::PURPLE,
];

fn at_z_pixel(z: f32) -> Transform {
    Transform {
        translation: Vec3::new(0., 0., z),
        ..default()
    }
}

fn insert_active_block(mut commands: Commands) {
    let color = COLORS[thread_rng().gen_range(0..COLORS.len())];

    let big_sprite = || Sprite {
        color,
        custom_size: Some(Vec2::new(CELL_SIDE_LEN, CELL_SIDE_LEN)),
        ..default()
    };
    let little_sprite = || Sprite {
        color: color * 0.5,
        custom_size: Some(Vec2::new(CELL_SIDE_LEN * 0.9, CELL_SIDE_LEN * 0.9)),
        ..default()
    };

    let shape = BlockShape::LShape;
    let spawn_at = IVec2::new(GRID_CELLS.x / 2, GRID_CELLS.y - 3);
    let descriptor = shape.create_descriptor(spawn_at);

    commands
        .spawn()
        .insert(ActiveBlock)
        .insert(Transform::identity())
        .insert(GlobalTransform::identity())
        .with_children(|parent| {
            for (idx, loc) in descriptor.locs().enumerate() {
                parent
                    .spawn()
                    .insert(Transform::identity())
                    .insert(GlobalTransform::identity())
                    .insert(ActiveBlock)
                    .insert(GridLocation { idx, loc })
                    .with_children(|parent| {
                        parent.spawn().insert_bundle(SpriteBundle {
                            sprite: big_sprite(),
                            transform: at_z_pixel(10.),
                            ..default()
                        });

                        parent.spawn().insert_bundle(SpriteBundle {
                            sprite: little_sprite(),
                            transform: at_z_pixel(11.),
                            ..default()
                        });
                    });
            }
        })
        .insert(TetrisBlock { descriptor });
}

fn handle_block_left_right(
    kb: Res<Input<KeyCode>>,
    board_state: Res<BoardState>,
    mut active_block_query: Query<&mut TetrisBlock, With<ActiveBlock>>,
) {
    if let Ok(mut block) = active_block_query.get_single_mut() {
        let mut try_nudge_descriptor = |dir| {
            if block
                .descriptor
                .locs()
                .all(|loc| board_state.is_occupied(loc + dir))
            {
                block.descriptor.nudge(dir);
                true
            } else {
                false
            }
        };

        if kb.just_pressed(KeyCode::Left) {
            try_nudge_descriptor((-1, 0).into());
        }

        if kb.just_pressed(KeyCode::Right) {
            try_nudge_descriptor((1, 0).into());
        }

        if kb.just_pressed(KeyCode::Down) {
            while try_nudge_descriptor((0, -1).into()) {}
        }
    }
}

fn handle_block_rotation(
    kb: Res<Input<KeyCode>>,
    board_state: Res<BoardState>,
    mut active_block_query: Query<&mut TetrisBlock, With<ActiveBlock>>,
) {
    if kb.just_pressed(KeyCode::Space) {
        if let Ok(mut block) = active_block_query.get_single_mut() {
            let original_descriptor = block.descriptor.clone();

            block.descriptor.rotate();

            // wall kick if gone off the edge
            let min_x = block.descriptor.locs().map(|loc| loc.x).min();
            match min_x {
                Some(x) if x < 0 => block.descriptor.nudge((-x, 0).into()),
                _ => {}
            }

            let max_x = block.descriptor.locs().map(|loc| loc.x).max();
            match max_x {
                Some(x) if x >= GRID_CELLS.x => {
                    block.descriptor.nudge((-(GRID_CELLS.x - x + 1), 0).into())
                }
                _ => {}
            }

            // revert if any cells are already occupied
            if block
                .descriptor
                .locs()
                .any(|loc| !board_state.is_occupied(loc))
            {
                block.descriptor = original_descriptor;
            }
        }
    }
}

fn update_child_grid_locations_from_block(
    query: Query<(&TetrisBlock, &Children), (With<ActiveBlock>, Changed<TetrisBlock>)>,
    mut grid_location: Query<&mut GridLocation>,
) {
    for (block, child_ents) in query.iter() {
        let locs: Vec<_> = block.descriptor.locs().collect();

        for &child_ent in child_ents.iter() {
            if let Ok(mut gl) = grid_location.get_mut(child_ent) {
                gl.loc = locs[gl.idx];
            }
        }
    }
}

fn update_child_transforms_from_grid_locations(
    mut query: Query<(&GridLocation, &mut Transform), Changed<GridLocation>>,
) {
    for (gl, mut tx) in query.iter_mut() {
        tx.translation = gl.to_translation();
    }
}

fn move_active_block_down(
    mut commands: Commands,
    board_state: Res<BoardState>,
    mut query: Query<&mut TetrisBlock, With<ActiveBlock>>,
) {
    for mut block in query.iter_mut() {
        // move down if possible
        if block
            .descriptor
            .locs()
            .all(|loc| board_state.is_occupied(loc + IVec2::new(0, -1)))
        {
            block.descriptor.nudge((0, -1).into());

            // cancel the existing skate timer if the block was able to
            // be moved down
            commands.remove_resource::<SkateTimer>();
        }
    }
}

fn start_stake_timer(
    mut commands: Commands,
    board_state: Res<BoardState>,
    mut query: Query<&TetrisBlock, With<ActiveBlock>>,
) {
    for block in query.iter_mut() {
        // can the block move down no further?
        if block
            .descriptor
            .locs()
            .any(|loc| !board_state.is_occupied(loc + IVec2::new(0, -1)))
        {
            // if so, start the skate timer
            commands.insert_resource(SkateTimer(Timer::from_seconds(1.0, false)));
        }
    }
}

fn deactivate_block_after_skate_timer(
    mut commands: Commands,
    mut timer: ResMut<SkateTimer>,
    time: Res<Time>,
    tetris_block_query: Query<(Entity, &TetrisBlock), With<ActiveBlock>>,
    board_state: Res<BoardState>,
    active_blocks_query: Query<Entity, With<ActiveBlock>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    commands.remove_resource::<SkateTimer>();

    if let Ok((tetris_block_ent, tetris_block)) = tetris_block_query.get_single() {
        // if any blocks have the location below them occupied...
        if tetris_block
            .descriptor
            .locs()
            .any(|loc| !board_state.is_occupied(loc + IVec2::new(0, -1)))
        {
            // finalize the location of the active blocks
            // and remove the tetris block entity
            commands.entity(tetris_block_ent).despawn();
            for entity in active_blocks_query.iter() {
                if entity == tetris_block_ent {
                    continue;
                }
                commands.entity(entity).remove::<ActiveBlock>();
            }
        }
    }
}
