use nalgebra::{
    Vector3, 
    Vector4
};
use winit::event::{
    VirtualKeyCode, 
    ElementState
};

#[repr(C)]
pub struct CameraProjection {
    origin: Vector4<f32>,
    upper_left_corner: Vector4<f32>,
    horizontal: Vector4<f32>,
    vertical: Vector4<f32>,
}

pub struct CameraController {
    delta_move: Vector3<f32>,
}

impl Default for CameraController {
    fn default() -> Self {
        Self { 
            delta_move: Default::default(), 
        }
    }
}

pub struct Camera {
    look_from: Vector3<f32>, 
    look_at: Vector3<f32>, 
    vector_up: Vector3<f32>, 
    field_of_view: f32, 
    aspect_ratio: f32,
    controller: CameraController,
}

impl Camera {
    pub fn new(look_from: Vector3<f32>, look_at: Vector3<f32>) -> Self {
        Self { 
            look_from, 
            look_at, 
            vector_up: Vector3::new(0.0, 0.0, 1.0), 
            field_of_view: 45.0, 
            aspect_ratio: 16.0 / 9.0, 
            controller: CameraController::default(),
        }
    }

    pub fn projection(&mut self) -> CameraProjection {
        self.look_from += self.controller.delta_move;

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
        let upper_left_corner = origin - horizontal * 0.5 + vertical * 0.5 + w.to_homogeneous();

        CameraProjection {
            origin,
            upper_left_corner,
            horizontal,
            vertical,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) {
        let delta = match state {
            ElementState::Pressed => 0.01,
            ElementState::Released => 0.0,
        };

        match key {
            VirtualKeyCode::W => {
                self.controller.delta_move.z = delta;
            },
            VirtualKeyCode::S => {
                self.controller.delta_move.z = -delta;
            },
            VirtualKeyCode::D => {
                self.controller.delta_move.x = delta;
            },
            VirtualKeyCode::A => {
                self.controller.delta_move.x = -delta;
            },
            _ => (),
        }
    }
}