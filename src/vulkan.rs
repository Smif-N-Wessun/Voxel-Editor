mod instance;
mod debug_messenger;
mod device;
mod surface;
mod swapchain;
mod command_buffer;
mod descriptor_set;
mod pipeline;
mod buffer;
mod image;
mod semaphore;

use nalgebra::Vector3;
use std::{
    mem::size_of, 
    ptr::copy_nonoverlapping as memcpy,
};

use super::{
    octree::Octree,
    camera::{
        Camera, 
        CameraProjection
    },
};

use ash::vk;
use winit::{
    event::{
        ElementState, 
        Event, 
        KeyboardInput, 
        VirtualKeyCode, 
        WindowEvent 
    },
    event_loop::{
        ControlFlow, 
        EventLoop 
    },
    window::{ 
        Window, 
        WindowBuilder 
    },
};
use self::{
    debug_messenger::DebugMessenger,
    instance::InstanceTrait,
    surface::Surface,
    device::Device,
    swapchain::Swapchain,
    semaphore::Semaphore,
    command_buffer::CommandBuffer,
    descriptor_set::DescriptorSet,
    buffer::Buffer,
    image::Image, 
    pipeline::Pipeline,
};

#[allow(dead_code)]
pub struct App {
    window: Window,
    event_loop: Option<EventLoop<()>>,
    instance: ash::Instance,
    debug_messenger: DebugMessenger,
    surface: Surface,
    device: Device,
    swapchain: Swapchain,
    semaphore: Semaphore,
    command_buffer: CommandBuffer,
    descriptor_set: DescriptorSet,
    pipeline: Pipeline,
    world_buffer: Buffer,
    raytrace_output_image: Image,
}

impl App {
    pub fn new() -> Self {
        let app_name = "Voxel editor";
        let entry = ash::Entry::linked();

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(app_name)
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)
            .expect("Window error");

        let instance = ash::Instance::new(&entry, &window, app_name);
        let debug_messenger = DebugMessenger::new(&entry, &instance);
        let surface = Surface::new(&entry, &instance, &window);
        let device = Device::new(&instance, &surface);
        let swapchain = Swapchain::new(&instance, &device, &surface);
        let semaphore = Semaphore::new(&device);
        let command_buffer = CommandBuffer::new(&device);
        let descriptor_set = DescriptorSet::new(&device);
        let pipeline = Pipeline::new(&device, &descriptor_set, include_bytes!("../shaders/spv/raytrace.spv"), 64);
        let world_buffer = Buffer::new_local(&instance, &device, &descriptor_set, size_of::<super::Octree>() as u64);
        let raytrace_output_image = Image::new(&instance, &window, &device, &descriptor_set);

        Self {
            event_loop: Some(event_loop),
            window,
            instance,
            debug_messenger,
            surface,
            device,
            swapchain,
            semaphore,
            command_buffer,
            descriptor_set,
            pipeline,
            world_buffer,
            raytrace_output_image,
        }
    }

    pub fn prepare(&self, octree: Octree) {
        let staging_buffer = Buffer::new_staging(
            &self.instance, 
            &self.device, 
            size_of::<super::Octree>() as u64,
        );

        let bytes = unsafe { std::slice::from_raw_parts((&octree as *const Octree) as *const u8, size_of::<Octree>()) };
        
        let memory = unsafe {
            self.device.map_memory(
                staging_buffer.memory(), 
                0, 
                size_of::<Octree>() as u64, 
                vk::MemoryMapFlags::empty()
            ).unwrap()
        };

        unsafe { 
            memcpy(bytes.as_ptr(), memory.cast(), size_of::<Octree>());
            self.device.unmap_memory(staging_buffer.memory());
        };

        // Transition image layouts
        self.command_buffer.begin(&self.device);

        for image in self.swapchain.present_images() {
            self.command_buffer.pipeline_barrier(
                &self.device, 
                *image,
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::PRESENT_SRC_KHR),
                (vk::AccessFlags::empty(), vk::AccessFlags::empty()),
                (vk::PipelineStageFlags::TOP_OF_PIPE,  vk::PipelineStageFlags::TOP_OF_PIPE),
            );
        }

        // Transition raytrace output image layout
        self.command_buffer.pipeline_barrier(
            &self.device, 
            self.raytrace_output_image.image(), 
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_SRC_OPTIMAL),
            (vk::AccessFlags::empty(), vk::AccessFlags::empty()),
            (vk::PipelineStageFlags::TOP_OF_PIPE,  vk::PipelineStageFlags::TOP_OF_PIPE),
        );

        self.command_buffer.copy_buffer(
            &self.device, 
            staging_buffer.buffer(), 
            self.world_buffer.buffer(), 
            size_of::<Octree>() as u64
        );

        self.command_buffer.end(&self.device);
        self.command_buffer.submit_single_time(&self.device);

        staging_buffer.destroy_buffer(&self.device);
    }

    fn render(&mut self, camera: &CameraProjection) {
        let image_index = unsafe { self.swapchain.acquire_next_image(self.semaphore.image_available()).unwrap().0 };

        self.command_buffer.begin(&self.device);
        self.command_buffer.bind_descriptor_sets(&self.device, &self.pipeline, &self.descriptor_set);
        self.command_buffer.bind_pipeline(&self.device, &self.pipeline);

        self.command_buffer.push_constants(&self.device, camera, &self.pipeline);
        self.command_buffer.dispatch(&self.device, self.window.inner_size().width / 16, self.window.inner_size().height / 16, 1);
        self.command_buffer.pipeline_barrier(
            &self.device, 
            self.swapchain.present_images()[image_index as usize], 
            (vk::ImageLayout::PRESENT_SRC_KHR, vk::ImageLayout::TRANSFER_DST_OPTIMAL),
            (vk::AccessFlags::TRANSFER_READ, vk::AccessFlags::TRANSFER_WRITE),
            (vk::PipelineStageFlags::TRANSFER,  vk::PipelineStageFlags::TRANSFER),
        );
        self.command_buffer.copy_image(
            &self.device, 
            (self.raytrace_output_image.image(), vk::ImageLayout::TRANSFER_SRC_OPTIMAL),
            (self.swapchain.present_images()[image_index as usize],  vk::ImageLayout::TRANSFER_DST_OPTIMAL),
            vk::Extent3D { width: self.window.inner_size().width, height: self.window.inner_size().height, depth: 1 },
        );
        self.command_buffer.pipeline_barrier(
            &self.device, 
            self.swapchain.present_images()[image_index as usize], 
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::PRESENT_SRC_KHR),
            (vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::TRANSFER_READ),
            (vk::PipelineStageFlags::TRANSFER,  vk::PipelineStageFlags::TRANSFER),
        );
        self.command_buffer.end(&self.device);
        self.command_buffer.submit_commands(&self.device, self.semaphore.image_available(), self.semaphore.render_complete());

        self.swapchain.present_frame(&self.device, image_index, self.semaphore.render_complete());
    }

    fn quit(&self, control_flow: &mut ControlFlow) {
        unsafe { self.device.device_wait_idle().unwrap() };
        *control_flow = ControlFlow::Exit;
    }

    pub fn run(mut self) {
        let mut camera = Camera::new(
            Vector3::new(12.0, 0.0, 11.0), // Look from
            Vector3::new(12.0, 12.0, 9.0)  // Look at
        );

        let event_loop = self.event_loop.take().unwrap();
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::MainEventsCleared => self.render(&camera.projection()),
                Event::WindowEvent {
                    event, 
                    ..
                } => match event {
                    WindowEvent::CloseRequested => self.quit(control_flow),
                    WindowEvent::CursorMoved {
                        position,
                        ..
                    } => println!("X: {}, Y: {}", position.x, position.y),
                    WindowEvent::KeyboardInput  {
                        input: KeyboardInput {
                            virtual_keycode: Some(key),
                            state,
                            ..
                        },
                        ..
                    } => match (key, state) {
                        (VirtualKeyCode::Escape, ElementState::Pressed) => self.render(&camera.projection()),
                        (key, state) => camera.process_keyboard(key, state),
                    },
                    _ => (),
                },
                _ => ()
            }
        });
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.raytrace_output_image.destroy_image(&self.device);
            self.world_buffer.destroy_buffer(&self.device);
            self.pipeline.destroy_pipeline(&self.device);
            self.descriptor_set.destroy_descriptor_set(&self.device);
            self.command_buffer.destroy_command_buffer(&self.device);
            self.semaphore.destroy_semaphore(&self.device);
            self.swapchain.destroy_swapchain(&self.device);
            self.device.destroy_device(None);
            self.surface.destroy_surface();
            self.debug_messenger.destroy_debug_messenger();
            self.instance.destroy_instance(None);
        }
    }
}