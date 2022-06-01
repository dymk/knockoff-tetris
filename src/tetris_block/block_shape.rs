use bevy::{math::IVec2, prelude::Component};
use lazy_static::lazy_static;

#[derive(Copy, Clone)]
pub enum Block {
    LShape,
    JShape,
    OShape,
    IShape,
}
impl Block {
    pub fn create_movable(&self, at_pos: IVec2) -> MovableBlock {
        match self {
            Block::LShape => MovableBlock {
                position: at_pos,
                rotation: 0,
                shape: &L_SHAPE_CONFIG,
            },
            Block::JShape => MovableBlock {
                position: at_pos,
                rotation: 0,
                shape: &J_SHAPE_CONFIG,
            },
            Block::OShape => MovableBlock {
                position: at_pos,
                rotation: 0,
                shape: &O_SHAPE_CONFIG,
            },
            Block::IShape => MovableBlock {
                position: at_pos,
                rotation: 0,
                shape: &I_SHAPE_CONFIG,
            },
        }
    }
}

pub enum RotDir {
    Right,
    Left,
}

#[derive(Component, Clone)]
pub struct MovableBlock {
    shape: &'static BlockDefinition,
    position: IVec2,
    rotation: u8,
}

pub type Kicks = &'static [IVec2];
impl MovableBlock {
    pub fn at_rotate(&self, rot_dir: RotDir) -> (Self, Kicks) {
        let mut ret = MovableBlock { ..*self };
        let kicks = ret.rotate(rot_dir);
        (ret, kicks)
    }

    pub fn rotate(&mut self, rot_dir: RotDir) -> Kicks {
        let kicks = &match rot_dir {
            RotDir::Right => &self.shape.kicks.right,
            RotDir::Left => &self.shape.kicks.left,
        }[self.rotation as usize];

        let num_rotations = self.shape.rotations.len();
        let self_rot = self.rotation as usize;

        self.rotation = match rot_dir {
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
        kicks
    }

    pub fn positions(
        &self,
    ) -> impl ExactSizeIterator<Item = IVec2> + DoubleEndedIterator<Item = IVec2> + '_ {
        self.shape.rotations[self.rotation as usize]
            .iter()
            .map(|&loc| loc + self.position)
    }

    pub fn at_nudged(&self, by: IVec2) -> MovableBlock {
        MovableBlock {
            position: self.position + by,
            ..*self
        }
    }
    pub fn nudge(&mut self, by: IVec2) {
        self.position += by;
    }
}

struct LRKicks {
    pub left: Vec<Vec<IVec2>>,
    pub right: Vec<Vec<IVec2>>,
}
struct BlockDefinition {
    pub rotations: Vec<Vec<IVec2>>,
    pub kicks: &'static LRKicks,
}

lazy_static! {
    static ref STANDARD_KICKS: LRKicks = LRKicks {
        left: conv_tuples_2(&[
            // 0 -> 1
            &[(0, 0), (1, 0), (-1, 0)],
            // 1 -> 2
            &[(0, 0), (1, 0), (-1, 0)],
            // 2 -> 3
            &[(0, 0), (1, 0), (-1, 0)],
            // 3 -> 0
            &[(0, 0), (1, 0), (-1, 0)],
        ]),
        right: conv_tuples_2(&[
            // 0 -> 1
            &[(0, 0), (1, 0), (-1, 0)],
            // 1 -> 2
            &[(0, 0), (1, 0), (-1, 0)],
            // 2 -> 3
            &[(0, 0), (1, 0), (-1, 0)],
            // 3 -> 0
            &[(0, 0), (1, 0), (-1, 0)],
        ])
    };
    static ref I_KICKS: LRKicks = LRKicks {
        left: conv_tuples_2(&[
            // 0 -> 1
            &[(0, 0)],
            // 1 -> 0
            &[(0, 0), (1, 0), (-1, 0), (2, 0), (-2, 0)],
        ]),
        right: conv_tuples_2(&[
            // 0 -> 1
            &[(0, 0)],
            // 1 -> 0
            &[(0, 0), (1, 0), (-1, 0), (2, 0), (-2, 0)],
        ])
    };

    static ref IDENTITY_KICK: &'static [(i32, i32)] = &[(0, 0)];

    static ref NO_KICKS: LRKicks = LRKicks {
        left: conv_tuples_2(&[&[(0, 0)]]),
        right: conv_tuples_2(&[&[(0, 0)]])
    };


    #[rustfmt::skip]
    static ref L_SHAPE_CONFIG: BlockDefinition = BlockDefinition {
        rotations: build_rotations(4, false, &[
                             (1, 1),
            (-1, 0), (0, 0), (1, 0)
        ]),
        kicks: &STANDARD_KICKS
    };

    #[rustfmt::skip]
    static ref J_SHAPE_CONFIG: BlockDefinition = BlockDefinition {
        rotations: build_rotations(4, false, &[
            (-1, 1),
            (-1, 0), (0, 0), (1, 0)
        ]),
        kicks: &STANDARD_KICKS
    };

    #[rustfmt::skip]
    static ref O_SHAPE_CONFIG: BlockDefinition = BlockDefinition {
        rotations: build_rotations(1, false, &[
            (0, 1), (1, 1),
            (0, 0), (1, 0)
        ]),
        kicks: &NO_KICKS
    };

    #[rustfmt::skip]
    static ref I_SHAPE_CONFIG: BlockDefinition = BlockDefinition {
        rotations: build_rotations(2, true, &[
            (-2, 0), (-1, 0), (0, 0), (1, 0)
        ]),
        kicks: &I_KICKS
    };

    static ref DOT_CONFIG: BlockDefinition = BlockDefinition { rotations: build_rotations(1, false, &[(0, 0)]), kicks: &STANDARD_KICKS };
    pub static ref TEST_MOVABLE_BLOCK: MovableBlock = MovableBlock {
        position: IVec2::new(0, 0),
        rotation: 0,
        shape: &DOT_CONFIG,
    };
}

fn conv_tuples(list: &[(i32, i32)]) -> Vec<IVec2> {
    list.iter().map(|&l| Into::into(l)).collect()
}
fn conv_tuples_2(list: &[&[(i32, i32)]]) -> Vec<Vec<IVec2>> {
    list.iter().map(|&l| conv_tuples(l)).collect()
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
            let vec = (vec - 1) / 2;
            vec
        }
    };

    let mut ret = Vec::new();
    ret.reserve(num_rotations);

    ret.push(list);
    for _ in 0..(num_rotations - 1) {
        let rotated = ret.last().unwrap().iter().map(|&l| rotate(l)).collect();
        ret.push(rotated);
    }

    ret
}

#[cfg(test)]
mod test {
    use crate::tetris_block::block_shape::conv_tuples_2;

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
