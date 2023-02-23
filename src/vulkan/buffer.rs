use ash::vk;
use std::slice;
use super::{
    Device,
    DescriptorSet,
};

pub struct Buffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
}

impl Buffer {
    pub fn new(instance: &ash::Instance, device: &Device, descriptor_set: &DescriptorSet, memory_flags: vk::MemoryPropertyFlags, size: u64) -> Self {
        let buffer = unsafe {
            let usage_flags = vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST;

            let queue_family_index = &device.queue_family_index();
    
            let create_info = vk::BufferCreateInfo::builder()
                .size(size)
                .usage(usage_flags)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .queue_family_indices(slice::from_ref(queue_family_index));

            device.create_buffer(&create_info, None).expect("Buffer creation error") 
        };

        let memory = unsafe {
            let memory_requirements = device.get_buffer_memory_requirements(buffer);
            let memory = instance.get_physical_device_memory_properties(device.physical());
            
            let memory_type_index = (0..memory.memory_type_count)
                .find(|i| {
                    let suitable = (memory_requirements.memory_type_bits & (1 << i)) != 0;
                    let memory_type = memory.memory_types[*i as usize];
                    suitable && memory_type.property_flags.contains(memory_flags)
                }).expect("Couldn't find suitable memory type");
            
            let allocate_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(memory_requirements.size)
                .memory_type_index(memory_type_index);

            device.allocate_memory(&allocate_info, None).expect("Failed to allocate memory")
        };
        
        unsafe {
            device.bind_buffer_memory(buffer, memory, 0).expect("Failed to bind buffer memory");

            if memory_flags == vk::MemoryPropertyFlags::DEVICE_LOCAL {
                let buffer_info = vk::DescriptorBufferInfo::builder()
                    .buffer(buffer)
                    .offset(0)
                    .range(size as vk::DeviceSize);
    
                let write_descriptor_set = vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_set.set())
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(slice::from_ref(&buffer_info));
    
                device.update_descriptor_sets(slice::from_ref(&write_descriptor_set), &[] as &[vk::CopyDescriptorSet])
            }
        }

        Self { 
            buffer, 
            memory, 
        }
    }

    pub fn destroy_buffer(&self, device: &Device) {
        unsafe {
            device.free_memory(self.memory, None);
            device.destroy_buffer(self.buffer, None);
        }
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn memory(&self) -> vk::DeviceMemory {
        self.memory
    }
}