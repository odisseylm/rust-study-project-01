use std::fmt::Debug;
use std::sync::Arc;
use anyhow::anyhow;
use axum::extract::Request;
use implicit_clone::ImplicitClone;
use mvv_common::{
    client_auth_cert_info::ClientAuthCertInfo,
    client_cert_auth::{get_http_current_client_auth_cert_from_req, get_http_current_client_auth_cert_from_req_parts},
};
use mvv_common::string::{remove_optional_prefix, remove_optional_suffix};
use crate::{
    AuthnBackendAttributes, AuthUserProvider,
    backend::{
        NoProposeHttpAuthAction, RequestAuthenticated,
        authz_backend::{AuthorizeBackend, PermissionProviderSource},
    },
    error::AuthBackendError,
    permission::{
        PermissionProvider, PermissionSet,
        empty_perm_provider::{AlwaysAllowedPermSet, EmptyPerm}
    },
};
//--------------------------------------------------------------------------------------------------



#[derive(Debug, Clone)]
#[readonly::make] // should be after 'derive'
pub struct ClientCertAuthBackend <
    User: axum_login::AuthUser,
    PermSet: PermissionSet + Clone = AlwaysAllowedPermSet<EmptyPerm>,
> where
    User: axum_login::AuthUser,
    <User as axum_login::AuthUser>::Id: TryFrom<String>,
    <<User as axum_login::AuthUser>::Id as TryFrom<String>>::Error: Debug,
{
    pub(crate) users_provider: Arc<dyn AuthUserProvider<User=User> + Send + Sync>,
    pub(crate) permission_provider: Arc<dyn PermissionProvider<User=User,
        Permission=<PermSet as PermissionSet>::Permission,PermissionSet=PermSet> + Send + Sync>,
}


#[derive(Debug, Clone)]
pub struct ClientCertAuthCredentials {
    client_cert: ClientAuthCertInfo,
}


impl <
    Usr: axum_login::AuthUser,
    PermSet: PermissionSet + Clone,
> ClientCertAuthBackend<Usr,PermSet>
    where
        Usr: axum_login::AuthUser,
        <Usr as axum_login::AuthUser>::Id: TryFrom<String>,
        <<Usr as axum_login::AuthUser>::Id as TryFrom<String>>::Error: Debug,
{
    #[inline]
    pub fn new(
        users_provider: Arc<dyn AuthUserProvider<User=Usr> + Send + Sync>,
        permission_provider: Arc<dyn PermissionProvider<User=Usr,
            Permission=<PermSet as PermissionSet>::Permission,PermissionSet=PermSet> + Send + Sync>,
    ) -> Result<ClientCertAuthBackend<Usr,PermSet>, AuthBackendError> {
        Ok(ClientCertAuthBackend::<Usr,PermSet> {
            users_provider, permission_provider
        })
    }

    #[allow(dead_code)]
    pub fn new_by_static_type <UsrProvider, U, P, PS> (users_and_perm_provider: Arc<UsrProvider>)
        -> Result<ClientCertAuthBackend<U,PS>, AuthBackendError>
    where
        U: axum_login::AuthUser,
        <U as axum_login::AuthUser>::Id: TryFrom<String>,
        <<U as axum_login::AuthUser>::Id as TryFrom<String>>::Error: Debug,
        PS: PermissionSet<Permission=P> + Clone,
        UsrProvider: Send + Sync + 'static + AuthUserProvider<User=U>
            + PermissionProvider<User=U,Permission=P,PermissionSet=PS>
    {
        // This combines the session layer with our backend to establish the auth service
        // which will provide the auth session as a request extension.
        //
        let user_provider: Arc<dyn AuthUserProvider<User=U> + Send + Sync> = users_and_perm_provider.implicit_clone();
        let perm_provider: Arc<dyn PermissionProvider<User=U,Permission=P,PermissionSet=PS> + Send + Sync> =
            users_and_perm_provider.implicit_clone();

        // Rust does not support casting dyn sub-trait to dyn super-trait :-(
        // let std_usr_provider: Arc<dyn crate::auth::AuthUserProvider<User = AuthUser> + Send + Sync> = wrap_static_ptr_auth_user_provider(Arc::clone(&usr_provider_impl));
        // Seems we may not use wrap_static_ptr_auth_user_provider(Arc::clone(&usr_provider_impl))
        // but we need to use usr_provider_impl.clone(), NOT Arc::clone(&usr_provider_impl) !!!
        // !!! With Arc::clone(&usr_provider_impl) auto casting does NOT work !!!
        //

        ClientCertAuthBackend::new(user_provider, perm_provider)
    }

    async fn do_authenticate_impl(&self, creds: Option<ClientCertAuthCredentials>) -> Result<Option<Usr>, AuthBackendError>
        where Self: 'static {

        match creds {
            None => Ok(None),
            Some(creds) => {
                let user_principal_id: String = extract_username_from_cert_user(
                    creds.client_cert.common_name) ?;

                // It would be nice to avoid 'clone', but because rust does not support
                // trait impl 'specialization' (in stable) it would be impossible.
                // Logging incorrect user is more preferable.
                //
                let user_principal_id: <Usr as axum_login::AuthUser>::Id = user_principal_id.clone().try_into()
                        .map_err(|err|AuthBackendError::ExtractUserFromReqError(anyhow!(
                            "Error converting user [{user_principal_id}] from string to principal ID ({err:?})"
                        ))) ?;

                let user = self.users_provider
                    .get_user_by_principal_identity(&user_principal_id).await ?;
                Ok(user)
            },
        }
    }
}


#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + 'static,
    PermSet: PermissionSet + Clone + 'static,
> axum_login::AuthnBackend for ClientCertAuthBackend<Usr,PermSet>
    where
        <Usr as axum_login::AuthUser>::Id: TryFrom<String>,
        <<Usr as axum_login::AuthUser>::Id as TryFrom<String>>::Error: Debug,
{
    type User = Usr;
    type Credentials = ClientCertAuthCredentials;
    type Error = AuthBackendError;

    #[inline]
    //noinspection DuplicatedCode
    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        self.do_authenticate_impl(Some(creds)).await
    }
    #[inline]
    //noinspection DuplicatedCode
    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = self.users_provider.get_user_by_principal_identity(user_id).await ?;
        Ok(user)
    }
}


// #[cfg(not(feature = "ambassador"))]
#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser,
    PermSet: PermissionSet + Clone,
> PermissionProviderSource for ClientCertAuthBackend<Usr,PermSet>
    where
        <Usr as axum_login::AuthUser>::Id: TryFrom<String>,
        <<Usr as axum_login::AuthUser>::Id as TryFrom<String>>::Error: Debug,
    {
    type User = Usr;
    type Permission = <PermSet as PermissionSet>::Permission;
    type PermissionSet = PermSet;

    #[inline]
    //noinspection DuplicatedCode
    fn permission_provider(&self) -> Arc<dyn PermissionProvider<User=Self::User, Permission=Self::Permission, PermissionSet=Self::PermissionSet> + Send + Sync> {
        self.permission_provider.clone()
    }
    #[inline]
    //noinspection DuplicatedCode
    fn permission_provider_ref<'a>(&'a self) -> &'a Arc<dyn PermissionProvider<User=Self::User, Permission=Self::Permission, PermissionSet=Self::PermissionSet> + Send + Sync> {
        &self.permission_provider
    }
}


// #[cfg(not(feature = "ambassador"))] // not supported by 'ambassador' now since it is not delegation
#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser,
    PermSet: PermissionSet + Clone,
> AuthorizeBackend for ClientCertAuthBackend<Usr,PermSet>
    where
        <Usr as axum_login::AuthUser>::Id: TryFrom<String>,
        <<Usr as axum_login::AuthUser>::Id as TryFrom<String>>::Error: Debug,
{
    //noinspection DuplicatedCode
}

#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + 'static,
    PermSet: PermissionSet + Clone + 'static,
> AuthnBackendAttributes for ClientCertAuthBackend<Usr,PermSet>
    where
        <Usr as axum_login::AuthUser>::Id: TryFrom<String>,
        <<Usr as axum_login::AuthUser>::Id as TryFrom<String>>::Error: Debug,
{
    type ProposeAuthAction = NoProposeHttpAuthAction;

    #[inline]
    fn user_provider(&self) -> Arc<dyn AuthUserProvider<User=Usr> + Send + Sync> {
        self.users_provider.clone()
    }
    #[inline]
    fn user_provider_ref<'a>(&'a self) -> &'a Arc<dyn AuthUserProvider<User=Self::User> + Sync + Send> {
        &self.users_provider
    }
    #[inline]
    fn propose_authentication_action(&self, _: &Request) -> Option<Self::ProposeAuthAction> {
        None
    }
}


#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + 'static,
    PermSet: PermissionSet + Clone + 'static,
> RequestAuthenticated for ClientCertAuthBackend<Usr,PermSet>
    where
        <Usr as axum_login::AuthUser>::Id: TryFrom<String>,
        <<Usr as axum_login::AuthUser>::Id as TryFrom<String>>::Error: Debug,
{
    async fn do_authenticate_request <
        RootBackend: axum_login::AuthnBackend + 'static,
        S: Send + Sync,
    > (&self, _auth_session: Option<axum_login::AuthSession<RootBackend>>, req: Request)
    -> (Request, Result<Option<Self::User>, Self::Error>)
    where Self: 'static
    {
        let creds = get_credentials_from_req(&req);
        match creds {
            Err(err) => (req, Err(err)),
            Ok(creds) => {
                (req, self.do_authenticate_impl(creds).await)
            }
        }
    }

    async fn do_authenticate_request_parts <
        RootBackend: axum_login::AuthnBackend + 'static,
        S: Send + Sync,
    > (&self, _auth_session: Option<axum_login::AuthSession<RootBackend>>, req: &http::request::Parts)
    -> Result<Option<Self::User>, Self::Error>
    where Self: 'static {
        let creds = get_credentials_from_req_parts(&req) ?;
        self.do_authenticate_impl(creds).await
    }
}


//--------------------------------------------------------------------------------------------------

fn get_credentials_from_req(req: &Request) -> Result<Option<ClientCertAuthCredentials>, AuthBackendError> {
    let client_auth_cert = get_http_current_client_auth_cert_from_req(&req)
        .map_err(AuthBackendError::ExtractUserFromReqError) ?;
    Ok(client_auth_cert.map(|client_cert| ClientCertAuthCredentials { client_cert }))
}
fn get_credentials_from_req_parts(req_parts: &http::request::Parts) -> Result<Option<ClientCertAuthCredentials>, AuthBackendError> {
    let client_auth_cert = get_http_current_client_auth_cert_from_req_parts(&req_parts)
        .map_err(AuthBackendError::ExtractUserFromReqError) ?;
    Ok(client_auth_cert.map(|client_cert| ClientCertAuthCredentials { client_cert }))
}


fn extract_username_from_cert_user(user: String) -> Result<String, AuthBackendError> {
    let user = user.to_lowercase(); // T O D O: why not COW?

    let is_proper_user_cert_name_format =
        user.starts_with("user-") || user.ends_with("-cn-user") || user.ends_with("-user");
    if !is_proper_user_cert_name_format {
        return Err(AuthBackendError::ExtractUserFromReqError(anyhow!(
                        "Unexpected client certificate username format [{user}]")));
    }

    let user_principal_id: String =
        if user.starts_with("user-") {
            remove_optional_prefix(user, "user-")
        } else {
            let user = remove_optional_suffix(user, "-cn-user");
            remove_optional_suffix(user, "-user")
        };

    Ok(user_principal_id)
}
