use std::path::Path;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(theater_film_tests)");
    println!("cargo:rerun-if-changed=tests/fixtures/theater");
    if Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/theater")
        .is_dir()
    {
        println!("cargo:rustc-cfg=theater_film_tests");
    }
}
