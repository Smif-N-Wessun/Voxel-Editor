use ash::vk;
use std::{
    slice, 
    mem::size_of,
    ptr::copy_nonoverlapping as memcpy,
};
use super::{
    Device,
    DescriptorSet,
};

pub struct StagingBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
}

impl StagingBuffer {
    pub fn new(instance: &ash::Instance, device: &Device, size: u64) -> Self {
        let (buffer, memory) = new_buffer(
            instance, 
            device, 
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE, 
            size
        );

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

    pub fn write<T: Sized>(&self, device: &Device, data: &T) {
        unsafe {
            let bytes = std::slice::from_raw_parts(
                (data as *const T) as *const u8,
                std::mem::size_of::<T>(),
            );

            let memory = device.map_memory(
                self.memory, 
                0, 
                size_of::<T>() as u64, 
                vk::MemoryMapFlags::empty()
            ).unwrap();

            memcpy(bytes.as_ptr(), memory.cast(), size_of::<T>());

            device.unmap_memory(self.memory);
        };
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn _memory(&self) -> vk::DeviceMemory {
        self.memory
    }
}

pub struct LocalBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
}

impl LocalBuffer {
    pub fn new(instance: &ash::Instance, device: &Device, descriptor_set: &DescriptorSet, size: u64, binding: u32) -> Self {
        let (buffer, memory) = new_buffer(
            instance,
            device, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL, 
            size
        );

        unsafe {
            let buffer_info = vk::DescriptorBufferInfo::builder()
                .buffer(buffer)
                .offset(0)
                .range(size as vk::DeviceSize);
    
            let write_descriptor_set = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set.set())
                .dst_binding(binding)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(slice::from_ref(&buffer_info));
    
            device.update_descriptor_sets(slice::from_ref(&write_descriptor_set), &[] as &[vk::CopyDescriptorSet]);
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

    pub fn _memory(&self) -> vk::DeviceMemory {
        self.memory
    }
}

fn new_buffer(instance: &ash::Instance, device: &Device, memory_flags: vk::MemoryPropertyFlags, size: u64) -> (vk::Buffer, vk::DeviceMemory) {
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

    unsafe { device.bind_buffer_memory(buffer, memory, 0).expect("Failed to bind buffer memory") };

    (buffer, memory)
}