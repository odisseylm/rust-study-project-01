use std::collections::HashMap;
use std::pin::Pin;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};
use mvv_auth::{
    grpc::server::public_access_permissions,
    permission::PermissionSet,
};
use mvv_common::string::StaticRefOrString;
use crate::grpc::health::v1::{
    health_server::Health,
    HealthCheckRequest, HealthCheckResponse,
    health_check_response::ServingStatus,
};
//--------------------------------------------------------------------------------------------------


pub struct HealthCheckService;

impl HealthCheckService {
    #[inline]
    #[allow(dead_code)]
    pub fn endpoints_roles <PermSet: PermissionSet>() -> HashMap<StaticRefOrString, PermSet> {
        HashMap::from([("/grpc.health.v1.Health".into(), public_access_permissions())])
    }
}


#[tonic::async_trait]
impl Health for HealthCheckService {

    async fn check(&self, _request: Request<HealthCheckRequest>) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse { status: ServingStatus::Serving as i32 }))
    }

    // type WatchStream = tonic::codegen::BoxStream<HealthCheckResponse>;
    type WatchStream = Pin<Box<dyn Stream<Item = Result<HealthCheckResponse, Status>> + Send + 'static>>;
    // type WatchStream = Box<dyn Stream<Item = Result<HealthCheckResponse, Status>> + Send + 'static>;
    // type WatchStream = AsyncStream<Result<HealthCheckResponse, Status>, ()>;

    async fn watch(&self, _request: Request<HealthCheckRequest>) -> Result<Response<Self::WatchStream>, Status> {
        // t o d o!("HealthCheckService::watch() is not implemented")

        // let service = _request.get_ref().service.as_str();
        let service_statuses = async_stream::stream! {
            for _ in 0..3 {
                yield Ok(HealthCheckResponse { status: ServingStatus::Serving as i32 });
            }
        };

        Ok(Response::new(Box::pin(service_statuses)))

        // Err(Status::unimplemented("HealthCheckService.watch() is not implemented."))
    }
}

/*
    async fn watch(&self, request: Request<HealthCheckRequest>) -> HealthResult<Self::WatchStream> {
        let name = &request.get_ref().service;
        let (mut tx, res_rx) = mpsc::channel(10);

        if let Some(status) = self.get_status(name) {
            let _ = tx.send(Ok(HealthCheckResponse {
                status: status as i32,
            }))
            .await;
            let mut rx = self.subscriber.clone();
            while let Some(value) = rx.recv().await {
                match value {
                    Some((changed_name, status)) => {
                        if *name == changed_name {
                            let _ = tx.send(Ok(HealthCheckResponse {
                                status: status as i32,
                            }))
                            .await;
                        }
                    },
                    _ => {},
                }
            }
            Ok(Response::new(res_rx))
        } else {
            Err(Status::new(Code::NotFound, ""))
        }
    }
*/


/*
fn aaa() {

    let aa = async_stream::stream! {
        for i in 0..3 {
            yield i;
        }
    };
}
*/
