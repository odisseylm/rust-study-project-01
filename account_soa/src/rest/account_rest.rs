use std::sync::Arc;
use axum::{
    Router, Json, routing::{ post as POST }, extract::{ Path, State, },
};
use axum_valid::{ Validified, /*Modified,*/ };
// use mvv_common::mvv_axum_valid::{ Validified, /*Modified,*/ };
use axum_valid::Valid as Valid;
// use mvv_common::mvv_axum_valid::Valid as Valid;
use bigdecimal::BigDecimal;
use tracing::{ debug, info /*, error*/ };
use log::{ debug as log_debug, info as log_info /*, error as log_error*/ };
use serde::{ Deserialize };
use utoipa::OpenApi;
use crate::{
    entity::{ self, prelude::UserId, AccountId, ClientId },
    rest::{
        auth::{ RequiredAuthorizationExtension, Role, },
        app_dependencies::Dependencies,
        dto::{ self, CURRENCY_PATTERN, ID_PATTERN },
        error_rest::{ RestAppError },
    },
    service::{ account_service::{ AccountService }, },
};
use super::path;
use mvv_common::rest::RestFutureToJson;

use mvv_common::{
    axum_open_api_axum_route as open_api_route,
};
//--------------------------------------------------------------------------------------------------



pub fn accounts_rest_router <
    AccountS: AccountService + Send + Sync + 'static,
> (
    dependencies: Dependencies<AccountS>,
) -> Router {
    use mvv_common::utoipa::AxumOpenApiRouterExt;

    let shared_state: Arc<AccountRest<AccountS>> = Arc::clone(&dependencies.state.account_rest);

    let r = Router::new()
        .route_from_open_api(open_api_route!(rest_get_client_account::<AccountS>))
        .route_from_open_api(open_api_route!(rest_get_client_accounts::<AccountS>))
        .route_from_open_api(open_api_route!(rest_transfer_amount::<AccountS>))
        .with_state(Arc::clone(&shared_state))
        .role_required(Role::Read)
        // investigation block
        .merge(Router::new()
            .route("/validate_test/input_validate_1", POST(call_rest_input_validate_by_validator::<AccountS>))
            .route("/validate_test/input_validate_3", POST(call_rest_input_validate_by_validify::<AccountS>))
        )
        .with_state(Arc::clone(&shared_state))
        ;

    r
}

// #[static_init::constructor]
#[static_init::dynamic]
static TEMP_CURRENT_USER_ID: UserId = {
    use mvv_common::unchecked::UncheckedResultUnwrap;
    UserId::from_str("11").unchecked_unwrap()
};

#[utoipa::path(
    get,
    path = "/client/{client_id}/account/{account_id}",
    operation_id = "getClientAccount", // in json format
    responses(
        // (status = 200, description = "Client account", body = api::models::Account)
        (status = 200, description = "Client account", body = Account)
    ),
    params(
        ("client_id" = String, Path, description = "Client id", example = "00000000-0000-0000-0000-000000000001"),
        ("account_id" = String, Path, description = "Account ID or IBAN", example="UA713736572172926969841832393"),
    ),
    tag = "mvv_account_soa", // as package/namespace
)]
async fn rest_get_client_account <
    AccountS: AccountService + 'static,
> (
    State(rest_service): State<Arc<AccountRest<AccountS>>>,
    Path(client_id): Path<path::ClientId>,
    Path(account_id): Path<path::AccountId>,
) -> Result<Json<dto::Account>, RestAppError> {
    rest_service.get_account(client_id, account_id).to_json().await
}


#[utoipa::path(
    get,
    path = "/client/{client_id}/account/all",
    operation_id = "getClientAccounts", // in json format
    responses(
        // (status = 200, description = "Client accounts", body = Vec<api::models::Account>)
        (status = 200, description = "Client accounts", body = Vec<Account>)
    ),
    params(
        ("client_id" = String, Path, description = "Client id", example = "00000000-0000-0000-0000-000000000001"),
    ),
    tag = "mvv_account_soa", // as package/namespace
)]
async fn rest_get_client_accounts <
    AccountS: AccountService + 'static,
> (
    State(rest_service): State<Arc<AccountRest<AccountS>>>,
    Path(client_id): Path<path::ClientId>,
)
    -> Result<Json<Vec<dto::Account>>, RestAppError> {
    rest_service.get_accounts(client_id).to_json().await
}


#[derive(utoipa::ToSchema)]
#[schema(as = TransferAmountRequest)] // api::models::Amount)]
#[derive(PartialEq, Eq, Deserialize)]
#[derive(educe::Educe)] #[educe(Debug)]
#[derive(validify::Validify)] // Now using validify::Payload causes compilation errors.
struct TransferAmountRequest {
    /// Account ID (UUID) or IBAN
    #[validate(length(min=1, max=320), regex(ID_PATTERN))]
    #[schema(example = "00000000-0000-0000-0000-000000000101")]
    from_account: String,

    /// Account ID (UUID) or IBAN
    #[validate(length(min=1, max=320), regex(ID_PATTERN))]
    #[schema(example = "00000000-0000-0000-0000-000000000102")]
    to_account: String,

    #[serde(with = "mvv_common::json::serde_json_bd::bd_with")]
    #[educe(Debug(method(mvv_common::entity::bd::bd_dbg_fmt)))]
    #[schema(value_type = f64, example = "12.34")]
    amount: BigDecimal,

    // 'validify' cannot automatically use third-party strings for length validation, but it is ok with regex.
    #[validate(regex(CURRENCY_PATTERN))] // for 'validify' // TODO: fix validation
    #[schema(value_type = String, example = "USD")]
    currency: InnerCurStr,
}

#[utoipa::path(
    post,
    path = "/client/{client_id}/transfer",
    operation_id = "transferAmount", // in json format
    params(
        ("client_id" = String, Path, description = "Client id", example = "00000000-0000-0000-0000-000000000001"),
    ),
    responses(
        (status = 201, description = "Success/Failure", body = ())
    ),
    tag = "mvv_account_soa", // as package/namespace
)]
async fn rest_transfer_amount <
    AccountS: AccountService + 'static,
> (
    State(rest_service): State<Arc<AccountRest<AccountS>>>,
    Path(client_id): Path<path::ClientId>,
    // 'utoipa' conflicts with 'validify'
    // Validified(Json(transfer_request)): Validified<Json<TransferAmountRequest>>,
    Json(transfer_request): Json<TransferAmountRequest>,
)
    -> Result<(), RestAppError> {

    // 'utoipa' conflicts with 'validify', we need to call validation manually.
    use validify::Validate;
    transfer_request.validate().map_err(RestAppError::ValidifyErrors) ?;

    rest_service.transfer(client_id, transfer_request).await
}


#[derive(OpenApi)]
#[openapi(
    paths(
        crate::rest::account_rest::rest_get_client_account,
        crate::rest::account_rest::rest_get_client_accounts,
        crate::rest::account_rest::rest_transfer_amount,
    ),
    components(
        schemas(
            crate::rest::dto::Amount,
            crate::rest::dto::Account,
            TransferAmountRequest,
        ),
    ),
// nest(
//     // you can nest sub apis here
//     (path = "/api/v1/ones", api = one::OneApi)
// )
)]
pub struct AccountRestOpenApi;


// It will be 'current user' in wab-app.
// pub struct CurrentUserAccountRest <AS: AccountService> {

pub struct AccountRest <AS: AccountService> {
    pub account_service: Arc<AS>,
}

impl<AS: AccountService> AccountRest<AS> {

    #[tracing::instrument( skip(self) )]
    pub async fn get_account(&self, client_id: path::ClientId, account_id: path::AccountId) -> Result<dto::Account, RestAppError> {
        use mvv_common::obj_ext::ValExt;
        use core::str::FromStr;

        debug!("TD get_user_account as debug");
        info! ("TI get_user_account as info");
        // error!("TI get_user_account as error");

        log_debug!("LD get_user_account as debug");
        log_info! ("LI get_user_account as info");
        // log_error!("LE get_user_account as error");

        let client_id = ClientId::from_str(&client_id.into_inner()) ?;
        let account_id = account_id.into_inner();

        let is_internal_account_id = account_id.len().is_one_of2(36, 38);
        let account = if is_internal_account_id {
            self.account_service.get_client_account_by_id(
                client_id, AccountId::from_str(&account_id) ?,
            ).await ?
        } else {
            self.account_service.get_client_account_by_iban(
                client_id, iban::Iban::from_str(&account_id) ?,
            ).await ?
        };

        Ok(map_account_to_rest(account))
    }


    #[tracing::instrument( skip(self) )]
    pub async fn get_accounts(&self, client_id: path::ClientId) -> Result<Vec<dto::Account>, RestAppError> {
        let client_id = ClientId::from_str(&client_id.into_inner()) ?;
        let accounts = self.account_service.get_client_accounts(client_id).await ?;

        let accounts_dto = accounts.into_iter()
                .map(move |ac| map_account_to_rest(ac))
                .collect::<Vec<_>>();
        Ok(accounts_dto)
    }


    #[tracing::instrument( skip(self) )]
    pub async fn transfer(&self, client_id: path::ClientId, transfer_request: TransferAmountRequest)
        -> Result<(), RestAppError> {

        use mvv_common::obj_ext::ValExt;
        use core::str::FromStr;

        let TransferAmountRequest {
            from_account: from_account_id,
            to_account: to_account_id,
            amount,
            currency,
        } = transfer_request;

        let client_id = ClientId::from_str(&client_id.into_inner()) ?;
        let from_account_id = from_account_id; // .into_inner();
        let to_account_id = to_account_id; // .into_inner();

        let is_internal_from_account_id = from_account_id.len().is_one_of2(36, 38);
        let is_internal_to_account_id = to_account_id.len().is_one_of2(36, 38);

        let transfer_res =
            if is_internal_from_account_id && is_internal_to_account_id {
                self.account_service.transfer_by_id(
                    client_id,
                    AccountId::from_str(&from_account_id) ?,
                    AccountId::from_str(&to_account_id) ?,
                    entity::prelude::Amount::new(
                        amount,
                        entity::prelude::Currency::from_inner(currency) ?,
                    ),
                ).await?

            } else if !is_internal_from_account_id && !is_internal_to_account_id {
                self.account_service.transfer_by_iban(
                    client_id,
                    iban::Iban::from_str(&from_account_id) ?,
                    iban::Iban::from_str(&to_account_id) ?,
                    entity::prelude::Amount::new(
                        amount,
                        entity::prelude::Currency::from_inner(currency) ?,
                    ),
                ).await?
            } else {
                return Err(RestAppError::IllegalArgument(
                    anyhow::anyhow!("All account IDs should have the same type.")));
            };

        Ok(transfer_res)
    }
}


fn map_account_to_rest(account: entity::Account) -> dto::Account {
    use crate::entity::account::AccountParts;
    use mvv_common::entity::amount::AmountParts;

    let AccountParts { id, iban, client_id, name, amount, created_at, updated_at } = account.into_parts();
    let AmountParts { value: amount_value, currency } = amount.into_parts();
    dto::Account {
        id: id.into_inner().to_string(),
        iban: iban.to_string(), // hm... where 'into_inner()' ??
        client_id: client_id.into_inner().to_string(),
        name,
        amount: dto::Amount { value: amount_value, currency: currency.into_inner() },
        created_at,
        updated_at,
    }
}



//--------------------------------------------------------------------------------------------------
//
//   Investigations
//
//--------------------------------------------------------------------------------------------------
//
async fn call_rest_input_validate_by_validator <
    AccountS: AccountService + 'static,
>(State(_rest_service): State<Arc<AccountRest<AccountS>>>, Valid(Json(input)): Valid<Json<ValidatedInput1>>)
  -> Result<Json<&'static str>, RestAppError> {
    info!("call_rest_input_validate: input = {:?}", input);

    // rest_service.input_validate().to_json().await
    Ok(Json("Ok_1"))
}
use validator::Validate;
use mvv_common::entity::InnerCurStr;

#[derive(Debug, validator::Validate)]
#[derive(serde::Serialize, serde::Deserialize)]
struct ValidatedInput1 {
    #[validate(nested)]
    email_parent_filed: ValidatedSubInput1,
}
#[derive(Debug, validator::Validate)]
#[derive(serde::Serialize, serde::Deserialize)]
struct ValidatedSubInput1 {
    #[validate(email)]
    email33: String,
}

// !! Grade does not work for me and there strange tricks for 'state'.
/*
// use garde::Valid;
use axum_valid::Garde;
// use axum_valid::ValidationRejection::Valid;
async fn call_rest_input_validate_by_garde <
    AccountS: AccountService + 'static,
>(State(rest_service): State<Arc<CurrentUserAccountRest<AccountS>>>, Garde(Json(input)): Garde<Json<ValidatedInput2>>)
    -> Result<Json<String>, RestAppError> {
    info!("call_rest_input_validate: input = {:?}", input);

    // rest_service.input_validate().to_json().await
    Ok(Json("Ok"))
}
#[derive(Debug, garde::Validate)]
#[derive(serde::Serialize, serde::Deserialize)]
struct ValidatedInput2 {
    #[garde(skip)]
    email: String,
}
*/


async fn call_rest_input_validate_by_validify <
    AccountS: AccountService + 'static,
>(State(_rest_service): State<Arc<AccountRest<AccountS>>>, Validified(Json(input)): Validified<Json<ValidatedInput3>>)
  -> Result<Json<&'static str>, RestAppError> {
    info!("call_rest_input_validate: input = {:?}", input);
    Ok(Json("Ok_3"))
}


#[derive(Debug, validify::Validify, validify::Payload)]
#[derive(serde::Deserialize)]
struct ValidatedInput3 {
    #[validate]
    email_parent_filed: ValidatedSubInput3,
}
#[derive(Debug, validify::Validify, validify::Payload)]
#[derive(serde::Deserialize)]
struct ValidatedSubInput3 {
    #[validate(email)]
    // #[validate(length(min = 1, max = 4))]
    email33: String,
}
