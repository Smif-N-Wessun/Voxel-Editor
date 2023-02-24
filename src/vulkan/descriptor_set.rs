use ash::vk;
use std::slice;
use super::Device;

pub struct DescriptorSet {
    set: vk::DescriptorSet,
    layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
}

impl DescriptorSet {
    pub fn new(device: &Device) -> Self {
        let pool = unsafe {
            let world_buffer = vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .build();
        
            let raytrace_output_image = vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .build();

            let pool_sizes = &[
                world_buffer, 
                raytrace_output_image
            ];

            let create_info = vk::DescriptorPoolCreateInfo::builder()
                .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET | vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
                .pool_sizes(pool_sizes)
                .max_sets(1);

            device.create_descriptor_pool(&create_info, None).expect("Descriptor pool creation error")
        };

        let layout = unsafe {
            let world_buffer = vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE)
                .build();
    
            let raytrace_output_image = vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE)
                .build();
    
            let layout_bindings = &[
                world_buffer,
                raytrace_output_image,
            ];

            let descriptor_binding_flags = {
                let flags = vk::DescriptorBindingFlags::UPDATE_AFTER_BIND;
                vec![flags; layout_bindings.len()]
            };

            let mut layout_binding_flags_create_info = vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder()
                .binding_flags(&descriptor_binding_flags[..]);
    
            let layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
                .push_next(&mut layout_binding_flags_create_info)
                .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
                .bindings(layout_bindings);

            device.create_descriptor_set_layout(&layout_create_info, None).expect("Descriptor set layout creation error")
        };

        let set = unsafe {
            let create_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(pool)
                .set_layouts(slice::from_ref(&layout));
            
            device.allocate_descriptor_sets(&create_info).expect("Descriptor set allocation error")[0]
        };

        Self {
            set, 
            layout, 
            pool, 
        }
    }

    pub fn destroy_descriptor_set(&self, device: &Device) {
        unsafe {
            device.destroy_descriptor_pool(self.pool, None);
            device.destroy_descriptor_set_layout(self.layout, None);
        }
    }

    pub fn set(&self) -> vk::DescriptorSet {
        self.set
    }

    pub fn layout(&self) -> vk::DescriptorSetLayout {
        self.layout
    }
}