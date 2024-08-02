use core::fmt::{ self, Debug, Display };


pub fn improve_prog_err(err: progenitor_client::Error) -> ProgenitorErrWrapper {
    ProgenitorErrWrapper(err)
}

pub struct ProgenitorErrWrapper(pub progenitor_client::Error);

impl From<progenitor_client::Error> for ProgenitorErrWrapper {
    fn from(value: progenitor_client::Error) -> Self {
        ProgenitorErrWrapper(value)
    }
}

impl Display for ProgenitorErrWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err = &self.0;
        let status = err.status();
        let status = status.as_ref().map(|s|s.as_str()).unwrap_or("");
        let url: &str = err_url(err).unwrap_or("");
        write!(f, "REST client error:\n \
         {url}  status={status} \n \
         {err} }}")
    }
}
impl Debug for ProgenitorErrWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err = &self.0;
        let status = err.status();
        let status = status.as_ref().map(|s|s.as_str()).unwrap_or("");
        let url: &str = err_url(err).unwrap_or("");
        write!(f, "REST client error:\n \
         {url}  status={status} \n \
         {err:?} }}")
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

impl std::error::Error for ProgenitorErrWrapper {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}
