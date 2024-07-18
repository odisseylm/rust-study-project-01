use std::sync::Arc;
use axum::{
    Router, Json, routing::{ get as GET, post as POST }, extract::{ Path, State, },
};
use tracing::{debug, info /*, error*/ };
use log::{ debug as log_debug, info as log_info /*, error as log_error*/ };

use crate::{
    entity::{ self, prelude::UserId, AccountId, ClientId },
    rest::{
        auth::{ RequiredAuthorizationExtension, Role, },
        app_dependencies::Dependencies,
        dto, error_rest::{ RestAppError },
    },
    service::{ account_service::{ AccountService }, },
};
use super::path;
use mvv_common::rest::RestFutureToJson;
//--------------------------------------------------------------------------------------------------


pub fn accounts_rest_router <
    AccountS: AccountService + Send + Sync + 'static,
> (
    dependencies: Dependencies<AccountS>,
) -> Router {

    let shared_state: Arc<AccountRest<AccountS>> = Arc::clone(&dependencies.state.account_rest);

    let accounts_router = Router::new()
        .route("/client/:client_id/account/all", GET(call_rest_get_client_accounts::<AccountS>))
        .route("/client/:client_id/account/:account_id", GET(call_rest_get_client_account::<AccountS>))
        .with_state(Arc::clone(&shared_state))
        .role_required(Role::Read)
        // investigation block
        .merge(Router::new()
            .route("/validate_test/input_validate_1", POST(call_rest_input_validate_by_validator::<AccountS>))
            // .route("/validate_test/input_validate_2", POST(call_rest_input_validate_by_garde::<AccountS>))
            .route("/validate_test/input_validate_3", POST(call_rest_input_validate_by_validify::<AccountS>))
        ).with_state(Arc::clone(&shared_state))
        ;

    accounts_router
}

// #[static_init::constructor]
#[static_init::dynamic]
static TEMP_CURRENT_USER_ID: UserId = {
    use mvv_common::unchecked::UncheckedResultUnwrap;
    UserId::from_str("11").unchecked_unwrap()
};


async fn call_rest_get_client_account<
    AccountS: AccountService + 'static,
> (
    State(rest_service): State<Arc<AccountRest<AccountS>>>,
    Path(path::ClientId { client_id }): Path<path::ClientId>,
    Path(path::AccountId { account_id }): Path<path::AccountId>,
) -> Result<Json<dto::Account>, RestAppError> {
    rest_service.get_account(client_id, account_id).to_json().await
}

async fn call_rest_get_client_accounts <
    AccountS: AccountService + 'static,
> (
    State(rest_service): State<Arc<AccountRest<AccountS>>>,
    Path(path::ClientId { client_id }): Path<path::ClientId>,
)
    -> Result<Json<Vec<dto::Account>>, RestAppError> {
    rest_service.get_accounts(client_id).to_json().await
}


// It will be 'current user' in wab-app.
// pub struct CurrentUserAccountRest <AS: AccountService> {

pub struct AccountRest <AS: AccountService> {
    pub account_service: Arc<AS>,
}

impl<AS: AccountService> AccountRest<AS> {

    #[tracing::instrument( skip(self) )]
    pub async fn get_account(&self, client_id: String, account_id: String) -> Result<dto::Account, RestAppError> {

        debug!("TD get_user_account as debug");
        info! ("TI get_user_account as info");
        // error!("TI get_user_account as error");

        log_debug!("LD get_user_account as debug");
        log_info! ("LI get_user_account as info");
        // log_error!("LE get_user_account as error");

        let is_internal_account_id = account_id.len().is_one_of2(36, 38);
        let account = if is_internal_account_id {
            self.account_service.get_client_account_by_id(
                ClientId::from_str(&client_id) ?,
                AccountId::from_str(&account_id) ?,
            ).await ?
        } else {
            use core::str::FromStr;
            self.account_service.get_client_account_by_iban(
                ClientId::from_str(&client_id) ?,
                iban::Iban::from_str(&account_id) ?,
            ).await ?
        };

        Ok(map_account_to_rest(account))
    }


    #[tracing::instrument( skip(self) )]
    pub async fn get_accounts(&self, client_id: String) -> Result<Vec<dto::Account>, RestAppError> {
        let accounts = self.account_service.get_client_accounts(
            ClientId::from_str(&client_id) ?,
        ).await;

        let accounts_dto = accounts
            .map( move |acs| acs.into_iter()
                .map(move |ac| map_account_to_rest(ac))
                .collect::<Vec<_>>()
            ) ?;
        Ok(accounts_dto)
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
// use axum_valid::Valid as Valid;
use mvv_common::mvv_axum_valid::Valid as Valid;
async fn call_rest_input_validate_by_validator <
    AccountS: AccountService + 'static,
>(State(_rest_service): State<Arc<AccountRest<AccountS>>>, Valid(Json(input)): Valid<Json<ValidatedInput1>>)
  -> Result<Json<&'static str>, RestAppError> {
    info!("call_rest_input_validate: input = {:?}", input);

    // rest_service.input_validate().to_json().await
    Ok(Json("Ok_1"))
}
use validator::Validate;
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

// use axum_valid::{ Validified, /*Modified,*/ };
use mvv_common::mvv_axum_valid::{ Validified, /*Modified,*/ };
use mvv_common::obj_ext::ValExt;

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