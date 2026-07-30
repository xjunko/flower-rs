fn main() {
    let linker = format!("{}/x86_64.ld", env!("CARGO_MANIFEST_DIR"));
    println!("cargo::rerun-if-changed={linker}");
    println!("cargo:rustc-link-arg=-T{linker}");
    println!("cargo:rerun-if-changed={linker}");
}
