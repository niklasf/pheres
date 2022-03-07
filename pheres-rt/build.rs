use std::env;

fn main() {
    cbindgen::Builder::new()
        .with_crate(env::var("CARGO_MANIFEST_DIR").unwrap())
        .generate()
        .expect("generate bindings")
        .write_to_file("pheres_rt.h");
}
