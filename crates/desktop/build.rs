const APP_CAPABILITIES: &str = "./capabilities/*.json";
const WITH_E2E_DRIVER: &str = "./capabilities/**/*.json";

fn main() {
    let pattern = if std::env::var_os("CARGO_FEATURE_E2E").is_some() {
        WITH_E2E_DRIVER
    } else {
        APP_CAPABILITIES
    };
    tauri_build::try_build(tauri_build::Attributes::new().capabilities_path_pattern(pattern))
        .expect("the Tauri build step");
}
