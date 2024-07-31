use std::sync::Arc;
use axum::{ Router, routing::get as GET };
use axum::extract::State;
use axum::response::IntoResponse;
use http::StatusCode;
use crate::{
    app_dependencies::Dependencies,
    auth::{ClientType, RequiredAuthorizationExtension},
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
        .client_type_required(ClientType::Usual)
        .with_state(dependencies)
}

struct RequestContext {
    current_client: Option<String>,
}


pub async fn current_client_accounts <
    AccountS: AccountService + Send + Sync + 'static,
> (
    /*auth_session: axum_login::AuthSession<AuthBackend>*/
    State(dependencies): State<Arc<Dependencies<AccountS>>>,
    // request_context: RequestContext,
) -> Result<impl IntoResponse, WebAppError> {

    let account_service = &dependencies.state.account_service;
    let client_id = Some("bla-bla".to_string()); // request_context.current_client;
    match client_id {
        None =>
            // TODO: use better
            Ok(StatusCode::NOT_FOUND.into_response()),
        Some(ref client) => {
            let accounts = account_service.get_client_accounts(client).await ?; // TODO: remove unwrap

            Ok(ClientAccountsTemplate {
                client_id: client.as_str(),
                accounts: &accounts,
            }.into_response())
        }
    }
}