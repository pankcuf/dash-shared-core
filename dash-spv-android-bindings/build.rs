extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // Here we write down crate names (!) where we want to retrieve C-bindings
    let includes = vec![
        // "dash-spv-masternode-processor".to_string(),
    ];
    let config = cbindgen::Config {
        language: cbindgen::Language::C,
        parse: cbindgen::ParseConfig {
            parse_deps: true,
            include: Some(includes.clone()),
            extra_bindings: includes,
            ..Default::default()
        },
        enumeration: cbindgen::EnumConfig {
            prefix_with_name: true,
            ..Default::default()
        },
        braces: cbindgen::Braces::SameLine,
        line_length: 80,
        tab_width: 4,
        // cpp_compat: false,
        documentation_style: cbindgen::DocumentationStyle::C,
        include_guard: Some("dash_shared_core_h".to_string()),
        ..Default::default()
    };
    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file("../target/dash_shared_core.h");
}
