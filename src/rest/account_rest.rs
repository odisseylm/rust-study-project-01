use std::sync::Arc;
// use axum::routing::{ delete, get, post };
use axum::routing::{ get };
use crate::entities::account::AccountId;
use crate::entities::prelude::UserId;
use crate::util::UncheckedResultUnwrap;
use crate::entities::entity;
use crate::rest::app_dependencies::Dependencies;
use crate::rest::dto;
use crate::rest::error_rest::AppRestError;
use crate::service::account_service::{ AccountService };



// fn accounts_rest_router<AccountS: crate::service::account_service::AccountService + Send + Sync>() -> axum::Router {
pub fn accounts_rest_router<
    AccountS: AccountService + Send + Sync + 'static,
    >(
    dependencies: Dependencies<AccountS>,
    ) -> axum::Router {

    use axum::Router;
    use axum::extract::{ Path, State };
    use std::sync::Arc;
    use super::utils::RestFutureToJson;

    let shared_state: Arc<AccountRest<AccountS>> = Arc::clone(&dependencies.account_rest);

    let accounts_router = Router::new()
        .route("/api/account/all", get(|State(state): State<Arc<AccountRest<AccountS>>>| async move {
            state.get_current_user_accounts().to_json().await
        }))
        .route("/api/account/:id", get(|State(state): State<Arc<AccountRest<AccountS>>>, Path(id): Path<String>| async move {
            state.get_user_account(id).to_json().await
        }))
        .with_state(shared_state)
        ;

    accounts_router
}

// #[static_init::constructor]
#[static_init::dynamic]
static TEMP_CURRENT_USER_ID: UserId = UserId::from_str("11").unchecked_unwrap();

pub struct AccountRest <AS: AccountService> {
    pub account_service: Arc<AS>,
}



impl<AS: AccountService> AccountRest<AS> {
    async fn current_user_id(&self) -> UserId {
        TEMP_CURRENT_USER_ID.clone()
    }

    pub async fn get_user_account(&self, account_id: String) -> Result<dto::Account, AppRestError> {
        let current_user_id = self.current_user_id().await;
        let account_id = AccountId::from_str(account_id.as_str()) ?;
        let account = self.account_service.get_user_account(account_id, current_user_id).await ?;
        let account_dto = map_account_to_rest(account);
        Ok(account_dto)
    }

    pub async fn get_current_user_accounts(&self) -> Result<Vec<dto::Account>, AppRestError> {
        let current_user_id = self.current_user_id().await;
        let accounts = self.account_service.get_user_accounts(current_user_id).await;
        let accounts_dto = accounts.map(move|acs|acs.into_iter().map(move |ac| map_account_to_rest(ac)).collect::<Vec<_>>()) ?;
        Ok(accounts_dto)
    }
}

/*
async fn handler_get_user_account <
    AccountS: AccountService + Send + Sync + 'static,
>(
    State(state): State<Arc<AccountRest<AccountS>>>,
    Path(id): Path<String>
) -> Result<Json<AccountDTO>, AppRestError> {
    state.get_user_account(id).rest_to_json().await
}
*/


#[allow(dead_code)] // !!! It is really used ?!
fn map_account_to_rest(account: entity::Account) -> dto::Account {
    dto::Account {
        id: account.id.to_string(), // TODO: use moving
        user_id: account.user_id.to_string(),
        amount: dto::Amount { value: account.amount.value.clone(), currency: account.amount.currency.to_string() },
        created_at: account.created_at,
        updated_at: account.updated_at,
    }
}

// async fn rest_to_json33<
//     F: Future<Output = Result<Vec<dto::Account>, AppRestError>>
//     >
//     (fut: F) -> impl Future<Output = Result<Json<Vec<dto::Account>>, AppRestError>>
//     //where F: impl Future<Output = Result<Vec<dto::Account>, AppRestError>>
//     {
//         async { fut.await.map(|data|Json(data)) }
// }


// fn dsdsd<
//     AccountS: AccountService + Send + Sync + 'static,
//     >() {
//     use crate::rest::dto;
//     let aa: impl Future<Output = Result<Vec<dto::Account>, anyhow::Error>> = AccountRest::<AccountS>::get_current_user_accounts;
// }

//
// fn post_foo_04() -> impl Future<Output = & 'static str> { async { "POST foo" } }
// fn post_foo_042() -> impl Future<Output = & 'static str> { async { t o d o!() } }
//
//
// // use crate::rest::dto;
// fn dsds() -> impl Future<Output = Result<Vec<dto::Account>, anyhow::Error>> {
//     async { t o d o!() }
// }
//
// fn hhh123<
//     F: Fn(i32) -> impl Future<Output = Result<Vec<dto::Account>, anyhow::Error>>
//     >
//     (f: F) -> impl Future<Output = Result<Vec<dto::Account>, anyhow::Error>> {
//     async { f(123).await }
// }

