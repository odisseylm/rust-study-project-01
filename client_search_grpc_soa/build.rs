use std::env;
use std::path::PathBuf;
//--------------------------------------------------------------------------------------------------



fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic_build::compile_protos("proto/mvv_client_search.proto") ?;

    // let proto_file = "./proto/mvv_client_search.proto";
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    /*
    let mut config: prost_build::Config = prost_build::Config::default();
    // config.disable_comments(&["."]);
    config.disable_comments(&[".mvv_client_search_api_v1.ErrorInfo"]);
    config.out_dir("./src/generated");
    // config.build_server(true);
    config.file_descriptor_set_path(out_dir.join("mvv_client_search_descriptor.bin"));
    config.format(true);
    // config.skip_debug()
    */

    tonic_build::configure()
        // .protoc_arg("--experimental_allow_proto3_optional") // for older systems
        .build_client(false)
        .build_server(true)
        .emit_rerun_if_changed(true)
        // .type_attribute()
        // NEEDED for reflection
        .file_descriptor_set_path(out_dir.join("mvv_client_search_descriptor.bin"))
        .out_dir("./src/generated")
        // .client_mod_attribute("./src/mvv_client_search_api_v1.rs", )
        // .type_attribute("proto/google/type/date.proto", )
        //.compile_well_known_types(true)
        // .disable_package_emission()
        // .compile(&[proto_file], &["proto/mvv_client_search.proto"])?;
        // .disable_comments("proto/mvv_client_search.proto")

        // Seems it does not work !?
        // Now it is disabled by feature "cleanup-markdown"
        // .disable_comments("mvv_client_search_api_v1.ErrorInfo")
        // .disable_comments(".mvv_client_search_api_v1.ErrorInfo")

        // .compile(&[proto_file], &["proto"])?;
        .compile(&[
            "./proto/mvv_client_search.proto",
            "./proto/health/v1/health.proto",
        ], &["proto"])?;
        // .compile_with_config(config, &[proto_file], &["proto"])?;

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
