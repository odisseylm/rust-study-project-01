use core::fmt::{ self, Debug, Display };
use crate::backtrace::BacktraceCell;
use crate::string::FormatMode;
//--------------------------------------------------------------------------------------------------


pub fn improve_prog_err(error: progenitor_client::Error) -> RestCallError {
    // RestCallError::ProgenitorError { error, backtrace: BacktraceInfo::capture() }
    RestCallError::ProgenitorError { error, backtrace: BacktraceCell::capture_backtrace() }
}

pub enum RestCallError {
    ProgenitorError {
        error: progenitor_client::Error,
        backtrace: BacktraceCell,
    },
}


impl std::error::Error for RestCallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RestCallError::ProgenitorError { ref error, .. } => error.source()
        }
    }
}


impl From<progenitor_client::Error> for RestCallError {
    fn from(error: progenitor_client::Error) -> Self {
        // RestCallError::ProgenitorError { error, backtrace: BacktraceInfo::capture() }
        RestCallError::ProgenitorError { error, backtrace: BacktraceCell::capture_backtrace() }
    }
}

impl Display for RestCallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RestCallError::ProgenitorError { ref error, .. } =>
                format_progenitor_err(f, FormatMode::Display, error, None)
        }
    }
}
impl Debug for RestCallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RestCallError::ProgenitorError { ref error, ref backtrace } =>
                format_progenitor_err(f, FormatMode::Debug, error, Some(backtrace))
        }
    }
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
