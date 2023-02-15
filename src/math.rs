pub fn round_vec_to_i32(vec: cgmath::Vector3<f32>) -> cgmath::Vector3<i32> {
    vec.map(|n| n.floor() as i32)
}
