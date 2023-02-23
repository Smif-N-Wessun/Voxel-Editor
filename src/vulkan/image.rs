use ash::vk;
use winit::window::Window;
use std::slice;
use super::{
    Device, 
    DescriptorSet
};

pub struct Image {
    image: vk::Image,
    view: vk::ImageView,
    memory: vk::DeviceMemory,
}

impl Image {
    pub fn new(instance: &ash::Instance, window: &Window, device: &Device, descriptor_set: &DescriptorSet) -> Self {
        let image = unsafe {
            let usage = vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC;
            let extent =  vk::Extent3D { 
                width: window.inner_size().width, 
                height: window.inner_size().height, 
                depth: 1 
            };

            let queue_family_index = &device.queue_family_index();
        
            let create_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .extent(extent)
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .queue_family_indices(slice::from_ref(queue_family_index))
                .initial_layout(vk::ImageLayout::UNDEFINED);

            device.create_image(&create_info, None).expect("Image creation error")
        };
    
        let memory = unsafe {
            let memory_requirements = device.get_image_memory_requirements(image);
            let memory = instance.get_physical_device_memory_properties(device.physical());
            
            let memory_type_index = (0..memory.memory_type_count)
                .find(|i| {
                    let suitable = (memory_requirements.memory_type_bits & (1 << i)) != 0;
                    let memory_type = memory.memory_types[*i as usize];
                    suitable && memory_type.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
                }).expect("Couldn't find suitable memory type");
            
            let allocate_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(memory_requirements.size)
                .memory_type_index(memory_type_index);
    
            device.allocate_memory(&allocate_info, None).expect("Failed to allocate memory")
        };
    
        let view = unsafe  {
            device.bind_image_memory(image, memory, 0).expect("Failed to bind image memory");

            let image_subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .build();
            
            let create_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .subresource_range(image_subresource_range);
            
            device.create_image_view(&create_info, None).expect("Failed to create image view")
        };
    
        unsafe {
            let image_info = vk::DescriptorImageInfo::builder()
                .sampler(vk::Sampler::null())
                .image_view(view)
                .image_layout(vk::ImageLayout::GENERAL);
    
            let write_descriptor_set = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set.set())
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(slice::from_ref(&image_info));
    
            device.update_descriptor_sets(slice::from_ref(&write_descriptor_set), &[] as &[vk::CopyDescriptorSet])
        }

        Self { 
            image, 
            view, 
            memory 
        }
    }

    pub fn destroy_image(&self, device: &Device) {
        unsafe {
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.image, None);
            device.free_memory(self.memory, None);
        }
    }

    pub fn image(&self) -> vk::Image {
        self.image
    }
}