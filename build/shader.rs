use super::deps;

pub fn compile_shaders() {
    compile_a_shader("shader/basic.frag");
    compile_a_shader("shader/basic.vert");
}

fn compile_a_shader(file_path: &str) {
    super::run(
        &deps::get_glslang_path(),
        &["-V", file_path, "-o", &format!("{file_path}.spv")],
    );
    println!("cargo:rerun-if-changed={file_path}");
}
