fn main() {
    println!("cargo:rustc-link-arg=-Tkernel/link.ld");
    println!("cargo:rerun-if-changed=kernel/link.ld");
}
