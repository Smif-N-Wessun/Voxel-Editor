use ash::{ 
    vk, 
    extensions::ext::DebugUtils 
};
use std::{ 
    borrow::Cow, 
    ffi::CStr
};

pub struct  DebugMessenger {
    loader: DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugMessenger {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        let loader = DebugUtils::new(&entry, &instance);
    
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));
    
        let messenger = unsafe {
            loader.create_debug_utils_messenger(&debug_info, None).expect("Debug messenger creation error")
        };

        Self { 
            loader, 
            messenger,
        }
    }

    pub fn destroy_debug_messenger(&self) {
        unsafe { self.loader.destroy_debug_utils_messenger(self.messenger, None) };
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );

    vk::FALSE
}