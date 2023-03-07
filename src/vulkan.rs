mod instance;
mod debug_messenger;
mod device;
mod surface;
mod swapchain;
mod command_buffers;
mod descriptor_set;
mod pipeline;
mod buffers;
mod image;
mod semaphores;
mod fences;

use ash::vk;
use nalgebra::{
    Vector2, 
    Vector3,
    Vector4,
};
use std::mem::size_of;
use winit::{
    event::{
        ElementState, 
        Event, 
        KeyboardInput, 
        VirtualKeyCode, 
        WindowEvent, 
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
use {
    super::octree::Octree,
    super::cursor::Cursor,
    super::mouse::{
        Mouse, 
        MouseState
    },
    super::camera::{
        Camera, 
        CameraProjection
    },
};

use self::{
    debug_messenger::DebugMessenger,
    instance::InstanceTrait,
    surface::Surface,
    device::Device,
    swapchain::Swapchain,
    semaphores::Semaphores,
    descriptor_set::DescriptorSet,
    buffers::{
        LocalBuffer, 
        StagingBuffer
    },
    image::Image, 
    pipeline::Pipeline, 
    command_buffers::{
        CommandBuffers, 
        CommandBufferCommands 
    },
    fences::{
        Fences, 
        FenceCommands
    }, 
};

pub struct App {
    window: Window,
    event_loop: Option<EventLoop<()>>,
    instance: ash::Instance,
    debug_messenger: DebugMessenger,
    surface: Surface,
    device: Device,
    swapchain: Swapchain,
    semaphores: Semaphores,
    fences: Fences,
    command_buffers: CommandBuffers,
    descriptor_set: DescriptorSet,
    raytrace_pipeline: Pipeline,
    mouse_raytrace_pipeline: Pipeline,
    edit_pipeline: Pipeline,
    world_buffer: LocalBuffer,
    cursor_buffer: LocalBuffer,
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
        let semaphores = Semaphores::new(&device, &swapchain);
        let fences = Fences::new(&device, &swapchain);
        let command_buffers = command_buffers::CommandBuffers::new(&device, &swapchain);
        let descriptor_set = DescriptorSet::new(&device);
        let raytrace_pipeline = Pipeline::new(
            &device, 
            &descriptor_set, 
            include_bytes!("../shaders/spv/raytrace.spv"), 
            (size_of::<CameraProjection>() + size_of::<MouseState>())  as u32
        );
        let mouse_raytrace_pipeline = Pipeline::new(
            &device, 
            &descriptor_set, 
            include_bytes!("../shaders/spv/raytrace_mouse.spv"), 
            (size_of::<CameraProjection>() + size_of::<MouseState>())  as u32
        );
        let edit_pipeline = Pipeline::new(
            &device, 
            &descriptor_set, 
            include_bytes!("../shaders/spv/edit.spv"), 
            size_of::<MouseState>()  as u32
        );
        let world_buffer = LocalBuffer::new(&instance, &device, &descriptor_set, size_of::<Octree>() as u64, 0);
        let cursor_buffer = LocalBuffer::new(&instance, &device, &descriptor_set, size_of::<Cursor>() as u64, 1);
        let raytrace_output_image = Image::new(&instance, &window, &device, &descriptor_set, 2);

        Self {
            event_loop: Some(event_loop),
            window,
            instance,
            debug_messenger,
            surface,
            device,
            swapchain,
            semaphores,
            fences,
            command_buffers,
            descriptor_set,
            raytrace_pipeline,
            mouse_raytrace_pipeline,
            edit_pipeline,
            world_buffer,
            cursor_buffer,
            raytrace_output_image,
        }
    }

    pub fn prepare(&self) {
        let command_buffer = self.command_buffers[0];
        
        // Transition image layouts
        command_buffer.begin(&self.device);

        for image in self.swapchain.present_images() {
            command_buffer.pipeline_barrier(
                &self.device, 
                *image,
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::PRESENT_SRC_KHR),
                (vk::AccessFlags::empty(), vk::AccessFlags::empty()),
                (vk::PipelineStageFlags::TOP_OF_PIPE,  vk::PipelineStageFlags::TOP_OF_PIPE),
            );
        }

        // Transition raytrace output image layout
        command_buffer.pipeline_barrier(
            &self.device, 
            self.raytrace_output_image.image(), 
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_SRC_OPTIMAL),
            (vk::AccessFlags::empty(), vk::AccessFlags::empty()),
            (vk::PipelineStageFlags::TOP_OF_PIPE,  vk::PipelineStageFlags::TOP_OF_PIPE),
        );

        command_buffer.end(&self.device);
        command_buffer.submit_single_time(&self.device);
    }

    fn render(&mut self, camera: CameraProjection, mouse: MouseState) {
        let (present_image, image_index) = self.swapchain.acquire_next_image(&self.semaphores);
        let fence = self.fences[image_index];
        let semaphore = self.semaphores[image_index];
        let command_buffer = self.command_buffers[image_index];
        let push_constant = (camera, mouse);

        if push_constant.1.left_button == ash::vk::TRUE {
            println!("Click");
        }

        fence.wait(&self.device);
        
        command_buffer.begin(&self.device);
        
        command_buffer.bind_descriptor_sets(&self.device, &self.edit_pipeline, &self.descriptor_set);
        command_buffer.bind_pipeline(&self.device, &self.edit_pipeline);
        command_buffer.push_constants(&self.device, &push_constant.1, &self.edit_pipeline);
        command_buffer.dispatch(&self.device, 1, 1, 1);

        command_buffer.bind_descriptor_sets(&self.device, &self.mouse_raytrace_pipeline, &self.descriptor_set);
        command_buffer.bind_pipeline(&self.device, &self.mouse_raytrace_pipeline);
        command_buffer.push_constants(&self.device, &push_constant, &self.mouse_raytrace_pipeline);
        command_buffer.dispatch(&self.device, 1, 1, 1);

        command_buffer.bind_descriptor_sets(&self.device, &self.raytrace_pipeline, &self.descriptor_set);
        command_buffer.bind_pipeline(&self.device, &self.raytrace_pipeline);
        command_buffer.push_constants(&self.device, &push_constant, &self.raytrace_pipeline);
        command_buffer.dispatch(
            &self.device, 
            self.window.inner_size().width / 16,
            self.window.inner_size().height / 16, 
            1
        );

        command_buffer.pipeline_barrier(
            &self.device, 
            present_image, 
            (vk::ImageLayout::PRESENT_SRC_KHR, vk::ImageLayout::TRANSFER_DST_OPTIMAL),
            (vk::AccessFlags::TRANSFER_READ, vk::AccessFlags::TRANSFER_WRITE),
            (vk::PipelineStageFlags::TRANSFER,  vk::PipelineStageFlags::TRANSFER),
        );
        command_buffer.copy_image(
            &self.device, 
            (self.raytrace_output_image.image(), vk::ImageLayout::TRANSFER_SRC_OPTIMAL),
            (present_image,  vk::ImageLayout::TRANSFER_DST_OPTIMAL),
            vk::Extent3D { width: self.window.inner_size().width, height: self.window.inner_size().height, depth: 1 },
        );
        command_buffer.pipeline_barrier(
            &self.device, 
            present_image, 
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::PRESENT_SRC_KHR),
            (vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::TRANSFER_READ),
            (vk::PipelineStageFlags::TRANSFER,  vk::PipelineStageFlags::TRANSFER),
        );
        command_buffer.end(&self.device);

        fence.reset(&self.device);

        command_buffer.submit_commands(
            &self.device, 
            semaphore.image_available(), 
            semaphore.render_complete(), 
            fence,
        );

        self.swapchain.present_frame(
            &self.device, 
            image_index as u32, 
            semaphore.render_complete(), 
        );
    }

    pub fn run(mut self) {
        let mut mouse = Mouse::default();
        let mut camera = Camera::new(
            Vector3::new(12.0, 0.0, 11.0), // Look from
            Vector3::new(12.0, 12.0, 9.0)  // Look at
        );

        let event_loop = self.event_loop.take().unwrap();
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::MainEventsCleared => self.render(camera.projection(), mouse.state()),
                Event::WindowEvent {
                    event, 
                    ..
                } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::MouseInput { 
                        button,
                        state,
                        ..
                    } => mouse.process_input(button, state),
                    WindowEvent::CursorMoved {
                        position,
                        ..
                    } => mouse.process_movement(position, &self.window),
                    WindowEvent::KeyboardInput  {
                        input: KeyboardInput {
                            virtual_keycode: Some(key),
                            state,
                            ..
                        },
                        ..
                    } => match (key, state) {
                        (VirtualKeyCode::Escape, ElementState::Pressed) => *control_flow = ControlFlow::Exit,
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
            self.device.device_wait_idle().unwrap();

            self.fences.destroy_fences(&self.device);
            self.semaphores.destroy_semaphore(&self.device);
            self.raytrace_output_image.destroy_image(&self.device);
            self.world_buffer.destroy_buffer(&self.device);
            self.cursor_buffer.destroy_buffer(&self.device);
            self.raytrace_pipeline.destroy_pipeline(&self.device);
            self.mouse_raytrace_pipeline.destroy_pipeline(&self.device);
            self.edit_pipeline.destroy_pipeline(&self.device);
            self.descriptor_set.destroy_descriptor_set(&self.device);
            self.command_buffers.destroy_command_buffer(&self.device);
            self.swapchain.destroy_swapchain(&self.device);
            self.device.destroy_device(None);
            self.surface.destroy_surface();
            self.debug_messenger.destroy_debug_messenger();
            self.instance.destroy_instance(None);
        }
    }
}