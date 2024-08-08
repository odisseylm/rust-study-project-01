use core::future::Future;
// use std::fmt::{Debug, Display};
use axum::Json;
use extension_trait::extension_trait;
use crate::backtrace::{backtrace, BacktraceCell, BacktraceSource};
//--------------------------------------------------------------------------------------------------



#[extension_trait]
pub impl <T, Fut, Err> RestFutureToJson<T,Err>
for Fut where Fut: Future<Output = Result<T, Err>> {
    fn to_json(self) -> impl Future<Output = Result<Json<T>, Err>> {
        async { self.await.map(|data|
            Json(data)) // at separate line for breakpoint :-)
        }
    }
}


/// It is general error wrapper to have easy possibility to
/// treat DTO parsing errors as HTTP BadRequest or InternalError.
#[derive(
    Debug,
    thiserror::Error,
    mvv_error_macro::ThisErrorFromWithBacktrace,
    mvv_error_macro::ThisErrorBacktraceSource,
)]
pub enum InvalidInputError {
    #[error("InvalidInputError (anyhow) {{ {0} }}")]
    AsAnyhow(anyhow::Error),
    #[error("InvalidInputError (std) {{ {0} }}")]
    AsStdError(Box<dyn std::error::Error>, BacktraceCell),
}


#[extension_trait::extension_trait]
pub impl<V: 'static, E: 'static> AsBadReqErrExt<V,E> for Result<V,E> {
    fn err_to_anyhow_bad_req(self) -> Result<V, InvalidInputError>
        where E: Into<anyhow::Error>
    {
        self.map_err(|err|{
            let as_anyhow: anyhow::Error = err.into();
            InvalidInputError::AsAnyhow(as_anyhow)
        })
    }
    fn err_to_std_err_bad_req(self) -> Result<V, InvalidInputError>
        where E: std::error::Error
    {
        self.map_err(|err| InvalidInputError::AsStdError(Box::new(err), backtrace()))
    }
    fn err_to_bad_req(self) -> Result<V, InvalidInputError>
        where E: std::error::Error + BacktraceSource
    {
        self.map_err(|err| {
            let bt = BacktraceCell::inherit_or_capture(&err);
            InvalidInputError::AsStdError(Box::new(err), bt)
        })
    }
}
