use bevy::prelude::*;

use crate::{components::ActiveBlock, GRID_CELLS};

use super::GridLocation;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BoardCell {
    Empty,
    Occupied(Entity),
}

pub struct BoardState {
    cells: Vec<Vec<BoardCell>>,
}
impl BoardState {
    pub fn new() -> BoardState {
        let cells = (0..GRID_CELLS.y)
            .map(|_| vec![BoardCell::Empty; GRID_CELLS.x as usize])
            .collect::<Vec<_>>();

        BoardState { cells }
    }

    pub fn clear(&mut self) {
        for row in self.cells.iter_mut() {
            for cell in row.iter_mut() {
                *cell = BoardCell::Empty;
            }
        }
    }

    pub fn set_location_occupied(&mut self, loc: IVec2, entity: Entity) {
        self.cells[loc.y as usize][loc.x as usize] = BoardCell::Occupied(entity);
    }

    pub fn is_occupied(&self, loc: IVec2) -> bool {
        if loc.x < 0 || loc.y < 0 || loc.x >= GRID_CELLS.x || loc.y >= GRID_CELLS.y {
            return false;
        }

        if let BoardCell::Occupied(_) = self.cells[loc.y as usize][loc.x as usize] {
            return false;
        }

        true
    }

    pub fn is_row_full(&self, row: usize) -> bool {
        self.cells[row].iter().all(|cell| {
            if let BoardCell::Occupied(_) = cell {
                true
            } else {
                false
            }
        })
    }

    pub fn remove_row(&mut self, row: usize) -> Vec<Entity> {
        let row = self.cells.remove(row);
        self.cells
            .push(vec![BoardCell::Empty; GRID_CELLS.x as usize]);
        return row
            .iter()
            .filter_map(|cell| match cell {
                BoardCell::Empty => None,
                BoardCell::Occupied(ent) => Some(*ent),
            })
            .collect();
    }

    pub fn peek_row(&self, row: usize) -> Vec<Entity> {
        self.cells[row]
            .iter()
            .filter_map(|cell| match cell {
                BoardCell::Empty => None,
                BoardCell::Occupied(ent) => Some(*ent),
            })
            .collect()
    }
}

pub fn rebuild_board_state(
    mut board_state: ResMut<BoardState>,
    query: Query<(Entity, &GridLocation), Without<ActiveBlock>>,
) {
    board_state.clear();
    for (entity, gl) in query.iter() {
        board_state.set_location_occupied(gl.loc, entity);
    }
}

pub fn clear_filled_lines(
    mut commands: Commands,
    mut query: Query<&mut GridLocation, Without<ActiveBlock>>,
    mut board_state: ResMut<BoardState>,
) {
    // from the top of the board, to the bottom, check full lines
    for row in (0..GRID_CELLS.y).rev() {
        if board_state.is_row_full(row as usize) {
            println!("row {} is full", row);

            // remove all the entities in that row
            for ent in board_state.remove_row(row as usize) {
                commands.entity(ent).despawn_recursive();
            }

            // for all the rows that come after, shift them down by one
            // don't need to go from (row+1) because the row was already removed
            for row_ in row..GRID_CELLS.y {
                println!("shifting all on row {} down one...", row_);
                for ent in board_state.peek_row(row_ as usize) {
                    println!("e: {:?}", ent);
                    if let Ok(mut gl) = query.get_mut(ent) {
                        println!("shifting {:?} down a 'y'", gl.loc);
                        gl.loc.y -= 1;
                    }
                }
            }
        }
    }
}
