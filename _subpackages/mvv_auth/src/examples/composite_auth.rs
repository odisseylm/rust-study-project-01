use std::sync::Arc;

use axum::extract::Request;
use axum::response::{ IntoResponse, Response };

use super::auth_user::{ AuthUserExample };
use crate::{
    backend::{
        AuthBackendMode, AuthnBackendAttributes, ProposeAuthAction,
        ProposeHttpBasicAuthAction, ProposeLoginFormAuthAction, RequestAuthenticated,
        authz_backend::{ AuthorizeBackend, PermissionProviderSource },
        psw_auth::PswAuthCredentials,
        http_basic_auth::{ HttpBasicAuthBackend },
        login_form_auth::{ LoginFormAuthBackend, LoginFormAuthConfig },
        oauth2_auth::{ OAuth2AuthBackend, OAuth2AuthCredentials },
    },
    error::{ AuthBackendError },
    user_provider::{ AuthUserProvider },
    psw::PlainPasswordComparator,
    permission::PermissionProvider,
    util::composite_util::{
        backend_usr_prov_ref, backend_perm_prov_ref,
        get_unique_user_provider_ref, get_unique_permission_provider_ref,
    },
};
// -------------------------------------------------------------------------------------------------


pub type Role = crate::permission::predefined::Role;
pub type RolePermissionsSet = crate::permission::predefined::RolePermissionsSet;


#[derive(Clone)]
pub struct CompositeAuthnBackendExample<
    > {
    users_provider: Arc<dyn AuthUserProvider<User=AuthUserExample> + Sync + Send>,
    permission_provider: Arc<dyn PermissionProvider<User=AuthUserExample,Permission=Role,PermissionSet=RolePermissionsSet> + Sync + Send>,
    http_basic_auth_backend: Option<HttpBasicAuthBackend<AuthUserExample,PlainPasswordComparator,RolePermissionsSet>>,
    login_form_auth_backend: Option<LoginFormAuthBackend<AuthUserExample,PlainPasswordComparator,RolePermissionsSet>>,
    oauth2_backend: Option<OAuth2AuthBackend<AuthUserExample,RolePermissionsSet>>,
}


impl CompositeAuthnBackendExample {
    fn backends(&self) -> (
        &Option<HttpBasicAuthBackend<AuthUserExample,PlainPasswordComparator,RolePermissionsSet>>,
        &Option<LoginFormAuthBackend<AuthUserExample,PlainPasswordComparator,RolePermissionsSet>>,
        &Option<OAuth2AuthBackend<AuthUserExample,RolePermissionsSet>>,
    ) {
        (&self.http_basic_auth_backend, &self.login_form_auth_backend, &self.oauth2_backend)
    }

    pub fn test_users() -> Result<CompositeAuthnBackendExample, anyhow::Error> {
        let in_mem_users = Arc::new(test::in_memory_test_users() ?);
        let user_provider: Arc<dyn AuthUserProvider<User=AuthUserExample> + Sync + Send> = in_mem_users.clone();
        let permission_provider: Arc<dyn PermissionProvider<User=AuthUserExample,Permission=Role,PermissionSet=RolePermissionsSet> + Sync + Send> = in_mem_users.clone();

        Ok(CompositeAuthnBackendExample {
            http_basic_auth_backend: Some(HttpBasicAuthBackend::new(user_provider.clone(), AuthBackendMode::AuthProposed, permission_provider.clone())),
            login_form_auth_backend: Some(LoginFormAuthBackend::new(user_provider.clone(), LoginFormAuthConfig {
                auth_mode: AuthBackendMode::AuthSupported,
                login_url: "/form/login",
            }, permission_provider.clone())),
            users_provider: user_provider,
            permission_provider,
            oauth2_backend: None,
        })
    }

    pub fn new_raw(
        users_provider: Arc<dyn AuthUserProvider<User=AuthUserExample> + Sync + Send>,
        permission_provider: Arc<dyn PermissionProvider<User=AuthUserExample,Permission=Role,PermissionSet=RolePermissionsSet> + Sync + Send>,
        http_basic_auth_backend: Option<HttpBasicAuthBackend<AuthUserExample,PlainPasswordComparator,RolePermissionsSet>>,
        login_form_auth_backend: Option<LoginFormAuthBackend<AuthUserExample,PlainPasswordComparator,RolePermissionsSet>>,
        oauth2_backend: Option<OAuth2AuthBackend<AuthUserExample,RolePermissionsSet>>,
    ) -> CompositeAuthnBackendExample {
        CompositeAuthnBackendExample { users_provider, http_basic_auth_backend, login_form_auth_backend, oauth2_backend, permission_provider }
    }

    pub fn with_backends(
        http_basic_auth_backend: Option<HttpBasicAuthBackend<AuthUserExample,PlainPasswordComparator,RolePermissionsSet>>,
        login_form_auth_backend: Option<LoginFormAuthBackend<AuthUserExample,PlainPasswordComparator,RolePermissionsSet>>,
        oauth2_backend: Option<OAuth2AuthBackend<AuthUserExample,RolePermissionsSet>>,
    ) -> Result<CompositeAuthnBackendExample, AuthBackendError> {

        let users_provider_ref = get_unique_user_provider_ref([
            backend_usr_prov_ref(&http_basic_auth_backend),
            backend_usr_prov_ref(&login_form_auth_backend),
            backend_usr_prov_ref(&oauth2_backend),
        ]) ? .clone();

        let permission_provider_ref = get_unique_permission_provider_ref([
            backend_perm_prov_ref(&http_basic_auth_backend),
            backend_perm_prov_ref(&login_form_auth_backend),
            backend_perm_prov_ref(&oauth2_backend),
        ]) ? .clone();

        Ok(CompositeAuthnBackendExample {
            users_provider: users_provider_ref,
            http_basic_auth_backend, login_form_auth_backend, oauth2_backend,
            permission_provider: permission_provider_ref })
    }

    // T O D O: Do we need redirection there??
    #[allow(unused_qualifications)]
    pub fn authorize_url(&self) -> Result<(oauth2::url::Url, oauth2::CsrfToken), AuthBackendError> {
        match self.oauth2_backend {
            None => Err(AuthBackendError::NoRequestedBackend),
            Some(ref oauth2_backend) => Ok(oauth2_backend.authorize_url()),
        }
    }
}


#[axum::async_trait]
impl RequestAuthenticated for CompositeAuthnBackendExample {

    async fn do_authenticate_request <
        RootBackend: axum_login::AuthnBackend + 'static,
        S: Send + Sync,
    > (&self, auth_session: axum_login::AuthSession<RootBackend>, req: Request)
       -> (Request, Result<Option<Self::User>, Self::Error>)
        where Self: 'static,
        RootBackend: axum_login::AuthnBackend<User = Self::User>,
    {
        // use tuple_heter_iter_macro::for_each_by_ref;
        use tuple_heter_iter_macro::tuple_for_each_by_ref;
        let mut req_and_res = (req, Ok(None));

        // Faked (really unused) variable to shut up Idea error notification.
        #[allow(dead_code, unused_variables)]
        let backend = &self.http_basic_auth_backend;

        // for_each_by_ref! { $backend, self.http_basic_auth_backend,
        //     self.login_form_auth_backend, self.oauth2_backend, {
        tuple_for_each_by_ref! { $backend, self.backends(), 3, {
            if let Some(ref backend) = backend {
                req_and_res = backend.do_authenticate_request::<RootBackend,()>(
                    auth_session.clone(), req_and_res.0).await;
                match req_and_res.1 {
                    Ok(None) => {} // Ok, lets continue finding user or error
                    _ => return req_and_res,
                }
            };
        } }

        req_and_res
    }
}


#[axum::async_trait]
impl axum_login::AuthnBackend for CompositeAuthnBackendExample {
    type User = AuthUserExample;
    type Credentials = CompositeAuthCredentials;
    type Error = AuthBackendError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        match creds {
            CompositeAuthCredentials::Password(creds) =>
                // There is no http backend because it has the same 'authenticate'
                // method with the same credentials type.
                //
                match self.login_form_auth_backend {
                    None => Err(AuthBackendError::NoRequestedBackend),
                    Some(ref backend) => backend.authenticate(creds).await.map_err(AuthBackendError::from)
                },
            CompositeAuthCredentials::OAuth(creds) =>
                match self.oauth2_backend {
                    None => Err(AuthBackendError::NoRequestedBackend),
                    Some(ref backend) => backend.authenticate(creds).await.map_err(AuthBackendError::from)
                },
        }
    }

    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.users_provider.get_user_by_principal_identity(user_id).await.map_err(From::from)
    }
}


#[derive(Debug, Clone, serde::Deserialize)]
pub enum CompositeAuthCredentials {
    Password(PswAuthCredentials),
    OAuth(OAuth2AuthCredentials),
}


#[derive(Debug, Clone)]
pub enum CompositeProposeAuthAction {
    ProposeLoginFormAuthAction(ProposeLoginFormAuthAction),
    ProposeHttpBasicAuthAction(ProposeHttpBasicAuthAction),
}
impl ProposeAuthAction for CompositeProposeAuthAction { }
#[inherent::inherent]
impl IntoResponse for CompositeProposeAuthAction {
    #[allow(dead_code)] // !! It is really used IMPLICITLY !!
    fn into_response(self) -> Response {
        match self {
            CompositeProposeAuthAction::ProposeLoginFormAuthAction(action) =>
                action.into_response(),
            CompositeProposeAuthAction::ProposeHttpBasicAuthAction(action) =>
                action.into_response(),
        }
    }
}
impl From<ProposeHttpBasicAuthAction> for CompositeProposeAuthAction {
    fn from(value: ProposeHttpBasicAuthAction) -> Self {
        CompositeProposeAuthAction::ProposeHttpBasicAuthAction(value)
    }
}
impl From<ProposeLoginFormAuthAction> for CompositeProposeAuthAction {
    fn from(value: ProposeLoginFormAuthAction) -> Self {
        CompositeProposeAuthAction::ProposeLoginFormAuthAction(value)
    }
}


#[axum::async_trait]
impl AuthnBackendAttributes for CompositeAuthnBackendExample {
    type ProposeAuthAction = CompositeProposeAuthAction;

    fn user_provider(&self) -> Arc<dyn AuthUserProvider<User=AuthUserExample> + Sync + Send> {
        self.users_provider.clone()
    }
    fn user_provider_ref<'a>(&'a self) -> &'a Arc<dyn AuthUserProvider<User=Self::User> + Sync + Send> {
        &self.users_provider
    }

    fn propose_authentication_action(&self, req: &Request) -> Option<Self::ProposeAuthAction> {
        // use tuple_heter_iter_macro::for_each_by_ref;
        // use tuple_heter_iter_macro::tuple_for_each_by_ref;
        use tuple_heter_iter_macro::tuple_find_some_by_ref;

        /*
        use forr::forr;
        forr! { $val:expr, $i:idx in [(), (2,), (2,3), ""] $*
            println!("### in forr: {:?}", $val);
        }
        forr! { $val:expr, $i:idx in [(), (2,), (2,3), ""] $*
            {
                println!("### in forr: {:?}", $val);
            }
        }
        */

        // Faked (really unused) variable to shut up Idea error notification.
        #[allow(dead_code, unused_variables)]
        let backend = &self.http_basic_auth_backend;
        use if_chain::if_chain;

        let propose_opt = tuple_find_some_by_ref! { $backend, self.backends(), {
            if_chain! {
                if let Some(ref backend) = backend;
                let proposes_auth_action = backend.propose_authentication_action(&req);
                if let Some(proposes_auth_action) = proposes_auth_action;
                then { Some(proposes_auth_action.into()) }
                else { None }
            }
            /*
            if let Some(ref backend) = backend {
                let proposes_auth_action = backend.propose_authentication_action(&req);
                if let Some(proposes_auth_action) = proposes_auth_action {
                    Some(proposes_auth_action.into())
                } else {
                    None
                }
            } else { None }
            */
        }};

        propose_opt
    }
}



#[axum::async_trait]
impl PermissionProviderSource for CompositeAuthnBackendExample {
    type User = AuthUserExample;
    type Permission = Role;
    type PermissionSet = RolePermissionsSet;

    #[inline]
    //noinspection DuplicatedCode
    fn permission_provider(&self) -> Arc<dyn PermissionProvider<User=AuthUserExample,Permission=Role,PermissionSet=RolePermissionsSet> + Send + Sync> {
        self.permission_provider.clone()
    }
    #[inline]
    //noinspection DuplicatedCode
    fn permission_provider_ref<'a>(&'a self) -> &'a Arc<dyn PermissionProvider<User=Self::User, Permission=Self::Permission, PermissionSet=Self::PermissionSet> + Send + Sync> {
        &self.permission_provider
    }
}
#[axum::async_trait]
impl AuthorizeBackend for CompositeAuthnBackendExample { }


pub mod test {
    use super::AuthUserExample;
    use super::super::auth_user::AuthUserExamplePswExtractor;
    use crate::{
        user_provider::{ AuthUserProviderError, InMemAuthUserProvider },
        permission::{ PermissionSet, predefined::{ Role, RolePermissionsSet, }},
    };

    pub fn in_memory_test_users()
        -> Result<InMemAuthUserProvider<AuthUserExample,Role,RolePermissionsSet,AuthUserExamplePswExtractor>, AuthUserProviderError> {
        InMemAuthUserProvider::with_users(vec!(
            AuthUserExample::new(1, "vovan", "qwerty"),
            AuthUserExample::with_role(1, "vovan-read", "qwerty", Role::Read),
            AuthUserExample::with_role(1, "vovan-write", "qwerty", Role::Write),
            AuthUserExample::with_roles(1, "vovan-read-and-write", "qwerty",
                RolePermissionsSet::from_permissions([Role::Read, Role::Write])),
        ))
    }

}
