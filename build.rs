
// Example custom build script.
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    // println!("### Hello from build.rs");

    let cmd = cargo_metadata::MetadataCommand::new();

    let cargo_metadata_res = cmd.exec();
    let resolve1 = cargo_metadata_res.expect("Cargo-metadata failed.")
        .resolve.expect("Problem with Cargo-metadata 'resolve' section.");
    let serde_json_opt = resolve1.nodes.iter()
        .find(|el| el.id.repr.contains("crates.io-index#serde_json@"));

    let raw_value_is_used = serde_json_opt.and_then(|serde_json|serde_json.features
        .iter().find(|f| f.as_str() == "raw_value"))
        .is_some();

    if raw_value_is_used {
        println!(r#"cargo::rustc-cfg=feature = "serde_json_raw_value""#);
    }

    // println!("cargo::rerun-if-changed=src/hello.c");
    // // Use the `cc` crate to build a C file and statically link it.
    // cc::Build::new()
    //     .file("src/hello.c")
    //     .compile("hello");
}
