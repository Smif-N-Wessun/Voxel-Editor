use nalgebra::{
    Vector3, 
    Vector4
};
use winit::event::{
    VirtualKeyCode, 
    ElementState
};

#[allow(dead_code)]
#[repr(C)]
pub struct CameraProjection {
    origin: Vector4<f32>,
    lower_left_corner: Vector4<f32>,
    horizontal: Vector4<f32>,
    vertical: Vector4<f32>,
}

pub struct Camera {
    look_from: Vector3<f32>, 
    look_at: Vector3<f32>, 
    vector_up: Vector3<f32>, 
    field_of_view: f32, 
    aspect_ratio: f32
}

impl Camera {
    pub fn new(look_from: Vector3<f32>, look_at: Vector3<f32>) -> Self {
        Self { 
            look_from, 
            look_at, 
            vector_up: Vector3::new(0.0, 0.0, 1.0), 
            field_of_view: 45.0, 
            aspect_ratio: 16.0 / 9.0, 
        }
    }

    pub fn projection(&self) -> CameraProjection {
        let theta = self.field_of_view.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = self.aspect_ratio * viewport_height;

        let w = (self.look_at - self.look_from).normalize();
        let u = w.cross(&self.vector_up).normalize();
        let v = u.cross(&w);

        let origin = self.look_from.to_homogeneous();
        let horizontal = (viewport_width * u).to_homogeneous();
        let vertical = (viewport_height * v).to_homogeneous();
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 + w.to_homogeneous();

        CameraProjection {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) {
        if state == ElementState::Pressed {
            match key {
                VirtualKeyCode::W => {
                    self.look_from.z += 0.25;
                },
                VirtualKeyCode::A => {
                    self.look_from.x -= 0.25;
                },
                VirtualKeyCode::S => {
                    self.look_from.z -= 0.25;
                },
                VirtualKeyCode::D => {
                    self.look_from.x += 0.25;
                },
                _ => (),
            }
        } 
    }
}