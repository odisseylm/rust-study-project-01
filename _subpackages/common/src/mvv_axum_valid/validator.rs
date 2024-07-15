use core::fmt;
use core::fmt::Display;
use core::ops::{ Deref, DerefMut };
use axum::extract::{ FromRequest, FromRequestParts, Request };
use axum_valid::{ HasValidate, ValidRejection };
use http::request::Parts;
use log::error;
use validator::Validate;
// use crate::http::log::{ ConnectionInfoRef };
//--------------------------------------------------------------------------------------------------



#[inline]
#[allow(dead_code)]
pub fn regex_validate(s: &str, regex: &regex::Regex) -> Result<(), validator::ValidationError> {
    if regex.is_match(s) { Ok(()) }
    else { Err(validator::ValidationError::new("regex")) }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Valid<E>(pub E);
impl<E> Deref for Valid<E> {
    type Target = E;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<E> DerefMut for Valid<E> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
impl<T: Display> Display for Valid<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl<E> Valid<E> {
    #[allow(dead_code)]
    /// Consume the `Valid` extractor and returns the inner type.
    pub fn into_inner(self) -> E { self.0 }
}


#[async_trait::async_trait]
impl<State, Extractor> FromRequest<State> for Valid<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidate + FromRequest<State> + Send + Sync,
    Extractor::Validate: Validate,
{
    type Rejection = ValidRejection<<Extractor as FromRequest<State>>::Rejection>;

    async fn from_request(req: Request, state: &State) -> Result<Self, Self::Rejection> {
        // optional: tracing can show connection info (without user) automatically
        // let connect_info = ConnectionInfo::from_request(&req);
        // let connect_info = ConnectionInfoRef::from_request(&req);

        let inner = Extractor::from_request(req, state)
            .await
            .map_err(ValidRejection::Inner)?;
        let v_res = inner.get_validate().validate();

        if v_res.is_err() {
            // error!("### ValidationError (1) [{connect_info:?}] : {v_res:?}");
            error!("### ValidationError (1) : {v_res:?}");
        }

        v_res ?;
        Ok(Valid(inner))
    }
}

#[async_trait::async_trait]
impl<State, Extractor> FromRequestParts<State> for Valid<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidate + FromRequestParts<State> + Send + Sync,
    Extractor::Validate: Validate,
{
    type Rejection = ValidRejection<<Extractor as FromRequestParts<State>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let inner = Extractor::from_request_parts(parts, state)
            .await
            .map_err(ValidRejection::Inner)?;
        let v_res = inner.get_validate().validate();
        if v_res.is_err() {
            // optional: tracing can show connection info (without user) automatically
            // let connect_info = ConnectionInfoRef::from_request_parts(parts);
            // error!("### ValidationError (2) [{connect_info:?}] : {v_res:?}");
            // TODO: fix getting user
            error!("### ValidationError (2) : {v_res:?}");
        }
        v_res?;
        Ok(Valid(inner))
    }
}
