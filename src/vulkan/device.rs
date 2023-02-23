use ash::vk;
use std::slice;
use super::Surface;

pub struct Device {
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
    queue: vk::Queue,
    queue_family_index: u32,
}

impl Device {
    pub fn new(instance: &ash::Instance, surface: &Surface) -> Self {
        let physical_device = unsafe { 
            let physical_devices = instance.enumerate_physical_devices().expect("Physical device error");

            let is_suitable = |device: vk::PhysicalDevice, device_type: vk::PhysicalDeviceType| {
                let properties = instance.get_physical_device_properties(device);
                properties.device_type == device_type && properties.api_version.ge(&vk::API_VERSION_1_2) 
            };
    
            match physical_devices.iter().find(|&&device| is_suitable(device, vk::PhysicalDeviceType::DISCRETE_GPU)) {
                Some(device) => *device,
                None => match physical_devices.iter().find(|&&device| is_suitable(device, vk::PhysicalDeviceType::INTEGRATED_GPU)) {
                    Some(device) => *device,
                    None => panic!("Physical device error"),
                }
            }
        };
        
        let queue_family_index = unsafe {
            let properties = instance.get_physical_device_queue_family_properties(physical_device);

            properties.into_iter().enumerate().find(|(index, info)| {
                info.queue_flags.contains(vk::QueueFlags::COMPUTE) 
                && surface.get_physical_device_surface_support(physical_device, *index as u32).unwrap() 
            }).expect("Could not find suitable queue family").0 as u32
        };

        let device = unsafe { 
            let device_extension_names = vec![ash::extensions::khr::Swapchain::name().as_ptr()];

            let create_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&[1.0]);
    
            let create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(slice::from_ref(&create_info))
                .enabled_extension_names(&device_extension_names);

            instance.create_device(physical_device, &create_info, None).expect("Device creation error") 
        };

        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        Self {
            device,
            physical_device,
            queue,
            queue_family_index,
        }
    }

    pub fn physical(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn queue(&self) -> vk::Queue {
        self.queue
    }

    pub fn queue_family_index(&self) -> u32 {
        self.queue_family_index
    }
}

impl std::ops::Deref for Device {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}