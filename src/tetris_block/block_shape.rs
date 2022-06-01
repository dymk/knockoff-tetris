use bevy::{math::IVec2, prelude::Component};
use lazy_static::lazy_static;

#[derive(Copy, Clone)]
pub enum BlockShape {
    LShape,
}
impl BlockShape {
    pub fn create_descriptor(&self, at_pos: IVec2) -> BlockShapeDescriptor {
        match self {
            BlockShape::LShape => BlockShapeDescriptor {
                position: at_pos,
                relative_locs: L_SHAPE_CONFIG.locs.to_vec(),
                rot_around_corner: L_SHAPE_CONFIG.rot_around_corner,
            },
        }
    }
}

#[derive(Component, Clone)]
pub struct BlockShapeDescriptor {
    position: IVec2,
    relative_locs: Vec<IVec2>,
    rot_around_corner: bool,
}

impl BlockShapeDescriptor {
    pub fn rotate(&mut self) {
        let rot_around_corner = self.rot_around_corner;
        for relative_loc in self.relative_locs.iter_mut() {
            *relative_loc = Self::rotate_(rot_around_corner, *relative_loc);
        }
    }

    pub fn rotate_back(&mut self) {
        self.rotate();
        self.rotate();
        self.rotate();
    }

    pub fn locs(&self) -> impl Iterator<Item = IVec2> + '_ {
        self.relative_locs.iter().map(|&loc| loc + self.position)
    }

    pub fn nudge(&mut self, by: IVec2) {
        self.position += by;
    }

    fn rotate_(rot_around_corner: bool, vec: IVec2) -> IVec2 {
        let mut vec = vec;

        if !rot_around_corner {
            IVec2::new(vec.y, -vec.x)
        } else {
            vec = (vec * 2) + 1;
            vec = IVec2::new(vec.y, -vec.x);
            vec = (vec - 1) / 2;
            vec
        }
    }
}

pub struct BlockShapeOffsets {
    locs: [IVec2; 4],
    rot_around_corner: bool,
}

lazy_static! {
    #[rustfmt::skip]
    pub static ref L_SHAPE_CONFIG: BlockShapeOffsets = BlockShapeOffsets {
        locs: conv_list([
            (0,  1),
            (0,  0),
            (0, -1), (1, -1)
        ]),
        rot_around_corner: false
    };
}

fn conv_list<const N: usize>(list: [(i32, i32); N]) -> [IVec2; N] {
    list.map(|(x, y)| IVec2::new(x, y))
}
