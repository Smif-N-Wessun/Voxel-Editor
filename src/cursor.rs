use nalgebra::Vector3;

#[repr(C)]
pub struct Cursor {
    pos: Vector3<f32>,
    highlighted_side: u32,
}