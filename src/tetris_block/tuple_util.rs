use bevy::math::IVec2;

pub fn conv_tuples(list: &[(i32, i32)]) -> Vec<IVec2> {
    list.iter().map(|&l| Into::into(l)).collect()
}
