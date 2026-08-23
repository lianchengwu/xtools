use std::path::PathBuf;

fn main() {
    let config = slint_build::CompilerConfiguration::new().with_library_paths(
        [("xtools-ui".into(), PathBuf::from("../xtools-ui/ui"))]
            .into_iter()
            .collect(),
    );
    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}
