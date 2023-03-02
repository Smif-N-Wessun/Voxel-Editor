use std::ops::Index;
use ash::vk;
use super::{
    Device, 
    Swapchain,
};

pub struct Fences {
    fences: Vec<vk::Fence>,
}

impl Fences {
    pub fn new(device: &Device, swapchain: &Swapchain) -> Self {
        let create_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        let mut fences = Vec::default();

        for _ in swapchain.present_images() {
            let fence = unsafe { device.create_fence(&create_info, None).expect("Fence creation error") };
            
            fences.push(fence);
        }

        Self { 
            fences,
        }
    }

    pub fn destroy_fences(&self, device: &Device) {
        unsafe { self.fences.iter().for_each(|fence| device.destroy_fence(*fence, None)) }
    }
}

impl Index<usize> for Fences {
    type Output = vk::Fence;

    fn index(&self, index: usize) -> &Self::Output {
        &self.fences[index]
    }
}

pub trait FenceCommands {
    fn wait(self, device: &Device);
    fn reset(self, device: &Device);
}

impl FenceCommands for vk::Fence {
    fn wait(self, device: &Device) {
        unsafe { 
            device.wait_for_fences(
                &[self],
                true, 
                u64::max_value()
            ).unwrap() 
        }
    }

    fn reset(self, device: &Device) {
        unsafe { device.reset_fences(&[self]).unwrap() }
    }
}