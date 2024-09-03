use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic_build::compile_protos("proto/solar-system-info/solar-system-info.proto")?;
    // tonic_build::compile_protos("proto/mvv_client_search.proto") ?;


    // tonic_build::configure()
    //     .file_descriptor_set_path(out_dir.join("mvv_client_search_descriptor.bin"))
    //     .compile(&["proto/mvv_client_search.proto"], &["/proto"])
    //     .unwrap();
    let proto_file = "./proto/mvv_client_search.proto";
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        // .protoc_arg("--experimental_allow_proto3_optional") // for older systems
        // .build_client(true)
        .build_server(true)
        .emit_rerun_if_changed(true)
        // .type_attribute()
        .file_descriptor_set_path(out_dir.join("mvv_client_search_descriptor.bin"))
        .out_dir("./src/generated")
        // .client_mod_attribute("./src/mvv_client_search_api_v1.rs", )
        // .type_attribute("proto/google/type/date.proto", )
        //.compile_well_known_types(true)
        // .disable_package_emission()
        // .compile(&[proto_file], &["proto/mvv_client_search.proto"])?;
        .compile(&[proto_file], &["proto"])?;

    /*
    tonic_build::configure()
        // .protoc_arg("--experimental_allow_proto3_optional") // for older systems
        .build_client(false)
        .build_server(false)
        .emit_rerun_if_changed(true)
        // .type_attribute()
        .file_descriptor_set_path(out_dir.join("google_types_descriptor.bin"))
        .out_dir("./src")
        // .client_mod_attribute("./src/mvv_client_search_api_v1.rs", )
        // .type_attribute("proto/google/type/date.proto", )
        //.compile_well_known_types(true)
        .disable_package_emission()
        .compile(&[proto_file], &["proto/google/type/date.proto"])?;
    */

    Ok(())
}
