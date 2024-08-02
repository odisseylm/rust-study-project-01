use std::sync::Arc;
use axum::{ Router, routing::get as GET };
use axum::extract::State;
use axum::response::IntoResponse;
use crate::{
    app_dependencies::Dependencies,
    auth::{ ClientFeature, RequiredAuthorizationExtension, ExtractCurrentUser },
    error::WebAppError,
    rest_dependencies::account_soa_client::{
        types::Account,
    },
    service::account_service::AccountService,
};
//--------------------------------------------------------------------------------------------------



pub fn accounts_web_router <
    AccountS: AccountService + Send + Sync + 'static,
> (
    dependencies: Arc<Dependencies<AccountS>>,
) -> Router {
    // use mvv_common::utoipa::AxumOpenApiRouterExt;

    let shared_state: Arc<AccountS> = Arc::clone(&dependencies.state.account_service);

    let r = Router::new()
        // .route_from_open_api("clent_accounts", rest_get_client_account::<AccountS>)
        // .route_from_open_api(open_api_route!(rest_get_client_account::<AccountS>))
        // .route_from_open_api(open_api_route!(rest_get_client_accounts::<AccountS>))
        // .route_from_open_api(open_api_route!(rest_transfer_amount::<AccountS>))
        .with_state(Arc::clone(&shared_state))
        // .role_required(Role::Read)

        // investigation block
        // .merge(Router::new()
        //     .route("/validate_test/input_validate_1", POST(call_rest_input_validate_by_validator::<AccountS>))
        //     .route("/validate_test/input_validate_3", POST(call_rest_input_validate_by_validify::<AccountS>))
        // )
        // .with_state(Arc::clone(&shared_state))
        ;

    r
}


#[derive(askama::Template)]
#[template(path = "client_accounts.html")]
struct ClientAccountsTemplate<'a> {
    client_id: &'a str,
    accounts: &'a Vec<Account>,
}

pub fn current_client_accounts_router <
    AccountS: AccountService + Send + Sync + 'static,
>(
    dependencies: Arc<Dependencies<AccountS>>,
) -> Router<()> {
    Router::new()
        .route("/current_client_accounts", GET(current_client_accounts::<AccountS>))
        .client_feature_required(ClientFeature::Standard)
        .with_state(dependencies)
}

// pub type UsrExtr<S> = ExtractCurrentUser<crate::auth::AuthBackend, ClientAuthUser, S>;

pub async fn current_client_accounts <
    AccountS: AccountService + Send + Sync + 'static,
> (
    // auth_session: axum_login::AuthSession<AuthBackend>,
    State(dependencies): State<Arc<Dependencies<AccountS>>>,
    // If we use auth session `axum_login::AuthSession<AuthBackend>`,
    // we will not be able to use Basic HTTP Auth.
    // ExtractCurrentUser { user: client_user, _pd }: ExtractCurrentUser<ClientAuthUser, crate::auth::AuthBackend>,
    current_user: ExtractCurrentUser,
) -> Result<impl IntoResponse, WebAppError> {

    let account_service = &dependencies.state.account_service;
    // TODO: proper process wrong formatted ID as 'bla-bla'

    let client_id = current_user.user.client_id; // auth_session.user.map(|user| user.client_id);
    // let client_id = "bla-bla".to_owned(); // T O D O: temp
    let accounts = account_service.get_client_accounts(&client_id).await ?;

    Ok(ClientAccountsTemplate {
        client_id: client_id.as_str(),
        accounts: &accounts,
    }.into_response())
}