pub fn compile_shaders(glslang_bin_path: &str) {
    compile_a_shader(glslang_bin_path, "shader/basic.frag");
    compile_a_shader(glslang_bin_path, "shader/basic.vert");
}

fn compile_a_shader(glslang_bin_path: &str, file_path: &str) {
    super::run(
        &format!("{glslang_bin_path}/glslangValidator"),
        &["-V", file_path, "-o", &format!("{file_path}.spv")],
    );
    println!("cargo:rerun-if-changed={file_path}");
}
