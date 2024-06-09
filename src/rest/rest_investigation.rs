#![allow(dead_code)]


use std::sync::Arc;
use axum::body::Body;
use axum::http::{StatusCode, Uri};
use axum::Json;
// use axum::routing::{ delete, get, post };
use axum::routing::{ get };
use super::dto::{ Account as AccountDTO };
use crate::rest::account_rest::AccountRest;
use crate::rest::error_rest::AppRestError;
use crate::service::account_service::{ AccountService };



// struct AppState {
//     // ...
// }
//
// async fn rest_handler <
//     AccountS: AccountService + Send + Sync + 'static,
//     // AccountR: AccountRest<AccountS> + Send + Sync,
//     RT: Debug + Display,
//     F, // : Fn(&AccountRest<AccountS>)-> impl Future<Output = Result<RT, AppRestError>>,
// > (
//     axum::extract::State(_state): axum::extract::State<Arc<AccountRest<AccountS>>>,
// ) -> &'static str
//     // where F: Fn(&AccountRest<AccountS>)-> impl Future<Output = Result<RT, AppRestError>>
//     where F: Fn(&AccountRest<AccountS>)-> impl Future<Output = & 'static str>
// {
//     // ...
//     //"Hello, World!"
//     t o d o!()
// }


async fn handler(
    axum::extract::State(_state): axum::extract::State<Arc<AppState>>,
) {
    // ...
}
async fn handler2 <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
> (
    axum::extract::State(_state): axum::extract::State<Arc<AccountRest<AccountS>>>,
) -> &'static str {
    // ...
    "Hello, World!"
}

async fn handler3 <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
>(
    axum::extract::State(_state): axum::extract::State<Arc<AccountRest<AccountS>>>,
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


#[derive(serde::Deserialize)]
struct Form22 {
    field1: Option<String>,
    field2: Option<String>,
}

async fn handler5 <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
>(
    axum::extract::State(_state): axum::extract::State<Arc<AccountRest<AccountS>>>,
    //axum::extract::Query()
    _uri: Uri,
    axum::extract::OriginalUri(_original_uri): axum::extract::OriginalUri,
    _path: axum::extract::NestedPath,
    axum::extract::RawQuery(_query): axum::extract::RawQuery,
    axum::extract::RawForm(_raw_form): axum::extract::RawForm,
    axum::extract::Host(_host): axum::extract::Host,
    axum::extract::Form(_form): axum::extract::Form<Form22>,
    _request: axum::extract::Request<Body>,

    // Custom extractor
    //  https://dev.to/pongsakornsemsuwan/rust-axum-extracting-query-param-of-vec-4pdm


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
// ) -> Json<AccountDTO> {
// ) -> Result<Json<AccountDTO>, anyhow::Error> {
) -> Result<Json<AccountDTO>, AppRestError> {
// ) -> Result<axum::response::Response, anyhow::Error> {
// ) -> Result<Json< crate::rest::dto::Account >, anyhow::Error> {
    let aa: Result<(), anyhow::Error> = Ok(());
    let _aa2 = aa ?;
    let ac: AccountDTO = state.get_user_account("678".to_string()).await.unwrap();
    Ok(Json(ac))

    // Ok(Json(ac))
    // Ok::<_, anyhow::Error>(axum::response::Response::new(Body::empty()))
}

// TODO: error processing
//       * https://docs.rs/axum/latest/axum/error_handling/
//       * https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs

async fn handler7 <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
>(
    axum::extract::State(state): axum::extract::State<Arc<AccountRest<AccountS>>>,
) -> Json<Vec<AccountDTO>> {
    let ac = state.get_current_user_accounts().await.unwrap();
    Json(ac)
}

async fn handler8() -> Result<String, StatusCode> {
    // ...
    todo!()
}

struct AppState {
    // ...
}


pub struct Dependencies <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
> {
    pub account_service: Arc<AccountS>,
    pub account_rest: Arc<AccountRest<AccountS>>,
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

    // type AA = State<Arc<AccountRest<AccountS>>>;
    // type AA2 = Arc<AccountRest<AccountS>>;
    let shared_state: Arc<AccountRest<AccountS>> = Arc::clone(&dependencies.account_rest);

    let accounts_router = Router::new()
        .route("current_user/account/all", get(|State(_state): State<Arc<AccountRest<AccountS>>>| async move {
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
        .route("current_user/account/all8", get(handler8))
        .route("current_user/account/id33", get(|State(state): State<Arc<AccountRest<AccountS>>>| async move {
            Json(state.get_current_user_accounts().await.unwrap())
            // state.get_current_user_accounts()
        }))
        .route("current_user/account/:id", get(|State(state): State<Arc<AccountRest<AccountS>>>,
                                                axum::extract::Path(id): axum::extract::Path<String>,
                                                pagination: Option<axum::extract::Query<Pagination>>,
        | async move {
            let axum::extract::Query(_pagination) = pagination.unwrap_or_default();
            Json(state.get_user_account(id).await.unwrap())
            // state.get_current_user_accounts()
        }))
        // .route("account/{id}", get(|State(state): State<Arc<AccountRest<AccountS>>>| async {
        //     state.get_current_user_account("666")
        // }))
        .with_state(shared_state)
        ;

    accounts_router
}

#[derive(serde::Deserialize)]
struct Pagination {
    page: usize,
    per_page: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self { page: 1, per_page: 30 }
    }
}


// -------------------------------------------------------------------------------

use std::future::Future;

trait AsyncFnOnce0 {
    type Output;

    async fn async_call(self) -> Self::Output;
}

impl<F, Fut> AsyncFnOnce0 for F
where F: FnOnce() -> Fut,
      Fut: Future,
{
    type Output = Fut::Output;

    async fn async_call(self) -> Self::Output {
        (self)().await
    }
}

fn bar() -> impl AsyncFnOnce0<Output = impl AsyncFnOnce0<Output = u32>> {
    || async {
        || async {
            42
        }
    }
}

async fn use_bar() {
    assert_eq!(bar().async_call().await.async_call().await, 42);
}

//-----------------------------------------------------------------------------------

// async fn rest_handler <
//     AccountS: AccountService + Send + Sync + 'static,
//     // AccountR: AccountRest<AccountS> + Send + Sync,
//     RT: Debug + Display,
//     F, // : Fn(&AccountRest<AccountS>)-> impl Future<Output = Result<RT, AppRestError>>,
// > (
//     axum::extract::State(_state): axum::extract::State<Arc<AccountRest<AccountS>>>,
// ) -> &'static str
//     where F: Fn(&AccountRest<AccountS>)-> impl MyAsyncFn0< Result<RT, AppRestError>>
// {
//     // ...
//     //"Hello, World!"
//     todo!()
// }


// fn bar33() -> impl MyAsyncFn0<i32, impl MyAsyncFn0<i32, u32>> {
//     |v_i32| async {
//         |v_i32| async {
//             42
//         }
//     }
// }
// fn bar33() -> impl MyAsyncFn0<i32, u32> {
//     |v_i32| async {
//         42
//     }
// }
//
// async fn use_bar33() {
//     assert_eq!(bar().async_call().await.async_call().await, 42);
// }

// async fn rest_handler33 <
//     AccountS: AccountService + Send + Sync + 'static,
//     // AccountR: AccountRest<AccountS> + Send + Sync,
//     RT: Debug + Display,
//     F, // : Fn(&AccountRest<AccountS>)-> impl Future<Output = Result<RT, AppRestError>>,
// > (
//     axum::extract::State(_state): axum::extract::State<Arc<AccountRest<AccountS>>>,
// ) -> &'static str
//     // where F: Fn(&AccountRest<AccountS>)-> impl Future<Output = Result<RT, AppRestError>>
//     where F: Fn(&AccountRest<AccountS>)-> impl Future<Output = & 'static str>
// {
//     // ...
//     //"Hello, World!"
//     todo!()
// }
