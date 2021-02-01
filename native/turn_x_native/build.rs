// Escape to C/C++ for some code initialization. Rationales will be listed in
// the C/C++ files.
use pkg_config;
extern crate cc;

fn main() {
    let include = format!(
        "-I{}",
        pkg_config::get_variable("openh264", "includedir").unwrap()
    );
    let lib = format!(
        "-L{}",
        pkg_config::get_variable("openh264", "libdir").unwrap()
    );
    cc::Build::new()
        .cpp(true)
        .include(include)
        .file("src/cxx/turnx_h264_cxxcalls.cpp")
        .flag(&lib)
        .flag("-lopenh264")
        .shared_flag(true)
        .compile("turnx_h264_cxxcalls.so");
}
