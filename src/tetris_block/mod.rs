mod block_definition;
mod board_state;
mod movable_block;
mod skate_timer;
mod tuple_util;

use self::board_state::BoardState;
use self::movable_block::{BlockName, MovableBlock, RotDir};
use self::skate_timer::SkateTimer;
use crate::{CELL_SIDE_LEN, GRID_CELLS};
use bevy::{core::FixedTimestep, ecs::schedule::ShouldRun, prelude::*};
use rand::{thread_rng, Rng};

#[derive(Component)]
struct TetrisBlock {
    movable: MovableBlock,
}

// Marks the TetrisBlock entity which is the Ghost
// if no Ghost attribute, it's the active falling TetrisPiece
#[derive(Component)]
struct Ghost;

#[derive(Deref)]
struct FrameNum(u64);

#[derive(Component)]
struct CellPosition(IVec2);
impl CellPosition {
    pub fn to_translation(&self) -> Vec3 {
        let screen_dims =
            Vec2::new(GRID_CELLS.width as f32, GRID_CELLS.height as f32) * CELL_SIDE_LEN;
        // offset to apply to move center (0, 0) to the bottom left of the screen
        let offset = -screen_dims / 2.;
        let this = Vec2::new(self.0.x as f32, self.0.y as f32) * CELL_SIDE_LEN;
        let shifted = this + offset + Vec2::new(CELL_SIDE_LEN / 2., CELL_SIDE_LEN / 2.);
        Vec3::new(shifted.x, shifted.y, 0.)
    }
    pub fn set(&mut self, new: IVec2) {
        self.0 = new;
    }
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
        app.insert_resource(FrameNum(0));
        app.insert_resource(PlaceBlock(false));
        app.add_system(update_pause_state);

        {
            let mut stage = SystemStage::parallel();
            stage.add_system(inc_frame_num);
            app.add_stage_after(CoreStage::First, "inc_frame_counter", stage);
        }

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
                .add_system(handle_block_user_movement)
                .add_system(position_ghost_block.after(handle_block_user_movement))
                .add_system(update_cell_positions.after(position_ghost_block))
                // moves the active block down every 1 second
                .add_system_set(
                    SystemSet::new()
                        .with_run_criteria(FixedTimestep::step(1.5))
                        .with_system(move_active_block_down.after(handle_block_user_movement)),
                )
                // checks if the skate timer can be started after block movement
                .add_system(
                    check_skate_timer
                        .after(move_active_block_down)
                        .after(update_cell_positions),
                )
                .add_system(place_block.after(check_skate_timer));

            app.add_stage_after(
                "spawn_new_blocks",
                "update_block_positions",
                update_block_positions,
            );
        }

        // step 3 - update the Transform of all the sprites that are on the screen
        {
            let mut update_block_transforms = SystemStage::parallel();
            update_block_transforms.add_system(update_cell_positions_from_board_state);
            update_block_transforms.add_system(
                update_transforms_from_cell_positions.after(update_cell_positions_from_board_state),
            );
            app.add_stage_after(
                "update_block_positions",
                "update_block_transforms",
                update_block_transforms,
            );
        }
    }
}

fn inc_frame_num(mut frame_num: ResMut<FrameNum>) {
    frame_num.0 += 1;
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

fn spawn_new_block(mut commands: Commands, frame_num: Res<FrameNum>) {
    let color = rand_color();
    let block = rand_block();

    println!("{} - spawning new block: {:?}", frame_num.0, block);

    let spawn_at = IVec2::new(
        (GRID_CELLS.width / 2) as i32,
        (GRID_CELLS.height - 3) as i32,
    );
    let movable = block.create_movable(spawn_at);

    // the active tetris block
    commands
        .spawn()
        .insert_bundle(TransformBundle::identity())
        .with_children(|builder| add_children(builder, color, false, &movable))
        .insert(TetrisBlock {
            movable: movable.clone(),
        });

    // the ghost tetris block
    commands
        .spawn()
        .insert_bundle(TransformBundle::identity())
        .with_children(|builder| add_children(builder, color, true, &movable))
        .insert(TetrisBlock { movable })
        .insert(Ghost);
}

fn add_children(builder: &mut ChildBuilder, color: Color, is_ghost: bool, movable: &MovableBlock) {
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
    let ghost_sprite = ||
        // desaturated color
        Sprite {
            color: color.as_hsla() * Vec4::new(1., 0.2, 1.0, 1.0),
            custom_size: Some(Vec2::new(CELL_SIDE_LEN, CELL_SIDE_LEN)),
            ..default()
        };

    for pos in movable.positions() {
        builder
            .spawn()
            .insert_bundle(TransformBundle::identity())
            .insert(CellPosition(pos))
            .with_children(|p2| {
                if !is_ghost {
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
                } else {
                    p2.spawn()
                        .insert_bundle(TransformBundle::identity())
                        // this position will be updated later to move the block to the lowest point possible on the screen
                        .insert_bundle(SpriteBundle {
                            sprite: ghost_sprite(),
                            transform: at_z_pixel(9.),
                            ..default()
                        });
                }
            });
    }
}

fn handle_block_user_movement(
    kb: Res<Input<KeyCode>>,
    board_state: Res<BoardState>,
    mut place_block: ResMut<PlaceBlock>,
    mut active_block_query: Query<&mut TetrisBlock, Without<Ghost>>,
) {
    let mut block = match active_block_query.get_single_mut() {
        Ok(block) => block,
        _ => return,
    };

    let mut nudge_movable = |dir| {
        let movable = block.movable.nudge(dir);
        if board_state.can_place(&movable) {
            block.movable = movable;
            true
        } else {
            false
        }
    };

    if kb.just_pressed(KeyCode::Left) {
        nudge_movable((-1, 0).into());
    }
    if kb.just_pressed(KeyCode::Right) {
        nudge_movable((1, 0).into());
    }
    // soft drop
    if kb.just_pressed(KeyCode::Down) {
        while nudge_movable((0, -1).into()) {}
    }
    // hard drop
    if kb.just_pressed(KeyCode::Up) {
        while nudge_movable((0, -1).into()) {}
        place_block.0 = true;
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

fn position_ghost_block(
    mut ghost_query: Query<&mut TetrisBlock, With<Ghost>>,
    block_query: Query<&TetrisBlock, Without<Ghost>>,
    board_state: Res<BoardState>,
) {
    let active = block_query.single();
    let mut ghost = ghost_query.single_mut();

    // from the original active position, move ghost down until it can't be moved
    // further
    ghost.movable = active.movable.clone();
    while board_state.can_place(&ghost.movable.nudge((0, -1).into())) {
        ghost.movable = ghost.movable.nudge((0, -1).into());
    }
}

fn update_cell_positions(
    query: Query<(&TetrisBlock, &Children)>,
    mut cell_query: Query<&mut CellPosition>,
) {
    for (block, children) in query.iter() {
        for (cell_pos, &child_ent) in block.movable.positions().zip(&children[..]) {
            if let Ok(mut cell) = cell_query.get_mut(child_ent) {
                cell.set(cell_pos)
            }
        }
    }
}

fn move_active_block_down(
    paused: Res<Paused>,
    board_state: Res<BoardState>,
    mut query: Query<&mut TetrisBlock>,
) {
    if paused.0 {
        return;
    }

    for mut block in query.iter_mut() {
        let movable = block.movable.nudge((0, -1).into());
        if board_state.can_place(&movable) {
            block.movable = movable;
        }
    }
}

// should the block be placed?
struct PlaceBlock(bool);

// stops the skate timer if the block can move downwards
fn check_skate_timer(
    mut commands: Commands,
    mut place_block: ResMut<PlaceBlock>,
    frame_num: Res<FrameNum>,
    timer: Option<ResMut<SkateTimer>>,
    board_state: Res<BoardState>,
    time: Res<Time>,
    query: Query<&TetrisBlock, Without<Ghost>>,
) {
    let active_movable = match query.get_single() {
        Ok(block) => &block.movable,
        Err(_) => return,
    };

    if board_state.can_place(&active_movable.nudge((0, -1).into())) {
        // if the block can move down, stop the skate timer
        // if so, stop the skate timer
        if timer.is_some() {
            println!("{} - block can drop, stopping skate timer", frame_num.0);
            commands.remove_resource::<SkateTimer>();
        }
        return;
    }

    // the block can't move down, start the timer or place the block
    match timer {
        Some(mut timer) => {
            if timer.0.tick(time.delta()).just_finished() {
                println!("{} - timer fired, signaling placing block", frame_num.0);
                commands.remove_resource::<SkateTimer>();
                place_block.0 = true;
            }
        }
        None => {
            println!("{} - starting skate timer", frame_num.0);
            commands.insert_resource(SkateTimer(Timer::from_seconds(2., false)));
        }
    }
}

fn place_block(
    mut place_block: ResMut<PlaceBlock>,
    frame_num: Res<FrameNum>,
    mut commands: Commands,
    active_query: Query<(Entity, &TetrisBlock, &Children), Without<Ghost>>,
    ghost_query: Query<Entity, (With<TetrisBlock>, With<Ghost>)>,
    mut board_state: ResMut<BoardState>,
) {
    if place_block.0 {
        place_block.0 = false;
        println!("{} - placing block", frame_num.0);
    } else {
        return;
    }

    let (active_entity, active_block, active_children) = match active_query.get_single() {
        Ok(tup) => tup,
        Err(_) => return,
    };

    let ghost_entity = match ghost_query.get_single() {
        Ok(tup) => tup,
        Err(_) => return,
    };

    // if there's still room to move the block downwards...
    if board_state.can_place(&active_block.movable.nudge((0, -1).into())) {
        // then bail out on finalizing block placement
        println!("{} - room below block, bailing", frame_num.0);
        return;
    }

    // no more room to move the block down, finalize plcaement
    board_state.place_block(&active_block.movable, &active_children[..]);

    // orphan children of the active, the board state effectively takes
    // ownership of their placement once the parent TetrisBlock is despawned
    commands
        .entity(active_entity)
        .remove_children(active_children);

    // and remove the tetris block entity
    commands.entity(active_entity).despawn();

    // remove the ghost entity and all its children recursively (they don't
    // persist after block placement)
    commands.entity(ghost_entity).despawn_recursive();

    // check for any lines that were filled, and clear them
    for ent in board_state.clear_filled_lines() {
        commands.entity(ent).despawn_recursive();
    }
}

fn update_cell_positions_from_board_state(
    mut query: Query<&mut CellPosition>,
    board_state: Res<BoardState>,
) {
    for (pos, ent) in board_state.iter_ents() {
        if let Ok(mut cell_position) = query.get_mut(ent) {
            cell_position.set(pos);
        }
    }
}

// move the sprites around according to the new board state, and cell positions
// for the blocks that care about that information
fn update_transforms_from_cell_positions(mut query: Query<(&mut Transform, &CellPosition)>) {
    for (mut transform, cell_position) in query.iter_mut() {
        transform.translation = cell_position.to_translation();
    }
}
