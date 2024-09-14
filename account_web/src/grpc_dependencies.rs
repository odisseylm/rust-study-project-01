

#[path = ""]
pub mod google {
    #[path = "grpc_dependencies/google.r#type.rs"]
    pub mod r#type;
}

// #[path = ""]
// pub mod health {
//     #[path = "grpc_dependencies/grpc.health.v1.rs"]
//     pub mod v1;
// }

#[path = ""]
pub mod mvv {
    #[path = ""]
    pub mod roles {
        #[path = "grpc_dependencies/mvv.roles.v1.rs"]
        pub mod v1;
    }
    // client.search.api.v1
    #[path = ""]
    pub mod client {
        #[path = ""]
        pub mod search {
            #[path = ""]
            pub mod api {
                #[path = "grpc_dependencies/mvv.client.search.api.v1.rs"]
                pub mod v1;
            }
        }
    }
}
