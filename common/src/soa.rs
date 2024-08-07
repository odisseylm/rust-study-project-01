use core::fmt;
use crate::backtrace::{ backtrace, BacktraceCell };
use crate::string::FormatMode;
//--------------------------------------------------------------------------------------------------


pub fn improve_prog_err(error: progenitor_client::Error) -> RestCallError {
    RestCallError::ProgenitorError(error, backtrace())
}

#[derive(educe::Educe)] #[educe(Debug)]
#[derive(
    thiserror::Error,
    mvv_error_macro::ThisErrorFromWithBacktrace,
    mvv_error_macro::ThisErrorBacktraceSource,
)]
pub enum RestCallError {
    #[error("ProgenitorError {0}")]
    ProgenitorError(
        #[source] #[from_with_bt]
        #[educe(Debug(method(progenitor_err_dbg_fmt)))]
        progenitor_client::Error,
        BacktraceCell),
}


fn progenitor_err_dbg_fmt(error: &progenitor_client::Error, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    format_progenitor_err(f, FormatMode::Debug, error, None) // Some(backtrace))
}

fn format_progenitor_err(
    f: &mut fmt::Formatter<'_>, format_mode: FormatMode,
    err: &progenitor_client::Error, backtrace: Option<&BacktraceCell>) -> fmt::Result {

    let status = err.status();
    let status = status.as_ref().map(|s|s.as_str()).unwrap_or("");
    let url: &str = err_url(err).unwrap_or("");

    match format_mode {
        FormatMode::Display =>
            write!(f, "REST client error:\n \
             {url}  status={status} \n \
             {err}"),
        FormatMode::Debug => {
            match backtrace {
                None =>
                    write!(f, "REST client error:\n \
                     {url}  status={status} \n \
                     {err:?}"),
                Some(ref backtrace) =>
                    write!(f, "REST client error:\n \
                     {url}  status={status} \n \
                     {err:?} \n{backtrace}"),
            }
        }
    }
}



fn err_url(err: &progenitor_client::Error) -> Option<&str> {
    use progenitor_client::Error;
    match err {
        Error::InvalidRequest(ref err) => Some(err.as_str()),
        Error::CommunicationError(ref err) => err.url().map(|url|url.as_str()),
        Error::InvalidUpgrade(ref err) => err.url().map(|url|url.as_str()),
        Error::ErrorResponse(ref _err) => None,
        Error::ResponseBodyError(ref err) => err.url().map(|url|url.as_str()),
        Error::InvalidResponsePayload(_, _) => None,
        Error::UnexpectedResponse(ref err) => Some(err.url().as_str()),
        Error::PreHookError(ref _err) => None,
    }
}
