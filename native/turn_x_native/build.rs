// Escape to C for some code initialization. Rationales will be listed in the
// C files.
use pkg_config;
extern crate cc;

fn main() {
    let include = format!(
        "-L{}",
        pkg_config::get_variable("vpx", "includedir").unwrap()
    );
    cc::Build::new()
        .file("src/c/turnx_vpx.c")
        .include(include)
        .flag("-lvpx")
        .shared_flag(true)
        .compile("libturnx_vpx.so");
}
