pub fn setup() {
    #[cfg(target_os = "macos")]
    set_molten_vk();
}

#[cfg(target_os = "macos")]
fn set_molten_vk() {
    use crate::logs::*;
    use std::env;

    let exe_dir = env::current_exe().expect_log("failed to get the execution file path");
    let exe_dir = exe_dir
        .parent()
        .expect_log("failed to get the execution directory path");
    unsafe {
        env::set_var("VK_ICD_FILENAMES", exe_dir);
        env::set_var("MVK_CONFIG_LOG_LEVEL", "0");
    }
}
