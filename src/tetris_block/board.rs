use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use bevy::prelude::*;

use super::movable_block::MovableBlock;

type BoardCell = Option<Entity>;
// #[derive(Clone, Copy, Eq, PartialEq)]
// enum BoardCell {
//     Empty,
//     Placed(Entity),
// }

pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<BoardCell>,
}
impl Board {
    pub fn new(width: usize, height: usize) -> Board {
        let cells = vec![None; width * height];

        Board {
            width,
            height,
            cells,
        }
    }

    // pub fn spawn_shape()

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }

    fn to_idx(&self, vec: IVec2) -> usize {
        ((self.width as i32 * vec.y) + vec.x) as usize
    }
    fn to_ivec(&self, idx: usize) -> IVec2 {
        IVec2::new((idx % self.width) as i32, (idx / self.width) as i32)
    }

    pub fn cell(&self, loc: IVec2) -> BoardCell {
        self.cells[self.to_idx(loc)]
    }
    pub fn cell_mut(&mut self, loc: IVec2) -> &mut BoardCell {
        let idx = self.to_idx(loc);
        &mut self.cells[idx]
    }

    pub fn iter_ents(&self) -> impl Iterator<Item = (IVec2, Entity)> + '_ {
        self.cells
            .iter()
            .enumerate()
            .filter_map(|(idx, &ent)| ent.map(|ent| (self.to_ivec(idx), ent)))
    }

    pub fn can_place(&self, block: &MovableBlock) -> bool {
        block.positions().all(|loc| !self.is_occupied(loc))
    }

    pub fn place_block(&mut self, block: &MovableBlock, ents: &[Entity]) {
        assert!(block.positions().len() == ents.len());
        for (idx, loc) in block.positions().enumerate() {
            self.set_occupied(loc, ents[idx]);
        }
    }

    fn set_occupied(&mut self, loc: IVec2, entity: Entity) {
        assert!(self.cell(loc).is_none());
        *self.cell_mut(loc) = Some(entity);
    }

    pub fn clear_filled_lines(&mut self) -> (HashSet<Entity>, HashMap<Entity, IVec2>) {
        let mut cleared_entities = HashSet::new();
        let mut moved_entities = HashMap::new();

        // from the top of the board, to the bottom, check full lines
        for row in (0..self.height).rev() {
            if self.is_row_full(row as usize) {
                // remove all the entities in this row
                for col in 0..self.width() {
                    let pos = IVec2::new(col as i32, row as i32);
                    if let Some(ent) = self.cell_mut(pos).take() {
                        cleared_entities.insert(ent);
                    }
                }

                for row_ in row..(self.height() - 1) {
                    // move everything from the rows above down one 'y' position
                    for col in 0..self.width() {
                        let from = IVec2::new(col as i32, (row_ + 1) as i32);
                        let to = IVec2::new(col as i32, row_ as i32);

                        let cell = self.cell(from);
                        if let Some(ent) = cell {
                            moved_entities.insert(ent, to);
                        }
                        *self.cell_mut(to) = cell;
                        *self.cell_mut(from) = None;
                    }
                }
            }
        }

        (cleared_entities, moved_entities)
    }

    fn is_occupied(&self, loc: IVec2) -> bool {
        if loc.x < 0 || loc.y < 0 || loc.x >= (self.width as i32) || loc.y >= (self.height as i32) {
            return true;
        }

        if self.cell(loc).is_some() {
            return true;
        }

        false
    }

    fn rows(
        &self,
    ) -> impl Iterator<Item = &[BoardCell]>
           + DoubleEndedIterator<Item = &[BoardCell]>
           + ExactSizeIterator<Item = &[BoardCell]>
           + '_ {
        self.cells.chunks(self.width)
    }

    pub fn is_row_full(&self, row: usize) -> bool {
        self.cells[(row * self.width as usize)..((row + 1) * self.width as usize)]
            .iter()
            .all(|cell| cell.is_some())
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Board State({})\n", self.rows().len()))?;
        let spacer = "-".repeat(self.width * 2) + "\n";
        f.write_str(spacer.as_str())?;

        for row in self.rows().rev() {
            let r = row
                .iter()
                .map(|elem| match elem {
                    Some(_) => "██",
                    None => "..",
                })
                .collect::<String>();

            f.write_str(r.as_str())?;
            f.write_str("\n")?
        }
        f.write_str(spacer.as_str())?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::tetris_block::movable_block::BlockName;

    use super::Board;

    #[test]
    fn test() {
        let board = Board::new(3, 3);
        assert!(!board.is_occupied((0, 0).into()));
        assert!(board.is_occupied((-1, 0).into()));

        let block = BlockName::Test.create_movable((0, 0).into());
        assert!(board.can_place(&block));
        assert!(board.can_place(&block.move_relative((1, 1).into())));
        assert!(!board.can_place(&block.move_relative((-1, 0).into())));
        assert!(!board.can_place(&block.move_relative((3, 0).into())));
    }
}
