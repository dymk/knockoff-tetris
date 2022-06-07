mod block_definition;
mod board;
mod cell_positioning;
mod movable_block;
mod skate_timer;
mod tuple_util;
// mod tweening_position;

use self::board::Board;
use self::cell_positioning::{AbsolutePositionedCell, CellPositioningPlugin};
use self::movable_block::{BlockName, MovableBlock, RotDir};
use self::skate_timer::SkateTimer;
use crate::tetris_block::cell_positioning::{AbsolutePositionedPiece, RelativePositionedCell};
use crate::{CELL_SIDE_LEN, GRID_CELLS};
use bevy::{core::FixedTimestep, ecs::schedule::ShouldRun, prelude::*};
use rand::{thread_rng, Rng};

#[derive(Component)]
struct TetrisBlock {
    movable: MovableBlock,
}

// Marks the active TetrisBlock (which is being moved by the player)
#[derive(Component)]
struct Active;

// Marks the TetrisBlock entity which is the Ghost
// if no Ghost attribute, it's the active falling TetrisPiece
#[derive(Component)]
struct Ghost;

#[derive(Deref)]
struct FrameNum(u64);

struct Paused(bool);

pub struct TetrisBlockPlugin;
impl Plugin for TetrisBlockPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Board::new(
            GRID_CELLS.width as usize,
            GRID_CELLS.height as usize,
        ));
        app.insert_resource(Paused(true));
        app.insert_resource(FrameNum(0));
        app.insert_resource(PlaceBlock(false));
        app.add_system(update_pause_state);
        // app.add_plugin(TweeningPositionPlugin);
        app.add_plugin(CellPositioningPlugin);

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
            let mut update_block_positions_stage = SystemStage::parallel();
            update_block_positions_stage
                .add_system(handle_block_user_movement)
                .add_system(position_ghost_block.after(handle_block_user_movement))
                // moves the active block down every 1 second
                .add_system_set(
                    SystemSet::new()
                        .with_run_criteria(FixedTimestep::step(1.5))
                        .with_system(move_active_block_down.after(handle_block_user_movement)),
                )
                // checks if the skate timer can be started after block movement
                .add_system(check_skate_timer.after(move_active_block_down))
                .add_system(place_block.after(check_skate_timer));

            app.add_stage_after(
                "spawn_new_blocks",
                "update_block_positions",
                update_block_positions_stage,
            );
        }

        // step 3 - update the Transform of all the sprites that are on the screen
        {
            // let mut update_block_transforms = SystemStage::parallel();
            // app.add_stage_after(
            //     "update_block_positions",
            //     "update_block_transforms",
            //     update_block_transforms,
            // );
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

const BLOCKS: &[BlockName] = &[
    // BlockName::L,
    // BlockName::J,
    // BlockName::O,
    BlockName::I,
    // BlockName::T,
    // BlockName::S,
    // BlockName::Z,
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
        // xxx - consider removing MovableBlock entirely as it contains
        // basically the same state as AbsolutePositionedPiece
        .insert(AbsolutePositionedPiece {
            pos: spawn_at,
            rot: 0,
            def: movable.definition,
        })
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(10., 10.)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., 0., 15.),
                ..default()
            },
            ..default()
        })
        .with_children(|builder| add_cell_children(builder, color, false, &movable))
        .insert(TetrisBlock {
            movable: movable.clone(),
        });

    // the ghost tetris block
    commands
        .spawn()
        .insert_bundle(TransformBundle::identity())
        .with_children(|builder| add_cell_children(builder, color, true, &movable))
        .insert(AbsolutePositionedPiece {
            pos: spawn_at,
            rot: 0,
            def: movable.definition,
        })
        .insert(TetrisBlock { movable })
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(10., 10.)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., 0., 15.),
                ..default()
            },
            ..default()
        })
        .insert(Ghost);
}

fn add_cell_children(
    builder: &mut ChildBuilder,
    color: Color,
    is_ghost: bool,
    movable: &MovableBlock,
) {
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
    fn at_z_level(z: f32) -> Transform {
        Transform {
            translation: Vec3::new(0., 0., z),
            ..default()
        }
    }

    for pos in movable.relative_positions() {
        builder
            .spawn()
            .insert_bundle(TransformBundle::identity())
            .insert(RelativePositionedCell {
                pos,
                def: movable.definition,
            })
            .with_children(|p2| {
                if !is_ghost {
                    p2.spawn().insert_bundle(SpriteBundle {
                        sprite: big_sprite(),
                        transform: at_z_level(10.),
                        ..default()
                    });
                    p2.spawn().insert_bundle(SpriteBundle {
                        sprite: little_sprite(),
                        transform: at_z_level(11.),
                        ..default()
                    });
                } else {
                    p2.spawn()
                        .insert_bundle(TransformBundle::identity())
                        // this position will be updated later to move the block to the lowest point possible on the screen
                        .insert_bundle(SpriteBundle {
                            sprite: ghost_sprite(),
                            transform: at_z_level(9.),
                            ..default()
                        });
                }
            });
    }
}

fn handle_block_user_movement(
    kb: Res<Input<KeyCode>>,
    board_state: Res<Board>,
    mut place_block: ResMut<PlaceBlock>,
    mut active_block_query: Query<(&mut TetrisBlock, &mut AbsolutePositionedPiece), Without<Ghost>>,
) {
    let (mut block, mut app) = match active_block_query.get_single_mut() {
        Ok(ok) => ok,
        _ => return,
    };

    let mut nudge_movable = |dir| {
        let movable = block.movable.move_relative(dir);
        println!(
            "attempting to nudge in {} to {}",
            dir,
            movable.root_position()
        );

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
        println!("hard drop");
        while nudge_movable((0, -1).into()) {}
        println!("block is at {} now", block.movable.root_position());
        place_block.0 = true;
    }

    let mut rotate = |dir| {
        let (movable, kicks) = block.movable.rotate(dir);

        for &kick in kicks {
            let movable = movable.move_relative(kick);
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

    app.pos = block.movable.root_position();
    app.rot = block.movable.rot();
}

fn position_ghost_block(
    mut ghost_query: Query<(&mut TetrisBlock, &mut AbsolutePositionedPiece), With<Ghost>>,
    block_query: Query<&TetrisBlock, Without<Ghost>>,
    board_state: Res<Board>,
) {
    let active = block_query.single();
    let (mut ghost, mut app) = ghost_query.single_mut();

    // from the original active position, move ghost down until it can't be moved
    // further
    ghost.movable = active.movable.clone();
    while board_state.can_place(&ghost.movable.move_relative((0, -1).into())) {
        ghost.movable = ghost.movable.move_relative((0, -1).into());
    }
    app.pos = ghost.movable.root_position();
    app.rot = ghost.movable.rot();
}

fn move_active_block_down(
    paused: Res<Paused>,
    board_state: Res<Board>,
    mut query: Query<&mut TetrisBlock>,
) {
    if paused.0 {
        return;
    }

    for mut block in query.iter_mut() {
        let movable = block.movable.move_relative((0, -1).into());
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
    board_state: Res<Board>,
    time: Res<Time>,
    query: Query<&TetrisBlock, Without<Ghost>>,
) {
    let active_movable = match query.get_single() {
        Ok(block) => &block.movable,
        Err(_) => return,
    };

    if board_state.can_place(&active_movable.move_relative((0, -1).into())) {
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
    mut cell_query: Query<&mut AbsolutePositionedCell>,
    mut board_state: ResMut<Board>,
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
    if board_state.can_place(&active_block.movable.move_relative((0, -1).into())) {
        // then bail out on finalizing block placement
        println!("{} - room below block, bailing", frame_num.0);
        return;
    }

    // no more room to move the block down, finalize plcaement
    board_state.place_block(&active_block.movable, &active_children[..]);

    // add absolute positioning to each placed cell
    let rot = active_block.movable.rot();
    for (pos, &child_ent) in active_block.movable.positions().zip(&active_children[..]) {
        commands
            .entity(child_ent)
            .insert(AbsolutePositionedCell { pos, rot });
    }

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
    let (cleared, moved) = board_state.clear_filled_lines();
    for ent in cleared {
        commands.entity(ent).despawn_recursive();
    }

    // update absolute positions of cells that were moved on the board
    for (ent, pos) in moved {
        if let Ok(mut c) = cell_query.get_component_mut::<AbsolutePositionedCell>(ent) {
            c.pos = pos;
        }
    }
}
