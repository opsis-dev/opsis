#![expect(
    unsafe_code,
    reason = "the cxx bridge expands to the calls across the C++ boundary, which cxx checks"
)]

mod bridge;
mod graph;
mod inspector;
mod look;
mod session;

fn main() {
    std::process::exit(bridge::ffi::run_app());
}
