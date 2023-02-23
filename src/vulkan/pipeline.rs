use ash::vk;
use std::{
    ffi::CStr, 
    slice
};
use super::{
    Device, 
    DescriptorSet
};

pub struct Pipeline {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}

impl Pipeline {
    pub fn new(device: &Device, descriptor_set: &DescriptorSet, source: &[u8], push_constant_size: u32) -> Self {
        let layout = unsafe {
            let push_constant_range = vk::PushConstantRange::builder()
                .stage_flags(vk::ShaderStageFlags::COMPUTE)
                .offset(0)
                .size(push_constant_size);

            let descriptor_set_layout = &descriptor_set.layout();
    
            let create_info = vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(slice::from_ref(descriptor_set_layout))
                .push_constant_ranges(slice::from_ref(&push_constant_range));
    
            device.create_pipeline_layout(&create_info, None).expect("Pipeline layout creation error")
        };

        let shader_module = unsafe {
            let (prefix, code, suffix) = source.align_to::<u32>();
    
            if !prefix.is_empty() || !suffix.is_empty() {
                panic!("Shader bytecode is not properly aligned.");
            }
    
            let create_info = vk::ShaderModuleCreateInfo::builder().code(code);
            device.create_shader_module(&create_info, None).expect("Shader module creation error") 
        };

        let pipeline = unsafe {            
            let shader_stage = vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::COMPUTE)
                .module(shader_module)
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap());
        
            let create_info = vk::ComputePipelineCreateInfo::builder()
                .stage(shader_stage.build())
                .layout(layout);
        
            device
                .create_compute_pipelines(vk::PipelineCache::null(), slice::from_ref(&create_info), None)
                .expect("Compute pipeline creation error")[0]
        };

        unsafe { device.destroy_shader_module(shader_module, None) };

        Self { 
            pipeline, 
            layout, 
        }
    }

    pub fn destroy_pipeline(&self, device: &Device) {
        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.layout, None);
        }
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }
}