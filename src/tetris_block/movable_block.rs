use std::f32::consts::TAU;

use crate::{
    tetris_block::{block_definition::LRKicks, tuple_util::conv_tuples},
    CELL_SIDE_LEN, GRID_CELLS,
};

use super::block_definition::BlockDefinition;
use bevy::{
    math::{IVec2, Quat, Vec3},
    prelude::{default, Component, Transform},
};
use lazy_static::lazy_static;

#[derive(Copy, Clone, Debug)]
pub enum BlockName {
    L,
    J,
    O,
    I,
    T,
    S,
    Z,
    Test,
}
impl BlockName {
    pub fn create_movable(&self, at_pos: IVec2) -> MovableBlock {
        match self {
            BlockName::L => MovableBlock::new(at_pos, &L_SHAPE_CONFIG),
            BlockName::J => MovableBlock::new(at_pos, &J_SHAPE_CONFIG),
            BlockName::O => MovableBlock::new(at_pos, &O_SHAPE_CONFIG),
            BlockName::I => MovableBlock::new(at_pos, &I_SHAPE_CONFIG),
            BlockName::T => MovableBlock::new(at_pos, &T_SHAPE_CONFIG),
            BlockName::S => MovableBlock::new(at_pos, &S_SHAPE_CONFIG),
            BlockName::Z => MovableBlock::new(at_pos, &Z_SHAPE_CONFIG),
            BlockName::Test => MovableBlock::new(at_pos, &DOT_CONFIG),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RotDir {
    Left,
    Right,
}

pub type Kicks = &'static [IVec2];

#[derive(Component, Clone)]
pub struct MovableBlock {
    pub definition: &'static BlockDefinition,
    position: IVec2,
    rotation: u8,
    rotation_continuous: i32,
}

impl MovableBlock {
    pub fn new(position: IVec2, definition: &'static BlockDefinition) -> MovableBlock {
        MovableBlock {
            definition,
            position,
            rotation: 0,
            rotation_continuous: 0,
        }
    }

    pub fn rot(&self) -> i32 {
        self.rotation_continuous
    }

    pub fn rotation(&self) -> f32 {
        if self.definition.rotations.len() == 1 {
            return 0.;
        } else {
            (self.rotation_continuous as f32) / (self.definition.rotations.len() as f32)
        }
    }

    pub fn root_position(&self) -> IVec2 {
        self.position
    }

    pub fn rotate(&mut self, rot_dir: RotDir) -> (Self, Kicks) {
        let kicks = &match rot_dir {
            RotDir::Right => &self.definition.kicks.right,
            RotDir::Left => &self.definition.kicks.left,
        }[self.rotation as usize];

        let num_rotations = self.definition.rotations.len();
        let self_rot = self.rotation as usize;

        self.rotation_continuous += match rot_dir {
            RotDir::Left => -1,
            RotDir::Right => 1,
        };

        let rotation = match rot_dir {
            RotDir::Right => {
                if self_rot == num_rotations - 1 {
                    0
                } else {
                    self_rot + 1
                }
            }
            RotDir::Left => {
                if self_rot == 0 {
                    num_rotations - 1
                } else {
                    self_rot - 1
                }
            }
        } as u8;
        (MovableBlock { rotation, ..*self }, kicks)
    }

    pub fn positions(
        &self,
    ) -> impl ExactSizeIterator<Item = IVec2> + DoubleEndedIterator<Item = IVec2> + '_ {
        self.definition.rotations[self.rotation as usize]
            .iter()
            .map(|&loc| loc + self.position)
    }

    pub fn relative_positions(
        &self,
    ) -> impl ExactSizeIterator<Item = IVec2> + DoubleEndedIterator<Item = IVec2> + '_ {
        self.definition.rotations[self.rotation as usize]
            .iter()
            .copied()
    }

    pub fn move_relative(&self, by: IVec2) -> MovableBlock {
        MovableBlock {
            position: self.position + by,
            ..*self
        }
    }

    pub fn around_corner(&self) -> bool {
        self.definition.around_corner
    }
}

lazy_static! {
    static ref STANDARD_KICKS: LRKicks = LRKicks::new(
        // right
        &[
            // 0 -> 1
            &[(-1, 0),(-1, 1),( 0,-2),(-1,-2)],
            // 1 -> 2
            &[( 1, 0),( 1,-1),( 0, 2),( 1, 2)],
            // 2 -> 3
            &[( 1, 0),( 1, 1),( 0,-2),( 1,-2)],
            // 3 -> 0
            &[(-1, 0),(-1,-1),( 0, 2),(-1, 2)]
        ],
        // left
        &[
            // 0 -> 3
            &[( 1, 0),( 1, 1),( 0,-2),( 1,-2)],
            // 1 -> 0
            &[( 1, 0),( 1,-1),( 0, 2),( 1, 2)],
            // 2 -> 1
            &[(-1, 0),(-1, 1),( 0,-2),(-1,-2)],
            // 3 -> 2
            &[(-1, 0),(-1,-1),( 0, 2),(-1, 2)]
        ]
    );
    static ref I_KICKS: LRKicks = LRKicks::new(
        // right
        &[
            // 0 -> 1
            &[(-2, 0), (1, 0), (-2, -1), (1, 2)],
            // 1 -> 2
            &[(-1, 0), (2, 0), (-1, -2), (2, -1)],
            // 2 -> 3
            &[( 2, 0), (-1, 0), ( 2, 1), (-1,-2)],
            // 3 -> 0
            &[( 1, 0), (-2, 0), ( 1,-2), (-2, 1)],
        ],
        // left
        &[
            // 0 -> 3
            &[(-1, 0), ( 2, 0), (-1, 2), ( 2,-1)],
            // 1 -> 0
            &[( 2, 0), (-1, 0), ( 2, 1), (-1,-2)],
            // 2 -> 1
            &[( 1, 0), (-2, 0), ( 1,-2), (-2, 1)],
            // 3 -> 2
            &[(-2, 0), ( 1, 0), (-2,-1), ( 1, 2)],
        ]
    );

    // used for blocks that have only have a single rotation state
    static ref NO_KICKS: LRKicks = LRKicks::new(&[&[]], &[&[]]);

    #[rustfmt::skip]
    static ref L_SHAPE_CONFIG: BlockDefinition = BlockDefinition::new(
        build_rotations(4, false, &[
                             (1, 1),
            (-1, 0), (0, 0), (1, 0)
        ]),
        STANDARD_KICKS.clone(),
        false
    );

    #[rustfmt::skip]
    static ref J_SHAPE_CONFIG: BlockDefinition = BlockDefinition::new(
        build_rotations(4, false, &[
            (-1, 1),
            (-1, 0), (0, 0), (1, 0)
        ]),
        STANDARD_KICKS.clone(),
        false
    );

    #[rustfmt::skip]
    static ref O_SHAPE_CONFIG: BlockDefinition = BlockDefinition::new(
        build_rotations(1, false, &[
            (0, 1), (1, 1),
            (0, 0), (1, 0)
        ]),
        NO_KICKS.clone(),
        false
    );

    #[rustfmt::skip]
    static ref I_SHAPE_CONFIG: BlockDefinition = BlockDefinition::new(
        build_rotations(4, true, &[
            (-2, 0), (-1, 0), (0, 0), (1, 0)
        ]),
        I_KICKS.clone(),
        true
    );

    #[rustfmt::skip]
    static ref T_SHAPE_CONFIG: BlockDefinition = BlockDefinition::new(
        build_rotations(4, false, &[
                     (0, 1),
            (-1, 0), (0, 0), (1, 0)
        ]),
        STANDARD_KICKS.clone(),
        false
    );

    #[rustfmt::skip]
    static ref S_SHAPE_CONFIG: BlockDefinition = BlockDefinition::new(
        build_rotations(4, false, &[
                     (0, 1), (1, 1),
            (-1, 0), (0, 0),
        ]),
        STANDARD_KICKS.clone(),
        false
    );

    #[rustfmt::skip]
    static ref Z_SHAPE_CONFIG: BlockDefinition = BlockDefinition::new(
        build_rotations(4, false, &[
            (-1, 1), (0, 1),
                     (0, 0), (1, 0),
        ]),
        STANDARD_KICKS.clone(),
        false
    );

    static ref DOT_CONFIG: BlockDefinition = BlockDefinition::new(build_rotations(1, false, &[(0, 0)]), NO_KICKS.clone(), false);
}

fn build_rotations(
    num_rotations: usize,
    rot_around_corner: bool,
    list: &[(i32, i32)],
) -> Vec<Vec<IVec2>> {
    assert!(num_rotations > 0);
    let list = conv_tuples(list);

    let rotate = |vec: IVec2| {
        if !rot_around_corner {
            IVec2::new(vec.y, -vec.x)
        } else {
            let vec = (vec * 2) + 1;
            let vec = IVec2::new(vec.y, -vec.x);
            (vec - 1) / 2
        }
    };

    let mut ret = Vec::new();
    ret.reserve(num_rotations);

    ret.push(list);
    for _ in 0..(num_rotations - 1) {
        let rotated = ret.last().unwrap().iter().copied().map(rotate).collect();
        ret.push(rotated);
    }

    ret
}

#[cfg(test)]
mod test {
    pub fn conv_tuples_2(list: &[&[(i32, i32)]]) -> Vec<Vec<IVec2>> {
        list.iter().map(|&l| conv_tuples(l)).collect()
    }

    use bevy::math::IVec2;

    use crate::tetris_block::tuple_util::conv_tuples;

    use super::build_rotations;

    #[test]
    fn test_build_rotations() {
        let rots = build_rotations(1, false, &[(0, 0), (1, 0), (2, 0)]);
        #[rustfmt::skip]
        assert_eq!(rots, conv_tuples_2(&[
            &[(0, 0), (1, 0), (2, 0)],
        ]));

        let rots = build_rotations(2, false, &[(0, 0), (1, 0), (2, 0)]);
        #[rustfmt::skip]
        assert_eq!(rots, conv_tuples_2(&[
            &[(0, 0), (1, 0), (2, 0)],
            &[
                (0, 0),
                (0, -1),
                (0, -2)
            ],
        ]));

        let rots = build_rotations(2, true, &[(0, 0), (1, 0), (2, 0)]);
        #[rustfmt::skip]
        assert_eq!(rots, conv_tuples_2(&[
            &[(0, 0), (1, 0), (2, 0)],
            &[
                (0, -1),
                (0, -2),
                (0, -3)
            ],
        ]));

        let rots = build_rotations(2, false, &[(0, 0)]);
        #[rustfmt::skip]
        assert_eq!(rots, conv_tuples_2(&[
            &[(0, 0)],
            &[(0, 0)],
        ]));

        let rots = build_rotations(4, true, &[(0, 0)]);
        #[rustfmt::skip]
        assert_eq!(rots, conv_tuples_2(&[
            &[(0, 0)],
            &[(0, -1)],
            &[(-1, -1)],
            &[(-1, 0)],
        ]));
    }
}
