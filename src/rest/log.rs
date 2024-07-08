use core::fmt::{ self, Debug, Display };
use axum::extract::Request;
use http::{Extensions, Method, Uri};
use http::request::Parts;

use crate::rest::auth::AuthUser;
type AuthSession = axum_login::AuthSession<crate::rest::auth::AuthBackend>;
//--------------------------------------------------------------------------------------------------



fn get_req_user(ext :&Extensions) -> Option<AuthUser> {
    let auth_session: Option<AuthSession> = ext.get::<AuthSession>().cloned();
    let session_user: Option<AuthUser> = auth_session.and_then(|s|s.user.map(|u|u));
    // TODO: get non-session user too
    // let session_user: Option<AuthUser> = auth_session.and_then(|s|s.backend.do_authenticate_request());
    session_user
}


pub(crate) struct ConnectionInfoRef<'a> {
    uri: Option<&'a Uri>,
    method: Option<&'a Method>,
    user: Option<AuthUser>,
    user_ref: Option<&'a AuthUser>,
}
impl ConnectionInfoRef<'_> {
    pub(crate) fn from_request_parts(parts: &mut Parts) -> ConnectionInfoRef {
        let uri = Some(&parts.uri);
        let method = Some(&parts.method);
        let user: Option<AuthUser> = get_req_user(&parts.extensions);
        ConnectionInfoRef { uri, method, user, user_ref: None }
    }
}
impl Display for ConnectionInfoRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref method) = self.method {
            write!(f, "{method} ") ?
        }
        if let Some(ref uri) = self.uri {
            write!(f, "{uri}") ?
        }
        if let Some(ref user) = self.user_ref {
            write!(f, ", user={user:?}") ?
        }
        if let Some(ref user) = self.user {
            write!(f, ", user={user:?}") ?
        }
        Ok(())
    }
}
impl Debug for ConnectionInfoRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref method) = self.method {
            write!(f, "{method:?} ") ?
        }
        if let Some(ref uri) = self.uri {
            write!(f, "{uri:?}") ?
        }
        if let Some(ref user) = self.user_ref {
            write!(f, ", user={user:?}") ?
        }
        if let Some(ref user) = self.user {
            write!(f, ", user={user:?}") ?
        }
        Ok(())
    }
}


pub(crate) struct ConnectionInfo {
    uri: Option<Uri>,
    method: Option<Method>,
    user: Option<AuthUser>,
}
impl ConnectionInfo {
    pub(crate) fn from_request(req: &Request) -> ConnectionInfo {
        let uri = Some(req.uri().clone());  // TODO: how to avoid 'clone'?? because it is needed only in case of error, but we have to clone it in any case ???
        let method = Some(req.method().clone());
        let user: Option<AuthUser> = get_req_user(req.extensions());
        ConnectionInfo { uri, method, user, }
    }
}
impl Display for ConnectionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let as_ref = ConnectionInfoRef { uri: self.uri.as_ref(), method: self.method.as_ref(), user_ref: self.user.as_ref(), user: None };
        <ConnectionInfoRef as Display>::fmt(&as_ref, f)
    }
}
impl Debug for ConnectionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let as_ref = ConnectionInfoRef { uri: self.uri.as_ref(), method: self.method.as_ref(), user_ref: self.user.as_ref(), user: None };
        <ConnectionInfoRef as Debug>::fmt(&as_ref, f)
    }
}
