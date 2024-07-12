
pub mod validator {
    use core::fmt;
    use core::fmt::Display;
    use core::ops::{ Deref, DerefMut };
    use std::error::Error;
    use askama_axum::{IntoResponse, Response};
    use axum::extract::{FromRequest, FromRequestParts, Request};
    use axum_valid::{ HasValidate, VALIDATION_ERROR_STATUS };
    use http::request::Parts;
    use log::error;
    use validator::Validate;
    use crate::rest::log::{ ConnectionInfoRef };


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

    #[derive(Debug)]
    pub enum ValidationRejection<V, E> {
        /// `Valid` variant captures errors related to the validation logic.
        Valid(V),
        /// `Inner` variant represents potential errors that might occur within the inner extractor.
        Inner(E),
    }

    pub type ValidRejection<E> = ValidationRejection<validator::ValidationErrors, E>;

    impl<E> From<validator::ValidationErrors> for ValidRejection<E> {
        fn from(value: validator::ValidationErrors) -> Self {
            Self::Valid(value)
        }
    }

    impl<V: Display, E: Display> Display for ValidationRejection<V, E> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                ValidationRejection::Valid(errors) => write!(f, "{errors}"),
                ValidationRejection::Inner(error) => write!(f, "{error}"),
            }
        }
    }

    impl<V: Error + 'static, E: Error + 'static> Error for ValidationRejection<V, E> {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                ValidationRejection::Valid(ve) => Some(ve),
                ValidationRejection::Inner(e) => Some(e),
            }
        }
    }

    impl<V: Display, E: IntoResponse> IntoResponse for ValidationRejection<V, E> {
        fn into_response(self) -> Response {
            match self {
                ValidationRejection::Valid(v) => {
                    // error!("### error 567, ValidationRejection::into_response() : {}", v);
                    (VALIDATION_ERROR_STATUS, v.to_string()).into_response()
                }
                ValidationRejection::Inner(e) => {
                    // error!("### error 568, ValidationRejection::into_response() ");
                    e.into_response()
                },
            }
        }
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
                let connect_info = ConnectionInfoRef::from_request_parts(parts);
                error!("### ValidationError (2) [{connect_info:?}] : {v_res:?}");
            }
            v_res?;
            Ok(Valid(inner))
        }
    }
}
