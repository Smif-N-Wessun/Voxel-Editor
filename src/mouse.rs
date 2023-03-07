use ash::vk;
use nalgebra::Vector2;
use winit::{
    event::{
        MouseButton, 
        ElementState
    }, 
    dpi::PhysicalPosition, 
    window::Window
};


#[repr(C)]
pub struct MouseState {
    coordinate: Vector2<f32>,
    pub left_button: vk::Bool32,
    right_button: vk::Bool32,
}

#[repr(C)]
pub struct Mouse {
    coordinate: Vector2<f32>,
    left_button: vk::Bool32,
    right_button: vk::Bool32,
}

impl Mouse {
    pub fn process_input(&mut self, button: MouseButton, state: ElementState) {
        let pressed = match state {
            ElementState::Pressed => vk::TRUE,
            ElementState::Released => vk::FALSE,
        };

        match button {
            MouseButton::Left => self.left_button = pressed,
            MouseButton::Right => self.right_button = pressed,
            _ => (),
        };
    }

    pub fn process_movement(&mut self, position: PhysicalPosition<f64>, window: &Window) {
        self.coordinate = Vector2::new(
            position.cast::<f32>().x / window.inner_size().width as f32, 
            position.cast::<f32>().y / window.inner_size().height as f32,
        );
    }

    pub fn state(&mut self) -> MouseState {
        let left_button = self.left_button;
        self.left_button = vk::FALSE;

        MouseState { 
            coordinate: self.coordinate, 
            left_button,
            right_button: self.right_button, 
        }
    }
}   

impl Default for Mouse {
    fn default() -> Self {
        Self { 
            coordinate: Default::default(), 
            left_button: Default::default(), 
            right_button: Default::default(),
        }
    }
}