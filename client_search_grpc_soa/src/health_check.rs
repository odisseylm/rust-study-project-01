use std::pin::Pin;
// use std::sync::Arc;
// use std::task::{Context, Poll};
// use async_stream::__private::AsyncStream;
// use futures_util::StreamExt;
// use tokio::sync::{mpsc, watch};
use tokio_stream::Stream;
use tonic::{Request, Response, Status};
use crate::generated::grpc_health_v1::{HealthCheckRequest, HealthCheckResponse};
use crate::generated::grpc_health_v1::health_check_response::ServingStatus;
//--------------------------------------------------------------------------------------------------


pub struct HealthCheckService;

// use tonic::{transport::server::ServiceName};
// type Stream22 = VecDeque<Result<HealthCheckResponse, Status>>;

#[tonic::async_trait]
impl crate::generated::grpc_health_v1::health_server::Health for HealthCheckService {

    async fn check(&self, _request: Request<HealthCheckRequest>) -> Result<Response<HealthCheckResponse>, Status> {
        use crate::generated::grpc_health_v1::HealthCheckResponse;
        use crate::generated::grpc_health_v1::health_check_response::ServingStatus;

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
// pin_project_lite::pin_project! {
    pub struct Temp345 {
        // data: Box<dyn Stream<Item = Result<HealthCheckResponse, Status>>>,
        //#[pin]
        data: tonic::codegen::BoxStream<HealthCheckResponse>,
        // data: Box<dyn Stream<Item = Result<HealthCheckResponse, Status>>>,
        // data: Pin<Box<dyn Stream<Item = Result<HealthCheckResponse, Status>>>>,
    }
// }
impl Stream for Temp345 {
    type Item = Result<HealthCheckResponse, Status>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        t o d o!()
        // self.data.poll_next(cx)
        // self.data.poll_next_unpin(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        t o d o!()
        // self.data.size_hint()
        // (0, None)
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
