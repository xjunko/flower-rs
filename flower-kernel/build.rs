fn main() {
    println!("cargo:rustc-link-arg=-Tflower-kernel/link.ld");
    println!("cargo:rerun-if-changed=flower-kernel/link.ld");
}
