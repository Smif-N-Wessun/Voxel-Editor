use nalgebra::{
    Vector3, 
    Vector4
};

const MEMORY_SIZE: usize = 1024; 
const STACK_SIZE: usize = 23;
const ROOT_ADDRESS: usize = MEMORY_SIZE - 1;

#[allow(dead_code)] // Not read in rust, only copied to GPU buffer
#[repr(C)]
pub struct Bounds {
    min: Vector4<f32>, // Even though min and max are 3 dimensional vectors they need to be Vector4
    max: Vector4<f32>, // becuase vec3 are 4 bytes aligned in GLSL
}

impl Bounds {
    fn new(location: Vector3<i32>, size: u32) -> Self {
        let min = nalgebra::convert::<Vector3<i32>, Vector3<f32>>(location);
        let max = min.add_scalar(size as f32);

        Self { 
            min: min.to_homogeneous(), 
            max: max.to_homogeneous(), 
        }
    }
}

#[allow(dead_code)]
#[repr(C)]
pub struct Octree {
    bounds: Bounds,
    descriptors: [u32; 1024],
}

impl Octree {
    pub fn new(location: Vector3<i32>, size: u32, voxels: Vec<Vector3<f32>>) -> Self {
        let descriptors = {
            let mut descriptors = [u32::default(); MEMORY_SIZE];
            let mut stack = [u32::default(); (STACK_SIZE + 1) as usize];
            let mut free_address = 0;
        
            let lowest_scale = 20 + 1; // Lowest non-leaf voxel scale
        
            for mut pos in voxels {
                // Find voxel's ancestors
                for scale in lowest_scale..=STACK_SIZE {
                    let shx = pos.x.to_bits() >> scale;
                    let shy = pos.y.to_bits() >> scale;
                    let shz = pos.z.to_bits() >> scale;
                    let prime_x = f32::from_bits(shx << scale);
                    let prime_y = f32::from_bits(shy << scale);
                    let prime_z = f32::from_bits(shz << scale);
        
                    let mut idx: u32 = 0;
                
                    if pos.x > prime_x {
                        idx |= 1;
                    }
                    if pos.y > prime_y {
                        idx |= 1 << 1;
                    }
                    if pos.z > prime_z {
                        idx |= 1 << 2;
                    }
        
                    pos.x = prime_x;
                    pos.y = prime_y;
                    pos.z = prime_z;
        
                    stack[scale as usize] = 1 << idx;
                }
                // Put them into octree
                let mut parent_valid_mask = stack[STACK_SIZE];
                let mut parent_child_pointer = 0;
        
                descriptors[ROOT_ADDRESS] |= parent_valid_mask;
        
                for scale in (lowest_scale..STACK_SIZE).rev() {
                    let address = parent_child_pointer + (31 - parent_valid_mask.leading_zeros());
                    let valid_mask = stack[scale as usize];
                    let mut current_descriptor = descriptors[address as usize];
        
                    current_descriptor |= valid_mask;
        
                    if current_descriptor >> 8 == 0 && scale != lowest_scale {
                        free_address += 8;
                        current_descriptor |= free_address << 8;
                    }
        
                    descriptors[address as usize] = current_descriptor;
                    parent_valid_mask = valid_mask;
                    parent_child_pointer = free_address;
                }
            }
            descriptors
        };

        Self { 
            bounds: Bounds::new(location, size),
            descriptors, 
        }
    }
}
