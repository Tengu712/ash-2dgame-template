use ash::{Entry, Instance, vk};

fn create_instance(entry: &Entry) -> Instance {
    let ai = vk::ApplicationInfo::default()
        .application_name(c"ash-2dgame-template")
        .application_version(0)
        .api_version(vk::make_api_version(0, 1, 0, 0));
    let ci = vk::InstanceCreateInfo::default().application_info(&ai);
    unsafe {
        entry
            .create_instance(&ci, None)
            .expect("Instance creation error")
    }
}

fn main() {
    let entry = unsafe { Entry::load() }.expect("Failed to load Vulkan loader");
    let instance = create_instance(&entry);
    unsafe { instance.destroy_instance(None) };
}
