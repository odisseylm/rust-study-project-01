use std::sync::Arc;
use axum::{
    Router, Json, routing::get as GET, extract::{ Path, State, },
};
use tracing::{ debug, info /*, error*/ };
use log::{ debug as log_debug, info as log_info /*, error as log_error*/ };

use crate::{
    entities::{ prelude::UserId, entity, account::AccountId, },
    rest::{
        auth::{ RequiredAuthorizationExtension, Role, },
        app_dependencies::Dependencies,
        dto, error_rest::{ RestAppError },
        utils::RestFutureToJson,
    },
    service::{ account_service::{ AccountService }, },
};
use super::path;
//--------------------------------------------------------------------------------------------------


pub fn accounts_rest_router <
    AccountS: AccountService + Send + Sync + 'static,
> (
    dependencies: Dependencies<AccountS>,
) -> Router {

    let shared_state: Arc<CurrentUserAccountRest<AccountS>> = Arc::clone(&dependencies.account_rest);

    let accounts_router = Router::new()
        .route("/account/all", GET(call_rest_get_user_accounts::<AccountS>))
        .route("/account/:account_id", GET(call_rest_get_user_account::<AccountS>))
        .with_state(shared_state.clone())
        .role_required(Role::Read)
        ;

    accounts_router
}

// #[static_init::constructor]
#[static_init::dynamic]
static TEMP_CURRENT_USER_ID: UserId = {
    use crate::util::UncheckedResultUnwrap;
    UserId::from_str("11").unchecked_unwrap()
};


async fn call_rest_get_user_account <
    AccountS: AccountService + 'static,
>(State(rest_service): State<Arc<CurrentUserAccountRest<AccountS>>>, Path(path::AccountId { account_id }): Path<path::AccountId>)
    -> Result<Json<dto::Account>, RestAppError> {
    rest_service.get_user_account(account_id).to_json().await
}

async fn call_rest_get_user_accounts <
    AccountS: AccountService + 'static,
>(State(rest_service): State<Arc<CurrentUserAccountRest<AccountS>>>)
    -> Result<Json<Vec<dto::Account>>, RestAppError> {
    rest_service.get_user_accounts().to_json().await
}


pub struct CurrentUserAccountRest <AS: AccountService> {
    pub account_service: Arc<AS>,
}

impl<AS: AccountService> CurrentUserAccountRest<AS> {
    async fn current_user_id(&self) -> UserId {
        TEMP_CURRENT_USER_ID.clone()
    }

    #[tracing::instrument(
        // skip(dependencies),
        skip(self),
    )]
    pub async fn get_user_account(&self, account_id: String) -> Result<dto::Account, RestAppError> {

        debug!("TD get_user_account as debug");
        info! ("TI get_user_account as info");
        // error!("TI get_user_account as error");

        log_debug!("LD get_user_account as debug");
        log_info! ("LI get_user_account as info");
        // log_error!("LE get_user_account as error");

        let current_user_id = self.current_user_id().await;
        let account_id = AccountId::from_str(account_id.as_str()) ?;
        let account = self.account_service.get_user_account(account_id, current_user_id).await ?;
        let account_dto = map_account_to_rest(account);
        Ok(account_dto)
    }

    pub async fn get_user_accounts(&self) -> Result<Vec<dto::Account>, RestAppError> {
        let current_user_id = self.current_user_id().await;
        let accounts = self.account_service.get_user_accounts(current_user_id).await;
        let accounts_dto = accounts.map(move|acs|acs.into_iter().map(move |ac| map_account_to_rest(ac)).collect::<Vec<_>>()) ?;
        Ok(accounts_dto)
    }
}


fn map_account_to_rest(account: entity::Account) -> dto::Account {
    use crate::entities::account::AccountParts;
    use crate::entities::amount::AmountParts;

    let AccountParts { id, user_id, amount, created_at, updated_at } = account.move_out();
    let AmountParts { value: amount_value, currency } = amount.move_out();
    dto::Account {
        id: id.move_string_out(),
        user_id: user_id.move_string_out(),
        amount: dto::Amount { value: amount_value, currency: currency.to_string() },
        created_at,
        updated_at,
    }
}
