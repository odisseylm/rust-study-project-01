
mod client_search_service;
mod client;
mod dependencies;
mod schema;
pub mod app;
pub mod cfg;
pub mod grpc;
mod health_check;
mod stream_static_wrapper;
mod auth;
// tonic::include_proto!("../proto/mvv_client_search");
// tonic::include_proto!("/home/volodymyr/projects/rust/rust-study-project-01/client_search_grpc_soa/proto/mvv.client.search.proto");
// tonic::include_proto!("mvv_client_search_descriptor.bin");

// pub mod solar_system_info {
//     tonic::include_proto!("mvv_client_search");
// }
