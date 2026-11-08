fn main() {
    println!("cargo::rerun-if-env-changed=OPSIS_COMMIT");
    let target = std::env::var("TARGET").unwrap_or_default();
    let commit = std::env::var("OPSIS_COMMIT").unwrap_or_default();
    println!("cargo::rustc-env=OPSIS_TARGET={target}");
    println!("cargo::rustc-env=OPSIS_COMMIT={commit}");
}
