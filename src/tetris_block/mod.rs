mod block_definition;
mod board_state;
mod movable_block;
mod skate_timer;
mod tuple_util;

use bevy::{core::FixedTimestep, ecs::schedule::ShouldRun, prelude::*};
use rand::{thread_rng, Rng};

use crate::{CELL_SIDE_LEN, GRID_CELLS};

use self::board_state::BoardState;
use self::movable_block::{BlockName, MovableBlock, RotDir};
use self::skate_timer::{skate_timer_absent, skate_timer_present, SkateTimer};

#[derive(Component)]
struct TetrisBlock {
    movable: MovableBlock,
}

fn loc_to_translation(loc: IVec2) -> Vec3 {
    let screen_dims = Vec2::new(GRID_CELLS.width as f32, GRID_CELLS.height as f32) * CELL_SIDE_LEN;
    // offset to apply to move center (0, 0) to the bottom left of the screen
    let offset = -screen_dims / 2.;
    let this = Vec2::new(loc.x as f32, loc.y as f32) * CELL_SIDE_LEN;
    let shifted = this + offset + Vec2::new(CELL_SIDE_LEN / 2., CELL_SIDE_LEN / 2.);
    Vec3::new(shifted.x, shifted.y, 0.)
}

struct Paused(bool);

pub struct TetrisBlockPlugin;
impl Plugin for TetrisBlockPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(BoardState::new(
            GRID_CELLS.width as usize,
            GRID_CELLS.height as usize,
        ));
        app.insert_resource(Paused(true));
        app.add_system(update_pause_state);

        // step 1 - add new blocks to the game state
        {
            let mut spawn_new_blocks = SystemStage::parallel();
            spawn_new_blocks.add_system_set(
                SystemSet::new()
                    .with_run_criteria(no_active_block_exists)
                    .with_system(spawn_new_block),
            );
            app.add_stage_after(CoreStage::Update, "spawn_new_blocks", spawn_new_blocks);
        }

        // step 2 - calculate the new position of blocks, finalize block placement,
        // clear any filled lines
        {
            let mut update_block_positions = SystemStage::parallel();
            update_block_positions
                .add_system(handle_block_left_right)
                // moves the active block down every 1 second
                .add_system_set(
                    SystemSet::new()
                        .with_run_criteria(FixedTimestep::step(1.))
                        .with_system(move_active_block_down),
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
                        .with_system(place_block_after_skate_timer),
                );
            app.add_stage_after(
                "spawn_new_blocks",
                "update_block_positions",
                update_block_positions,
            );
        }

        // step 3 - update the Transform of all the sprites that are on the screen
        {
            let mut update_block_transforms = SystemStage::parallel();
            update_block_transforms.add_system(update_child_transforms_from_board_state);
            app.add_stage_after(
                "update_block_positions",
                "update_block_transforms",
                update_block_transforms,
            );
        }
    }
}

fn update_pause_state(input: Res<Input<KeyCode>>, mut paused: ResMut<Paused>) {
    if input.just_pressed(KeyCode::Space) {
        paused.0 = !paused.0;
    }
}

fn no_active_block_exists(query: Query<(), With<TetrisBlock>>) -> ShouldRun {
    if query.iter().next().is_some() {
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
fn rand_color() -> Color {
    COLORS[thread_rng().gen_range(0..COLORS.len())]
}

fn at_z_pixel(z: f32) -> Transform {
    Transform {
        translation: Vec3::new(0., 0., z),
        ..default()
    }
}

const BLOCKS: &[BlockName] = &[
    BlockName::L,
    BlockName::J,
    BlockName::O,
    BlockName::I,
    BlockName::T,
    BlockName::S,
    BlockName::Z,
];
fn rand_block() -> BlockName {
    BLOCKS[thread_rng().gen_range(0..BLOCKS.len())]
}

fn spawn_new_block(mut commands: Commands) {
    let color = rand_color();
    let block = rand_block();

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

    let spawn_at = IVec2::new(
        (GRID_CELLS.width / 2) as i32,
        (GRID_CELLS.height - 3) as i32,
    );
    let movable_block = block.create_movable(spawn_at);

    commands
        .spawn()
        .insert_bundle(TransformBundle::identity())
        .with_children(|p1| {
            for _ in movable_block.positions() {
                p1.spawn()
                    .insert_bundle(TransformBundle::identity())
                    .with_children(|p2| {
                        p2.spawn().insert_bundle(SpriteBundle {
                            sprite: big_sprite(),
                            transform: at_z_pixel(10.),
                            ..default()
                        });
                        p2.spawn().insert_bundle(SpriteBundle {
                            sprite: little_sprite(),
                            transform: at_z_pixel(11.),
                            ..default()
                        });
                    });
            }
        })
        .insert(TetrisBlock {
            movable: movable_block,
        });
}

fn handle_block_left_right(
    kb: Res<Input<KeyCode>>,
    board_state: Res<BoardState>,
    mut active_block_query: Query<&mut TetrisBlock>,
) {
    let mut block = match active_block_query.get_single_mut() {
        Ok(block) => block,
        _ => return,
    };

    let mut try_nudge_descriptor = |dir| {
        let movable = block.movable.nudge(dir);
        if board_state.can_place(&movable) {
            block.movable = movable;
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

    let mut rotate = |dir| {
        let (movable, kicks) = block.movable.rotate(dir);

        for &kick in kicks {
            let movable = movable.nudge(kick);
            if board_state.can_place(&movable) {
                block.movable = movable;
                return;
            }
        }
    };

    if kb.just_pressed(KeyCode::A) {
        rotate(RotDir::Left);
    }
    if kb.just_pressed(KeyCode::D) {
        rotate(RotDir::Right);
    }

    if kb.just_pressed(KeyCode::C) {
        println!("{:?}", board_state.as_ref());
    }
}

// move the sprites around according to the new board state
fn update_child_transforms_from_board_state(
    mut query: Query<&mut Transform>,
    block_query: Query<(&TetrisBlock, &Children)>,
    board_state: Res<BoardState>,
) {
    let mut update_entity_xform = |loc, ent| {
        if let Ok(mut tx) = query.get_mut(ent) {
            let translation = loc_to_translation(loc);
            if translation != tx.translation {
                tx.translation = translation;
                assert!(tx.translation == translation);
            }
        }
    };

    for (block, children) in block_query.iter() {
        for (pos, &ent) in block.movable.positions().zip(&children[..]) {
            update_entity_xform(pos, ent);
        }
    }

    for (pos, ent) in board_state.iter_ents() {
        update_entity_xform(pos, ent);
    }
}

fn move_active_block_down(
    paused: Res<Paused>,
    mut commands: Commands,
    board_state: Res<BoardState>,
    mut query: Query<&mut TetrisBlock>,
) {
    if paused.0 {
        return;
    }

    for mut block in query.iter_mut() {
        // move down if possible
        let movable = block.movable.nudge((0, -1).into());
        if board_state.can_place(&movable) {
            block.movable = movable;
            commands.remove_resource::<SkateTimer>();
        }
    }
}

fn start_stake_timer(
    mut commands: Commands,
    board_state: Res<BoardState>,
    mut query: Query<&TetrisBlock>,
) {
    for block in query.iter_mut() {
        // can the block move down no further?
        if !board_state.can_place(&block.movable.nudge((0, -1).into())) {
            // if so, start the skate timer
            commands.insert_resource(SkateTimer(Timer::from_seconds(1.0, false)));
        }
    }
}

fn place_block_after_skate_timer(
    mut commands: Commands,
    mut timer: ResMut<SkateTimer>,
    time: Res<Time>,
    query: Query<(Entity, &TetrisBlock, &Children)>,
    mut board_state: ResMut<BoardState>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    commands.remove_resource::<SkateTimer>();

    if let Ok((tetris_block_ent, tetris_block, children)) = query.get_single() {
        // if any blocks have the location below them occupied...
        if !board_state.can_place(&tetris_block.movable.nudge((0, -1).into())) {
            // finalize the location of the block cells
            board_state.place_block(&tetris_block.movable, &children[..]);
            // orphan children, else their children despawn
            commands.entity(tetris_block_ent).remove_children(children);
            // and remove the tetris block entity
            commands.entity(tetris_block_ent).despawn();

            // check for any lines that were filled, and clear them
            for ent in board_state.clear_filled_lines() {
                commands.entity(ent).despawn_recursive();
            }
        }
    }
}
