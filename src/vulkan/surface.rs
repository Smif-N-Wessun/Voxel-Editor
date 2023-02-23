use ash::{ 
    vk,
    extensions::khr, 
    prelude::VkResult
};
use winit::window::Window;
use super::Device;

pub struct Surface {
    surface: vk::SurfaceKHR,
    loader: khr::Surface,
}

impl Surface {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance, window: &Window) -> Self {
        let loader = khr::Surface::new(&entry, &instance);
        let surface = unsafe { ash_window::create_surface(&entry, &instance, &window, None).expect("Surface creation error") };

        Self { 
            surface, 
            loader, 
        }
    }

    pub fn destroy_surface(&self) {
        unsafe { self.loader.destroy_surface(self.surface, None) }
    }

    pub unsafe fn get_physical_device_surface_support(&self, physical_device: vk::PhysicalDevice, queue_family_index: u32) -> VkResult<bool> {
        self.loader.get_physical_device_surface_support(physical_device, queue_family_index, self.surface)
    }

    pub unsafe fn get_physical_device_surface_formats(&self, device: &Device) -> VkResult<Vec<vk::SurfaceFormatKHR>> {
        self.loader.get_physical_device_surface_formats(device.physical(), self.surface)
    }

    pub unsafe fn get_physical_device_surface_capabilities(&self, device: &Device) -> VkResult<vk::SurfaceCapabilitiesKHR>  {
        self.loader.get_physical_device_surface_capabilities(device.physical(), self.surface)
    }

    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }
}
