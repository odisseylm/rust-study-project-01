use core::fmt::Debug;
use log::error;
use crate::backtrace::{backtrace, BacktraceCell};



// #[derive(educe::Educe)] #[educe(Debug)]
#[derive(
    Debug,
    thiserror::Error,
    mvv_error_macro::ThisErrorFromWithBacktrace,
    mvv_error_macro::ThisErrorBacktraceSource,
)]
pub enum GrpcCallError {
    #[cfg(feature = "tonic")]
    #[error("Connection connection error {0}")]
    ConnectionError(
        #[source] #[from_with_bt]
        //#[educe(Debug(method(progenitor_err_dbg_fmt)))]
        tonic::transport::Error, BacktraceCell),
    #[cfg(feature = "tonic")]
    #[error("Grpc call error {0}")]
    CallError(
        #[source] #[from_with_bt]
        tonic::Status, BacktraceCell),
    #[error("Grpc error InvalidUri {0} ({1})")]
    InvalidUri(
        #[source] // #[from_with_bt]
        http::uri::InvalidUri, String, BacktraceCell),
    #[error("GrpcAnyhowError {0}")]
    AnyhowError(
        #[source] #[from]
        anyhow::Error),
}


impl GrpcCallError {
    pub fn invalid_uri(uri: &str, err: http::uri::InvalidUri) -> Self {
        GrpcCallError::InvalidUri(err, uri.to_owned(), backtrace())
    }
}


#[cfg(feature = "tonic")]
#[extension_trait::extension_trait]
pub impl <T,E: Debug> TonicErrToStatusExt for Result<T,E> {
    type Value = T;
    type Error = E;
    #[track_caller]
    fn to_tonic_internal_err(self, err_label: &str) -> Result<Self::Value, tonic::Status> {
        self.map_err(|err|{
            error!("{err_label}; caused by {err:?}");

            let out_err =
                if cfg!(debug_assertions) {
                    format!("{err_label}; caused by {err:?}")
                } else {
                    err_label.to_owned()
                };

            tonic::Status::internal(out_err)
        })
    }
}
