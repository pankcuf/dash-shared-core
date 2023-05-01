extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut config = cbindgen::Config::from_file("./cbindgen.toml").expect("Error config");
    // Here we write down crate names (!) where we want to retrieve C-bindings
    let includes = vec![/*"dash-spv-ffi".to_string(), "dash-spv-models".to_string()*/];
    config.language = cbindgen::Language::C;
    config.parse = cbindgen::ParseConfig {
        parse_deps: true,
        include: Some(includes.clone()),
        extra_bindings: includes,
        ..Default::default()
    };

    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file("target/dash_spv_masternode_processor.h");
}
