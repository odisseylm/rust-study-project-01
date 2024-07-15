use axum::async_trait;
use axum::extract::{ FromRequest, FromRequestParts, Request };
use axum::http::request::Parts;
use core::fmt::{ Display, Formatter };
use core::ops::{ Deref, DerefMut };
use axum_valid::{ HasValidify, PayloadExtractor, ValidifyRejection };
use log::error;
use validify::ValidifyPayload;
//--------------------------------------------------------------------------------------------------



#[derive(Debug, Clone, Copy, Default)]
pub struct Validified<E>(pub E);
impl<E> Deref for Validified<E> {
    type Target = E;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<E> DerefMut for Validified<E> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
impl<T: Display> Display for Validified<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { self.0.fmt(f) }
}
impl<E> Validified<E> {
    #[allow(dead_code)]
    /// Consume the `Validified` extractor and returns the inner type.
    pub fn into_inner(self) -> E { self.0 }
}


#[async_trait]
impl<State, Extractor> FromRequest<State> for Validified<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidify,
    Extractor::PayloadExtractor: FromRequest<State>,
{
    type Rejection =
    ValidifyRejection<<Extractor::PayloadExtractor as FromRequest<State>>::Rejection>;

    async fn from_request(req: Request, state: &State) -> Result<Self, Self::Rejection> {
        // optional: tracing can show connection info (without user) automatically
        // let connect_info = ConnectionInfo::from_request(&req);

        let payload = Extractor::PayloadExtractor::from_request(req, state)
            .await
            .map_err(ValidifyRejection::Inner)?
            .get_payload();
        let validify_res = Extractor::Validify::validify_from(payload);

        if let Err(ref err) = validify_res {
            // error!("### ValidationError (2) [{connect_info:?}] : {err:?}");
            error!("### ValidationError (2): {err:?}");
        }

        let validify = validify_res ?;
        Ok(Validified(Extractor::from_validify(validify)))
    }
}

#[async_trait]
impl<State, Extractor> FromRequestParts<State> for Validified<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidify,
    Extractor::PayloadExtractor: FromRequestParts<State>,
{
    type Rejection =
    ValidifyRejection<<Extractor::PayloadExtractor as FromRequestParts<State>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        // optional: tracing can show connection info (without user) automatically
        // let connect_info = ConnectionInfoRef::from_request_parts(&parts);

        let payload = Extractor::PayloadExtractor::from_request_parts(parts, state)
            .await
            .map_err(ValidifyRejection::Inner)?
            .get_payload();

        let validify_res = Extractor::Validify::validify_from(payload);

        if let Err(ref err) = validify_res {
            // error!("### ValidationError (2) [{connect_info:?}] : {err:?}");
            error!("### ValidationError (2): {err:?}");
        }

        let validify = validify_res ?;
        Ok(Validified(Extractor::from_validify(validify)))
    }
}
