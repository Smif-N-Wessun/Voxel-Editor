use nalgebra::Vector4;

#[repr(C)]
pub struct Cursor {
    pos: Vector4<f32>,
    normal: Vector4<i32>,
}