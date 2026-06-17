use std::path::Path;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let link_path = Path::new(&manifest_dir).join("link.ld");
    println!("cargo:rustc-link-arg=-T{}", link_path.display());
    println!("cargo:rerun-if-changed={}", link_path.display());
}
