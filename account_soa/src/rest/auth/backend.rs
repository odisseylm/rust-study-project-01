// use core::fmt;
use std::sync::Arc;

use axum::extract::Request;
use axum::response::{ IntoResponse, Response };
use implicit_clone::ImplicitClone;

use mvv_auth::{
    AuthBackendError, PlainPasswordComparator, PasswordComparator,
    backend::{
        AuthBackendMode, AuthnBackendAttributes, ProposeAuthAction,
        ProposeHttpBasicAuthAction, ProposeLoginFormAuthAction, RequestAuthenticated,
        authz_backend::{ AuthorizeBackend, PermissionProviderSource },
        psw_auth::PswAuthCredentials,
        http_basic_auth::{ HttpBasicAuthBackend },
        login_form_auth::{ LoginFormAuthBackend, LoginFormAuthConfig },
        oauth2_auth::{ OAuth2AuthBackend, OAuth2AuthCredentials, OAuth2Config, OAuth2UserStore },
    },
    user_provider::{ AuthUserProvider },
    permission::PermissionProvider,
    // util::composite_util::{
    //     backend_usr_prov_ref, backend_perm_prov_ref,
    //     get_unique_user_provider_ref, get_unique_permission_provider_ref,
    // },
};
use mvv_common::backtrace::backtrace;
use super::user::{AuthUser, Role, RolePermissionsSet };
// -------------------------------------------------------------------------------------------------



#[derive(Clone)]
//noinspection DuplicatedCode
pub struct CompositeAuthBackend <
    > {
    user_provider: Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send>,
    permission_provider: Arc<dyn PermissionProvider<User=AuthUser,Permission=Role,PermissionSet=RolePermissionsSet> + Sync + Send>,
    //
    http_basic_auth_backend: Option<HttpBasicAuthBackend<AuthUser,RolePermissionsSet>>,
    login_form_auth_backend: Option<LoginFormAuthBackend<AuthUser,RolePermissionsSet>>,
    oauth2_backend: Option<OAuth2AuthBackend<AuthUser,RolePermissionsSet>>,
}


//noinspection DuplicatedCode
impl CompositeAuthBackend {
    fn backends(&self) -> (
        &Option<HttpBasicAuthBackend<AuthUser,RolePermissionsSet>>,
        &Option<LoginFormAuthBackend<AuthUser,RolePermissionsSet>>,
        &Option<OAuth2AuthBackend<AuthUser,RolePermissionsSet>>,
    ) {
        (&self.http_basic_auth_backend, &self.login_form_auth_backend, &self.oauth2_backend)
    }

    pub fn new <UsrProvider> (
        psw_comp: Arc<dyn PasswordComparator + Send + Sync>,
        users_and_perm_provider: Arc<UsrProvider>,
    )
        -> Result<CompositeAuthBackend, AuthBackendError>
    where
        UsrProvider: Send + Sync + 'static
                   + AuthUserProvider<User=AuthUser>
                   + PermissionProvider<User=AuthUser,Permission=Role,PermissionSet=RolePermissionsSet>
                   + OAuth2UserStore,
    {
        // This combines the session layer with our backend to establish the auth service
        // which will provide the auth session as a request extension.
        //
        let user_provider: Arc<dyn AuthUserProvider<User=AuthUser> + Send + Sync> = users_and_perm_provider.implicit_clone();
        let oauth_user_provider: Arc<dyn OAuth2UserStore<User=AuthUser> + Send + Sync> = users_and_perm_provider.implicit_clone();
        let permission_provider: Arc<dyn PermissionProvider<User=AuthUser, Permission=Role, PermissionSet=RolePermissionsSet> + Send + Sync> =
            users_and_perm_provider.implicit_clone();

        // Rust does not support casting dyn sub-trait to dyn super-trait :-(
        // let std_usr_provider: Arc<dyn crate::auth::AuthUserProvider<User = AuthUser> + Send + Sync> = wrap_static_ptr_auth_user_provider(Arc::clone(&usr_provider_impl));
        // Seems we may not use wrap_static_ptr_auth_user_provider(Arc::clone(&usr_provider_impl))
        // but we need to use usr_provider_impl.clone(), NOT Arc::clone(&usr_provider_impl) !!!
        // !!! With Arc::clone(&usr_provider_impl) auto casting does NOT work !!!
        //
        let config = OAuth2Config::git_from_env() ?;
        let oauth2_backend_opt: Option<OAuth2AuthBackend<AuthUser,RolePermissionsSet>> = match config {
            None => None,
            Some(config) => {
                let mut config = config.clone();
                // config.auth_mode = AuthBackendMode::AuthProposed;
                config.login_url = "/login";

                Some(OAuth2AuthBackend::new(
                    Arc::clone(&user_provider),
                    Arc::clone(&oauth_user_provider), // it is automatically cast to another 'dyn' object. It should be done THERE!
                    config,
                    None, // default will be created
                    Arc::clone(&permission_provider),
                ) ?)
            }
        };

        let http_basic_auth_backend = HttpBasicAuthBackend::<AuthUser, RolePermissionsSet>::new(
            Arc::clone(&psw_comp),
            Arc::clone(&user_provider),
            // AuthBackendMode::AuthProposed, // It makes sense for pure server SOA (especially for testing)
            AuthBackendMode::AuthSupported,
            Arc::clone(&permission_provider),
        );
        let login_form_auth_backend = LoginFormAuthBackend::<AuthUser, RolePermissionsSet>::new(
            Arc::clone(&psw_comp),
            Arc::clone(&user_provider),
            // It makes sense for web-app
            LoginFormAuthConfig {
                auth_mode: AuthBackendMode::AuthProposed,
                login_url: "/login",
            },
            Arc::clone(&permission_provider),
        );

        Ok(CompositeAuthBackend {
            user_provider,
            permission_provider,
            http_basic_auth_backend: Some(http_basic_auth_backend),
            login_form_auth_backend: Some(login_form_auth_backend),
            // None,
            oauth2_backend: oauth2_backend_opt,
        })
    }

    pub fn test_users() -> Result<CompositeAuthBackend, anyhow::Error> {
        let psw_comp: Arc<dyn PasswordComparator + Sync + Send> = Arc::new(PlainPasswordComparator::new());
        let in_mem_users = Arc::new(test::in_memory_test_users() ?);
        let user_provider: Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send> = in_mem_users.implicit_clone();
        let permission_provider: Arc<dyn PermissionProvider<User=AuthUser,Permission=Role,PermissionSet=RolePermissionsSet> + Sync + Send> = in_mem_users.implicit_clone();

        Ok(CompositeAuthBackend {
            http_basic_auth_backend: Some(HttpBasicAuthBackend::new(Arc::clone(&psw_comp), Arc::clone(&user_provider), AuthBackendMode::AuthProposed, Arc::clone(&permission_provider))),
            login_form_auth_backend: Some(LoginFormAuthBackend::new(Arc::clone(&psw_comp), Arc::clone(&user_provider), LoginFormAuthConfig {
                auth_mode: AuthBackendMode::AuthSupported,
                login_url: "/login",
            }, Arc::clone(&permission_provider))),
            user_provider,
            permission_provider,
            oauth2_backend: None,
        })
    }

    // T O D O: Do we need redirection there??
    #[allow(unused_qualifications)]
    pub fn authorize_url(&self) -> Result<(oauth2::url::Url, oauth2::CsrfToken), AuthBackendError> {
        match self.oauth2_backend {
            None => Err(AuthBackendError::NoRequestedBackend(backtrace())),
            Some(ref oauth2_backend) => Ok(oauth2_backend.authorize_url()),
        }
    }
}


#[axum::async_trait]
//noinspection DuplicatedCode
impl RequestAuthenticated for CompositeAuthBackend {

    async fn do_authenticate_request <
        RootBackend: axum_login::AuthnBackend + 'static,
        S: Send + Sync,
    > (&self, auth_session: axum_login::AuthSession<RootBackend>, req: Request)
       -> (Request, Result<Option<Self::User>, Self::Error>)
        where Self: 'static,
        RootBackend: axum_login::AuthnBackend<User = Self::User>,
    {
        use mvv_tuple_heter_iter_macro::tuple_for_each_by_ref;
        let mut req_and_res = (req, Ok(None));

        // Faked (really unused) variable to shut up Idea error notification.
        #[allow(dead_code, unused_variables)]
        let backend = &self.http_basic_auth_backend;

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


    async fn do_authenticate_request_parts <
        RootBackend: axum_login::AuthnBackend + 'static,
        S: Send + Sync,
    > (&self, auth_session: axum_login::AuthSession<RootBackend>, req: &http::request::Parts)
       -> Result<Option<Self::User>, Self::Error>
        where Self: 'static,
        RootBackend: axum_login::AuthnBackend<User = Self::User>,
    {
        use mvv_tuple_heter_iter_macro::tuple_for_each_by_ref;
        let mut res = Ok(None);

        // Faked (really unused) variable to shut up Idea error notification.
        #[allow(dead_code, unused_variables)]
        let backend = &self.http_basic_auth_backend;

        tuple_for_each_by_ref! { $backend, self.backends(), 3, {
            if let Some(ref backend) = backend {
                res = backend.do_authenticate_request_parts::<RootBackend,()>(
                    auth_session.clone(), req).await;
                match res {
                    Ok(None) => {} // Ok, lets continue finding user or error
                    _ => return res,
                }
            };
        } }

        res
    }

}


#[axum::async_trait]
//noinspection DuplicatedCode
impl axum_login::AuthnBackend for CompositeAuthBackend {
    type User = AuthUser;
    type Credentials = CompositeAuthCredentials;
    type Error = AuthBackendError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        match creds {
            CompositeAuthCredentials::Password(creds) =>
                // There is no http backend because it has the same 'authenticate'
                // method with the same credentials type.
                //
                match self.login_form_auth_backend {
                    None => Err(AuthBackendError::NoRequestedBackend(backtrace())),
                    Some(ref backend) => backend.authenticate(creds).await.map_err(AuthBackendError::from)
                },
            CompositeAuthCredentials::OAuth(creds) =>
                match self.oauth2_backend {
                    None => Err(AuthBackendError::NoRequestedBackend(backtrace())),
                    Some(ref backend) => backend.authenticate(creds).await.map_err(AuthBackendError::from)
                },
        }
    }

    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.user_provider.get_user_by_principal_identity(user_id)
            .await.map_err(From::from)
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
//noinspection DuplicatedCode
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
//noinspection DuplicatedCode
impl AuthnBackendAttributes for CompositeAuthBackend {
    type ProposeAuthAction = CompositeProposeAuthAction;

    fn user_provider(&self) -> Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send> {
        Arc::clone(&self.user_provider)
    }
    fn user_provider_ref<'a>(&'a self) -> &'a Arc<dyn AuthUserProvider<User=Self::User> + Sync + Send> {
        &self.user_provider
    }

    fn propose_authentication_action(&self, req: &Request) -> Option<Self::ProposeAuthAction> {
        use mvv_tuple_heter_iter_macro::tuple_find_some_by_ref;

        // Faked (really unused) variable to shut up Idea error notification.
        #[allow(dead_code, unused_variables)]
        let backend_opt = &self.http_basic_auth_backend;

        let propose_opt = tuple_find_some_by_ref! { $backend_opt, self.backends(), {
            backend_opt.as_ref().and_then(|backend|
                backend.propose_authentication_action(&req)
                    .map(|action|action.into()))
        }};

        propose_opt
    }
}



#[axum::async_trait]
//noinspection DuplicatedCode
impl PermissionProviderSource for CompositeAuthBackend {
    type User = AuthUser;
    type Permission = Role;
    type PermissionSet = RolePermissionsSet;

    #[inline]
    //noinspection DuplicatedCode
    fn permission_provider(&self) -> Arc<dyn PermissionProvider<User=AuthUser,Permission=Role,PermissionSet=RolePermissionsSet> + Send + Sync> {
        Arc::clone(&self.permission_provider)
    }
    #[inline]
    //noinspection DuplicatedCode
    fn permission_provider_ref<'a>(&'a self) -> &'a Arc<dyn PermissionProvider<User=Self::User, Permission=Self::Permission, PermissionSet=Self::PermissionSet> + Send + Sync> {
        &self.permission_provider
    }
}
#[axum::async_trait]
impl AuthorizeBackend for CompositeAuthBackend { }



pub mod test {
    use mvv_auth::{
        AuthUserProviderError,
        permission::PermissionSet,
        user_provider::InMemAuthUserProvider,
    };
    use super::super::user::{
        AuthUser, UserRolesExtractor, Role, RolePermissionsSet,
    };

    pub fn in_memory_test_users()
        -> Result<InMemAuthUserProvider<AuthUser,Role,RolePermissionsSet,UserRolesExtractor>, AuthUserProviderError> {
        InMemAuthUserProvider::with_users([
            AuthUser::new(1, "vovan", "qwerty"),
            AuthUser::with_role(2, "vovan-write", "qwerty", Role::Write),
            AuthUser::with_role(3, "vovan-read", "qwerty", Role::Read),
            AuthUser::with_roles(4, "vovan-read-and-write", "qwerty",
                RolePermissionsSet::from_permissions([Role::Read, Role::Write])),
        ])
    }
}
