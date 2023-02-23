use ash::vk;
use super::Device;

pub struct Semaphore {
    image_available: vk::Semaphore,
    render_complete: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: &Device) -> Self {
        let create_info = vk::SemaphoreCreateInfo::builder();

        let image_available = unsafe { device.create_semaphore(&create_info, None).expect("Semaphore creation error") };
        let render_complete = unsafe { device.create_semaphore(&create_info, None).expect("Semaphore creation error") };

        Self {
            image_available,
            render_complete 
        }
    }

    pub fn destroy_semaphore(&self, device: &Device) {
        unsafe {
            device.destroy_semaphore(self.image_available, None);
            device.destroy_semaphore(self.render_complete, None);
        }
    }   

    pub fn image_available(&self) -> vk::Semaphore {
        self.image_available
    }

    pub fn render_complete(&self) -> vk::Semaphore {
        self.render_complete
    }
}
