use core::marker::PhantomData;
use log::error;
use crate::backend::RequestAuthenticated;
//--------------------------------------------------------------------------------------------------



pub struct ExtractCurrentUser <
    Usr: axum_login::AuthUser,
    Backend: axum_login::AuthnBackend<User = Usr> + RequestAuthenticated + 'static,
> {
    pub user: Usr,
    _pd: PhantomData<Backend>,
}


#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + Clone + 'static,
    Backend: axum_login::AuthnBackend<User = Usr> + RequestAuthenticated + 'static,
    S: Send + Sync,
> axum::extract::FromRequestParts<S> for ExtractCurrentUser<Usr, Backend> {
    type Rejection = (http::StatusCode, &'static str);
    async fn from_request_parts(parts: &mut http::request::Parts, _state: &S)
        -> Result<Self, Self::Rejection> {
        extract_current_user_from_request_parts::<Usr, Backend, S>(parts).await
    }
}


async fn extract_current_user_from_request_parts <
    Usr: axum_login::AuthUser + 'static,
    Backend: axum_login::AuthnBackend<User = Usr> + RequestAuthenticated + 'static,
    S: Send + Sync,
> (parts: &mut http::request::Parts)
    -> Result<ExtractCurrentUser<Usr, Backend>, (http::StatusCode, &'static str)> {

    let user: Option<&Usr> = parts.extensions.get();
    if let Some(user) = user {
        return Ok(ExtractCurrentUser::<Usr, Backend> { user: user.clone(), _pd: PhantomData })
    }

    let auth_session: Option<&axum_login::AuthSession<Backend>> = parts.extensions.get();

    let user_res: Result<Usr, &'static str> = match auth_session {
        None => {
            Err("Error of getting auth session. Probably axum_login is not set up properly.")
        },
        Some(auth_session) => {
            let user = &auth_session.user;
            match user {
                Some(ref user) => {
                    Ok(user.clone())
                }
                None => {
                    // T O D O: how to avoid these cloning?
                    let auth_session = (*auth_session).clone();
                    let backend = auth_session.backend.clone();

                    let res = backend.do_authenticate_request_parts::<Backend, S>(
                        auth_session, parts).await;
                    match res {
                        Err(ref _err) =>
                            Err("Error of getting auth session"),
                        Ok(None) =>
                            Err("Error of getting auth user"),
                        Ok(Some(user)) =>
                            Ok(user.clone()),
                    }
                }
            }
        }
    };

    match user_res {
        Ok(user) => {
            parts.extensions.insert(user.clone());
            Ok(ExtractCurrentUser::<Usr, Backend> { user, _pd: PhantomData })
        }
        Err(err_msg) => {
            error!("{err_msg}");

            let response =
                if cfg!(debug_assertions) { err_msg }
                else { "Internal error. See logs." };
            Err( (http::StatusCode::INTERNAL_SERVER_ERROR, response) )
        }
    }
}
