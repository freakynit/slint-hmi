use std::{collections::HashMap, env, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let config = slint_build::CompilerConfiguration::new().with_library_paths(HashMap::from([(
        "slint-hmi".to_string(),
        manifest_dir.join("../../ui"),
    )]));
    slint_build::compile_with_config("ui/main.slint", config).unwrap();
}
