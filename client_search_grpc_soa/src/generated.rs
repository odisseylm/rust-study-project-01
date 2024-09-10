
#[path = ""]
pub mod google {
    #[path = "generated/google.r#type.rs"]
    pub mod r#type;
}
#[path = ""]
pub mod grpc {
    #[path = ""]
    pub mod health {
        #[path = "generated/grpc.health.v1.rs"]
        pub mod v1;
    }
}
#[path = ""]
pub mod mvv {
    #[path = ""]
    pub mod roles {
        #[path = "generated/mvv.roles.v1.rs"]
        pub mod v1;
    }
    // client.search.api.v1
    #[path = ""]
    pub mod client {
        #[path = ""]
        pub mod search {
            #[path = ""]
            pub mod api {
                #[path = "generated/mvv.client.search.api.v1.rs"]
                pub mod v1;
            }
        }
    }
}
