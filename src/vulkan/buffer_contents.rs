#![allow(dead_code)]

use nalgebra::{
    Vector3, 
    Vector4
};

pub const MODEL_SIZE: usize = 4;
const MODELS_N: usize = 1; // Number of models in scene

pub struct BoundingBox {
    pub min_bound: Vector4<f32>,
    pub max_bound: Vector4<f32>,
}

pub struct Model {
    pub bounding_box: BoundingBox,
    pub voxels: [u32; MODEL_SIZE * MODEL_SIZE * MODEL_SIZE],
}

pub struct Scene {
    pub model: Model,
}

impl BoundingBox {
    pub fn new(min_bound: Vector3<f32>, max_bound: Vector3<f32>) -> Self {
        Self { 
            min_bound: min_bound.to_homogeneous(), 
            max_bound: max_bound.to_homogeneous(),
        }
    }
}

impl Model {
    pub fn new(min_bound: Vector3<f32>, voxel_data: Vec<(Vector3<usize>, u32)>) -> Self {
        let bounding_box = BoundingBox::new(
            min_bound, 
            min_bound.add_scalar(MODEL_SIZE as f32),
        );

        let mut voxels = [0u32; MODEL_SIZE.pow(3)];

        voxel_data.iter().for_each(|&(location, color_index)| voxels[Self::location_to_index(location)] = color_index);

        Self {
            bounding_box,
            voxels,
        }
    }

    fn location_to_index(location: Vector3<usize>) -> usize {
        location.x + location.y * MODEL_SIZE + location.z * MODEL_SIZE.pow(2)
    }
}