use opsis_filament::Backend;

fn main() {
    if let Err(error) = opsis_effects::serve(Backend::Vulkan) {
        eprintln!("opsis-renderer: {error}");
        std::process::exit(1);
    }
}
