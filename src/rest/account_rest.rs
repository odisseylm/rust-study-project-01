use std::sync::Arc;
use axum::Json;
use axum::routing::{ delete, get, post };
use crate::entities::account::AccountId;
use crate::entities::prelude::UserId;
use crate::util::UncheckedResultUnwrap;
use super::dto::{ Account as AccountDTO, Amount as AmountDTO };
use crate::entities::prelude::{ Account as AccountEntity };
// use crate::service::account_service::{ AccountServiceSafe as AccountService };
use crate::service::account_service::{ AccountService };


async fn handler(
    axum::extract::State(_state): axum::extract::State<Arc<AppState>>,
) {
    // ...
}
async fn handler2 <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
>(
    axum::extract::State(_state): axum::extract::State<Arc<AccountRest<AccountS>>>,
) -> &'static str {
    // ...
    "Hello, World!"
}

async fn handler3 <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
>(
    axum::extract::State(state): axum::extract::State<Arc<AccountRest<AccountS>>>,
) -> String {
    // ...
    "Hello, World!".to_string()
}

async fn handler4 <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
>(
    axum::extract::State(_state): axum::extract::State<Arc<AccountRest<AccountS>>>,
) -> Json<String> {
    // ...
    Json("Hello, World!".to_string())
}

async fn handler5 <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
>(
    axum::extract::State(state): axum::extract::State<Arc<AccountRest<AccountS>>>,
) -> Json<AccountDTO> {
    // ...
    //Json("Hello, World!".to_string())
    todo!()
}

async fn handler6 <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
>(
    axum::extract::State(state): axum::extract::State<Arc<AccountRest<AccountS>>>,
) -> Json<AccountDTO> {
    let ac: AccountDTO = state.get_user_account("678".to_string()).await.unwrap();
    Json(ac)
}

async fn handler7 <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
>(
    axum::extract::State(state): axum::extract::State<Arc<AccountRest<AccountS>>>,
) -> Json<Vec<AccountDTO>> {
    let ac = state.get_current_user_accounts().await.unwrap();
    Json(ac)
}

struct AppState {
    // ...
}


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
            // "Hello, World!"
            // Json(state.get_current_user_accounts())
            // let accounts_r = state.get_current_user_accounts().await;
            // Json(accounts_r)
            // Json(accounts_r.unwrap())
            // match accounts_r {
            //     Ok(accounts) => { Json(accounts) }
            //     Err(_) => { Json("Error") }
            // }
            // Json("Hello, World!")
            "Hello, World!"
        }))
        // .route("current_user/account/all", get(|State(state): State<Arc<AccountRest<AccountS>>>| async {
        //     // "Hello, World!"
        //     // Json(state.get_current_user_accounts())
        //     let accounts_r: Vec<AccountDTO> = state.get_current_user_accounts().await.unwrap();
        //     Json(accounts_r.first().unwrap())
        //     // Json(accounts_r)
        //     // Json(accounts_r.unwrap())
        //     // match accounts_r {
        //     //     Ok(accounts) => { Json(accounts) }
        //     //     Err(_) => { Json("Error") }
        //     // }
        //     // Json("Hello, World!")
        //     // "Hello, World!"
        // }))
        .route("current_user/account/all2", get(handler2))
        .route("current_user/account/all3", get(handler3))
        .route("current_user/account/all4", get(handler4))
        // .route("current_user/account/all5", get(handler5))
        .route("current_user/account/all6", get(handler6))
        .route("current_user/account/all7", get(handler7))
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
        amount: AmountDTO { value: account.amount.value.clone(), currency: account.amount.currency.to_string() },
        created_at: account.created_at,
        updated_at: account.updated_at,
    }
}