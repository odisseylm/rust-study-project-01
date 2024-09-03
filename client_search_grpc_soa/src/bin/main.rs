// use std::{env, path::PathBuf};


fn main()  {
    // let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    //
    // tonic_build::configure()
    //     .file_descriptor_set_path(out_dir.join("mvv_client_search_descriptor.bin"))
    //     .compile(&["proto/mvv_client_search.proto"], &["/proto"])
    //     .unwrap();

    // let proto_file = "./proto/mvv_client_search.proto";
    // let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    //
    // tonic_build::configure()
    //     // .protoc_arg("--experimental_allow_proto3_optional") // for older systems
    //     .build_client(true)
    //     .build_server(true)
    //     .file_descriptor_set_path(out_dir.join("mvv_client_search_descriptor.bin"))
    //     .out_dir("./src")
    //     .compile(&[proto_file], &["proto"])?;
    //
    // Ok(())
}
