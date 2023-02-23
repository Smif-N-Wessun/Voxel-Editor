use ash::{
    vk, 
    Entry,
    Instance,
    extensions::ext::DebugUtils
};
use std::ffi::CStr;
use winit::window::Window;

pub trait InstanceTrait {
    fn new(entry: &Entry, window: &Window, app_name: &str) -> Instance;
}

impl InstanceTrait for Instance {
    fn new(entry: &Entry, window: &Window, app_name: &str) -> Instance {
        let app_name = String::from(app_name.to_owned() + "\0");
        let app_name = CStr::from_bytes_with_nul(app_name.as_bytes()).unwrap();

        let appinfo = vk::ApplicationInfo::builder()
            .application_name(app_name)
            .application_version(0)
            .engine_name(app_name)
            .engine_version(0)
            .api_version(vk::API_VERSION_1_2);

        let layer_names = vec![
            CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0").unwrap().as_ptr()
        ];

        let mut instance_extensions = ash_window::enumerate_required_extensions(window)
            .unwrap()
            .to_vec();

        instance_extensions.push(DebugUtils::name().as_ptr());

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&appinfo)
            .enabled_extension_names(&instance_extensions)
            .enabled_layer_names(&layer_names);

        unsafe { entry.create_instance(&create_info, None).expect("Instance creation error") }
    }
}