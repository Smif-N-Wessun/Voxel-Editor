use ash::{ 
    vk,
    extensions::khr,
};
use super::{
    Device,
    Surface, 
    Semaphores,
};
use std::slice;

const MIN_IMAGE_COUNT: u32 = 3;

pub struct Swapchain {
    swapchain: vk::SwapchainKHR,
    loader: khr::Swapchain,
    present_images: Vec<vk::Image>,
    present_image_views: Vec<vk::ImageView>,
    image_index: usize,
}

impl Swapchain {
    pub fn new(instance: &ash::Instance, device: &Device, surface: &Surface) -> Self {
        let loader = khr::Swapchain::new(&instance, &device);
        let swapchain = unsafe {
            let format = surface
                .get_physical_device_surface_formats(device)
                .unwrap()
                .into_iter()
                .find(|format| format.format == vk::Format::B8G8R8A8_SRGB)
                .unwrap();
            
            let extent = surface
                .get_physical_device_surface_capabilities(device)
                .unwrap()
                .current_extent;
            
            let image_usage = vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::COLOR_ATTACHMENT;
            let queue_family_index = &device.queue_family_index();
            
            let create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(surface.surface())
                .min_image_count(MIN_IMAGE_COUNT)
                .image_format(format.format)
                .image_color_space(format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(image_usage)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .queue_family_indices(slice::from_ref(queue_family_index))
                .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(vk::PresentModeKHR::IMMEDIATE)
                .clipped(true)
                .old_swapchain(vk::SwapchainKHR::null());

            loader.create_swapchain(&create_info, None).expect("Swapchain creation error") 
        };
        let present_images = unsafe { loader.get_swapchain_images(swapchain).unwrap() };

        let present_image_views = unsafe {
            let components = vk::ComponentMapping::builder()
                .r(vk::ComponentSwizzle::IDENTITY)
                .g(vk::ComponentSwizzle::IDENTITY)
                .b(vk::ComponentSwizzle::IDENTITY)
                .a(vk::ComponentSwizzle::IDENTITY)
                .build();

            let subresourse_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .build();

            present_images
                .iter()
                .map(|i| {
                    let info = vk::ImageViewCreateInfo::builder()
                        .image(*i)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(vk::Format::B8G8R8A8_SRGB)
                        .components(components)
                        .subresource_range(subresourse_range);

                        device.create_image_view(&info, None).unwrap()
                    })
                .collect::<Vec<_>>()
        };

        Self { 
            swapchain, 
            loader, 
            present_images, 
            present_image_views,
            image_index: 0,
        }
    }

    pub fn destroy_swapchain(&self, device: &Device) {
        unsafe { 
            self.present_image_views.iter().for_each(|image_view| device.destroy_image_view(*image_view, None));
            self.loader.destroy_swapchain(self.swapchain, None) 
        }
    }
    
    pub fn acquire_next_image(&mut self, semaphores: &Semaphores) -> (vk::Image, usize) {
        let semaphore = semaphores[self.image_index].image_available();

        let image_index = unsafe { 
            self.loader.acquire_next_image(
                self.swapchain, 
                u64::max_value(), 
                semaphore, 
                vk::Fence::null()
            ).unwrap().0 as usize
        };

        self.image_index = (image_index + 1) % self.present_images.len();

        (self.present_images[image_index], image_index)
    }

    pub fn present_frame(&self, device: &Device, image_index: u32, semaphore: vk::Semaphore) {
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(slice::from_ref(&semaphore))
            .swapchains(slice::from_ref(&self.swapchain))
            .image_indices(slice::from_ref(&image_index));

        unsafe { self.loader.queue_present(device.queue(), &present_info).unwrap(); }
    }

    pub fn present_images(&self) -> &Vec<vk::Image> {
        &self.present_images
    }
}