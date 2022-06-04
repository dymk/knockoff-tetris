use bevy::math::IVec2;

use super::tuple_util::conv_tuples;

#[derive(Clone)]
pub struct LRKicks {
    pub right: Vec<Vec<IVec2>>,
    pub left: Vec<Vec<IVec2>>,
}
impl LRKicks {
    pub fn new(right: &[&[(i32, i32)]], left: &[&[(i32, i32)]]) -> LRKicks {
        assert!(right.len() == left.len());
        assert!(right
            .iter()
            .map(|slice| slice.len())
            .all(|len| len == right[0].len()));
        assert!(left
            .iter()
            .map(|slice| slice.len())
            .all(|len| len == left[0].len()));

        LRKicks {
            right: tuples_to_kicks(right),
            left: tuples_to_kicks(left),
        }
    }
}

pub struct BlockDefinition {
    pub rotations: Vec<Vec<IVec2>>,
    pub kicks: LRKicks,
}
impl BlockDefinition {
    pub fn new(rotations: Vec<Vec<IVec2>>, kicks: LRKicks) -> BlockDefinition {
        assert!(rotations.len() == kicks.left.len());
        assert!(rotations.len() == kicks.right.len());
        BlockDefinition { rotations, kicks }
    }
}

fn tuples_to_kicks(list: &[&[(i32, i32)]]) -> Vec<Vec<IVec2>> {
    list.iter()
        .map(|&l| {
            let mut head = conv_tuples(&[(0, 0)]);
            head.append(&mut conv_tuples(l));
            head
        })
        .collect()
}
