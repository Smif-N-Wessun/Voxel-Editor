use ash::vk;
use std::slice;
use super::{
    Device, 
    Pipeline, 
    DescriptorSet
};

pub struct CommandBuffer {
    buffer: vk::CommandBuffer,
    pool: vk::CommandPool,
}

impl CommandBuffer{
    pub fn new(device: &Device) -> Self {
        let pool = unsafe {
            let create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(device.queue_family_index())
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
    
            device.create_command_pool(&create_info, None).expect("Command pool creation error")
        };

        let buffer = unsafe {
            let create_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);

            device.allocate_command_buffers(&create_info).expect("Command buffer creation error")[0]
        };

        Self {
            buffer, 
            pool 
        }
    }

    pub fn destroy_command_buffer(&self, device: &Device) {
        unsafe { device.destroy_command_pool(self.pool, None) }
    }

    pub fn begin(&self, device: &Device) {
        let begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe { device.begin_command_buffer(self.buffer, &begin_info).unwrap() }
    }

    pub fn end(&self, device: &Device) {
        unsafe { device.end_command_buffer(self.buffer).unwrap() }
    }

    pub fn _reset(&self, device: &Device) {
        unsafe { device.reset_command_buffer(self.buffer, vk::CommandBufferResetFlags::empty()).unwrap() }
    }

    pub fn submit_single_time(&self, device: &Device) {
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(slice::from_ref(&self.buffer));

        unsafe { 
            device.queue_submit(device.queue(), slice::from_ref(&submit_info), vk::Fence::null()).unwrap();
            device.queue_wait_idle(device.queue()).unwrap();
        };
    }

    pub fn submit_commands(&self, device: &Device, wait_semaphore: vk::Semaphore,signal_semaphore: vk::Semaphore) {
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(slice::from_ref(&wait_semaphore))
            .wait_dst_stage_mask(slice::from_ref(&vk::PipelineStageFlags::ALL_COMMANDS))
            .command_buffers(slice::from_ref(&self.buffer))
            .signal_semaphores(slice::from_ref(&signal_semaphore));

        unsafe {
            device.queue_submit(device.queue(), slice::from_ref(&submit_info), vk::Fence::null()).unwrap();
            device.queue_wait_idle(device.queue()).unwrap();
        };
    }

    pub fn bind_descriptor_sets(&self, device: &Device, pipeline: &Pipeline, descriptor_set: &DescriptorSet) {
        unsafe {
            device.cmd_bind_descriptor_sets(
                self.buffer, 
                vk::PipelineBindPoint::COMPUTE, 
                pipeline.layout(), 
                0,
                slice::from_ref(&descriptor_set.set()), 
                &[],
            )
        }
    }

    pub fn bind_pipeline(&self, device: &Device, pipeline: &Pipeline) {
        unsafe { device.cmd_bind_pipeline(self.buffer, vk::PipelineBindPoint::COMPUTE, pipeline.pipeline()) }
    }

    pub fn push_constants<T: Sized>(&self, device: &Device, constants: &T, pipeline: &Pipeline) {
        unsafe {
            let bytes = std::slice::from_raw_parts(
                (constants as *const T) as *const u8,
                std::mem::size_of::<T>(),
            );

            device.cmd_push_constants(self.buffer, pipeline.layout(), vk::ShaderStageFlags::COMPUTE, 0, bytes);
        }
    }

    pub fn dispatch(&self, device: &Device, group_count_x: u32, group_count_y: u32, group_count_z: u32) {
        unsafe { device.cmd_dispatch(self.buffer, group_count_x, group_count_y, group_count_z) }
    }

    pub fn pipeline_barrier(
        &self, 
        device: &Device,
        image: vk::Image,
        layouts: (vk::ImageLayout, vk::ImageLayout),
        access_masks: (vk::AccessFlags, vk::AccessFlags),
        stage_masks: (vk::PipelineStageFlags, vk::PipelineStageFlags),
    ) {
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(layouts.0)
            .new_layout(layouts.1)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(subresource_range)
            .src_access_mask(access_masks.0)
            .dst_access_mask(access_masks.1);

        unsafe {
            device.cmd_pipeline_barrier(
                self.buffer, 
                stage_masks.0, 
                stage_masks.1, 
                vk::DependencyFlags::empty(), 
                &[] as &[vk::MemoryBarrier], 
                &[] as &[vk::BufferMemoryBarrier], 
                slice::from_ref(&barrier),
            );
        }
    }

    pub fn copy_image(
        &self,
        device: &Device, 
        src_image_and_layout: (vk::Image, vk::ImageLayout),
        dst_image_and_layout: (vk::Image, vk::ImageLayout),
        extent: vk::Extent3D
    ) {
        let subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let region = vk::ImageCopy::builder()
            .src_subresource(subresource)
            .src_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .dst_subresource(subresource)
            .src_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .extent(extent);

        unsafe {
            device.cmd_copy_image(
                self.buffer, 
                src_image_and_layout.0, 
                src_image_and_layout.1,
                dst_image_and_layout.0, 
                dst_image_and_layout.1, 
                slice::from_ref(&region),
            );
        }
    }

    pub fn copy_buffer(
        &self,
        device: &Device,
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        size: u64
    ) {
        let regions = vk::BufferCopy::builder().size(size);
        unsafe { device.cmd_copy_buffer(self.buffer, src_buffer, dst_buffer, slice::from_ref(&regions)) }
    }
}