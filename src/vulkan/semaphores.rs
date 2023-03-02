use std::ops::Index;
use ash::vk;
use super::{
    Device, 
    Swapchain,
};

#[derive(Clone, Copy)]
pub struct SemaphorePair {
    image_available: vk::Semaphore,
    render_complete: vk::Semaphore,
}

impl SemaphorePair {
    pub fn image_available(&self) -> vk::Semaphore {
        self.image_available
    }

    pub fn render_complete(&self) -> vk::Semaphore {
        self.render_complete
    }
}

pub struct Semaphores {
    pairs: Vec<SemaphorePair>,
}

impl Semaphores {
    pub fn new(device: &Device, swapchain: &Swapchain) -> Self {
        let create_info = vk::SemaphoreCreateInfo::builder();

        let mut pairs = Vec::default();

        for _ in swapchain.present_images() {
            let pair = unsafe {
                SemaphorePair {
                    image_available: device.create_semaphore(&create_info, None).expect("Semaphore creation error"),
                    render_complete: device.create_semaphore(&create_info, None).expect("Semaphore creation error"),
                }
            };
            pairs.push(pair);
        }

        Self {
            pairs,
        }
    }

    pub fn destroy_semaphore(&self, device: &Device) {
        unsafe {
            self.pairs.iter().for_each(|pair| {
                device.destroy_semaphore(pair.image_available, None);
                device.destroy_semaphore(pair.render_complete, None);
            })
        }
    }  
}

impl Index<usize> for Semaphores {
    type Output = SemaphorePair;

    fn index(&self, index: usize) -> &Self::Output {
        &self.pairs[index]
    }
}