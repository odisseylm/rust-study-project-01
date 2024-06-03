
// Example custom build script.
fn main() {
    // panic!("### Test error from build.rs.");

    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("### Hello from build.rs");

    println!("cargo::rustc-cfg=ossl102");

    // serde_json_raw_value
    println!(r#"cargo::rustc-cfg=feature = "serde_json_raw_value""#);
    // println!("cargo::rerun-if-changed=src/hello.c");
    // // Use the `cc` crate to build a C file and statically link it.
    // cc::Build::new()
    //     .file("src/hello.c")
    //     .compile("hello");
}
