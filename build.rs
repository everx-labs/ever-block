use std::env;

mod common {
    include!("./common/build/build.rs");
    pub(crate) fn build() {
        main();
    }
}

fn main() {
    // Take care on wasm cross-compilation
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch.ne("wasm32") {
        println!("cargo:rustc-cfg=feature=\"std\"");
    }
    common::build();
}
