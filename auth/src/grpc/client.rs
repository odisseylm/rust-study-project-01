use core::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Status};
use mvv_common::cfg::DependencyConnectConf;
use mvv_common::grpc::TonicErrToStatusExt;
//--------------------------------------------------------------------------------------------------



pub struct GrpcClientAuthInterceptor<C: DependencyConnectConf> {
    pub config: Arc<C>,
}
impl<C: DependencyConnectConf> tonic::service::Interceptor for GrpcClientAuthInterceptor<C> {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let cfg = &self.config;

        let user = match cfg.user() {
            None => return Ok(request),
            Some(user) => user.as_str(),
        };
        let psw = cfg.password().as_ref()
            .map(|psw|psw.as_str()).unwrap_or("");

        use axum_extra::headers::{ Authorization, authorization::Credentials };
        let auth = Authorization::basic(user, psw);

        let as_header_v = auth.0.encode();
        let as_header_v = as_header_v.to_str()
            .to_tonic_internal_err("GrpcClientAuthInterceptor. header name error") ?;

        request.metadata_mut().insert(
            "authorization",
            tonic::metadata::MetadataValue::from_str(as_header_v)
                .to_tonic_internal_err("GrpcClientAuthInterceptor. header value error") ?
        );

        Ok(request)
    }
}
