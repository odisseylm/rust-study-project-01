use std::future::Future;
use anyhow::Error;
use axum::Json;
use axum::routing::{delete, get, post};
use crate::entities::account::AccountId;
use crate::entities::prelude::UserId;
use crate::util::UncheckedResultUnwrap;
use super::dto::{Account as AccountDTO, Account, Amount};
use crate::entities::prelude::{ Account as AccountEntity };


async fn handler(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<AppState>>,
) {
    // ...
}

struct AppState {
    // ...
}

use crate::service::account_service::AccountService;

pub struct Dependencies <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
> {
    pub account_service: std::sync::Arc<AccountS>,
    pub account_rest: std::sync::Arc<AccountRest<AccountS>>,
}


// fn accounts_rest_router<AccountS: crate::service::account_service::AccountService + Send + Sync>() -> axum::Router {
fn accounts_rest_router<
    AccountS: AccountService + Send + Sync + 'static,
    >(
    dependencies: Dependencies<AccountS>,
    ) -> axum::Router {
    use axum::Router;
    use axum::extract::State;
    use std::sync::Arc;

    /*
    let shared_state = std::sync::Arc::new(AppState { /* ... */ });

    let accounts_router = Router::new()
        // .route("account/{id}", get(get_root))
        // .route("user/account/{id}", post(post_root))
        // .route("user/account/", post(post_root))
        // .route("account", delete(delete_root))

        // .route("/", get(handler)).with_state(shared_state)

        // .route("/", get(|State(state): State<AppState>| async {
        // .route("/", get(|_: State<AppState>| async {
        .route("/", get(|State(state): State<std::sync::Arc<AppState>>| async {
            // use state
        })).with_state(shared_state)
        ;
     */

    let shared_state: Arc<AccountRest<AccountS>> = Arc::clone(&dependencies.account_rest);

    let accounts_router = Router::new()
        .route("current_user/account/all", get(|State(state): State<Arc<AccountRest<AccountS>>>| async move {
            "Hello, World!"
            // Json(state.get_current_user_accounts())
            // let accounts_r = state.get_current_user_accounts().await;
            // Json(accounts_r)
            // Json(accounts_r.unwrap())
            // match accounts_r {
            //     Ok(accounts) => { Json(accounts) }
            //     Err(_) => { Json("Error") }
            // }
            // Json("Hello, World!")
        }))
        .route("current_user/account/{id}", get(|State(state): State<Arc<AccountRest<AccountS>>>| async {
            // state.get_current_user_account("666")
            // state.get_current_user_accounts()
        }))
        // .route("account/{id}", get(|State(state): State<Arc<AccountRest<AccountS>>>| async {
        //     state.get_current_user_account("666")
        // }))
        .with_state(shared_state)
        ;

    accounts_router
}


// #[static_init::constructor]
#[static_init::dynamic]
static TEMP_CURRENT_USER_ID: UserId = UserId::from_str("11").unchecked_unwrap();

struct AccountRest <AS: AccountService> {
    account_service: AS,
}

impl<AS: AccountService> AccountRest<AS> {
    async fn current_user_id(&self) -> UserId {
        TEMP_CURRENT_USER_ID.clone()
    }

    pub async fn get_user_account(&self, account_id: String) -> Result<AccountDTO, anyhow::Error> {
        let current_user_id = self.current_user_id().await;
        let account_id = AccountId::from_str(account_id.as_str())?;
        let account = self.account_service.get_user_account(account_id, current_user_id).await ?;
        let account_dto = map_account_to_rest(account);
        Ok(account_dto)
    }

    pub async fn get_current_user_accounts(&self) -> Result<Vec<AccountDTO>, anyhow::Error> {
        let current_user_id = self.current_user_id().await;
        let mut accounts = self.account_service.get_user_accounts(current_user_id).await;
        let accounts_dto = accounts.map(move|acs|acs.into_iter().map(move |ac| map_account_to_rest(ac)).collect::<Vec<_>>()) ?;
        Ok(accounts_dto)
    }
}


fn map_account_to_rest(account: AccountEntity) -> AccountDTO {
    AccountDTO {
        id: account.id.to_string(), // TODO: use moving
        user_id: account.user_id.to_string(),
        amount: Amount { value: account.amount.value.clone(), currency: account.amount.currency.to_string() },
        created_at: account.created_at,
        updated_at: account.updated_at,
    }
}