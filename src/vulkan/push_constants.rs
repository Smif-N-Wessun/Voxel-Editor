#![allow(dead_code)]

use nalgebra::{
    Vector3, 
    Vector4
};

#[repr(C)]
pub struct Camera {
    pub origin: Vector4<f32>,
    pub lower_left_corner: Vector4<f32>,
    pub horizontal: Vector4<f32>,
    pub vertical: Vector4<f32>,
}

impl Camera {
    pub fn new(
        look_from: Vector3<f32>, 
        look_at: Vector3<f32>, 
        vector_up: Vector3<f32>, 
        field_of_view: f32, 
        aspect_ratio: f32
    ) -> Self {
        let theta = field_of_view.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = (look_at - look_from).normalize();
        let u = w.cross(&vector_up).normalize();
        let v = u.cross(&w);

        let origin = look_from.to_homogeneous();
        let horizontal = (viewport_width * u).to_homogeneous();
        let vertical = (viewport_height * v).to_homogeneous();
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 + w.to_homogeneous();

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
}
}