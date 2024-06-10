use std::sync::Arc;
use axum::{routing::get, Extension, Router, Json};
use axum::extract::State;
use tower_http::{trace::TraceLayer};
use tower::ServiceBuilder;
use crate::rest::rest_auth::{ auth_manager_layer, AuthnBackend0, Cred0 };


async fn is_authenticated_by_session_or_basic_auth(auth_session: axum_login::AuthSession<AuthnBackend0>,
                            basic_auth_creds: Option<axum_extra::TypedHeader<axum_extra::headers::Authorization<axum_extra::headers::authorization::Basic>>>) -> bool {
    use axum_extra::headers::Authorization;
    use axum_extra::TypedHeader;

    if auth_session.user.is_some() {
        return true;
    }

    if let Some(TypedHeader(Authorization(ref creds))) = basic_auth_creds {
        // T O D O: avoid to_string()
        let is_auth_res = auth_session.authenticate(Cred0 { username: creds.username().to_string(), password: creds.password().to_string() }).await;
        is_auth_res.is_ok()
    }
    else { false }
}


macro_rules! predicate_required22 {
    ($predicate:expr, $alternative:expr) => {{
        use axum::middleware::{ from_fn, Next };
        use axum::response::IntoResponse;
        use axum_extra::TypedHeader;
        use axum_extra::headers::{ authorization::Basic, Authorization};

        axum::middleware::from_fn(|
                auth_session: axum_login::AuthSession<_>,
                basic_auth_creds: Option<TypedHeader<Authorization<Basic>>>,
                req, next: Next
            | async move {
                if is_authenticated_by_session_or_basic_auth(auth_session, basic_auth_creds).await {
                    next.run(req).await
                } else {
                    $alternative.into_response()
                }
            },
        )
    }};
}



#[macro_export]
macro_rules! login_required22 {
    ($backend_type:ty) => {{
        async fn is_authenticated(auth_session: axum_login::AuthSession<$backend_type>) -> bool {
            auth_session.user.is_some()
        }

        predicate_required22!(
            is_authenticated,
            $crate::rest::error_rest::unauthenticated_401_response()
        )
    }};
}


async fn handler(state22: State<State22>) {}
async fn handler_1_protected_with_state(state: Extension<Arc<State22>>, auth_session: axum_login::AuthSession<AuthnBackend0>) -> Json<String> {
    println!("### handler1 PROTECTED with_state, state: {}", state.x);
    // ...
    Json("bla-bla 22".to_string())
}
async fn handler_2_open_with_state(state: Extension<Arc<State22>>, auth_session: axum_login::AuthSession<AuthnBackend0>) -> Json<String> {
    println!("### handler2 OPEN state, state: {}", state.x);
    // ...
    Json("bla-bla 23".to_string())
}
async fn handler_3_protected_with_state(state: Extension<Arc<State22>>, auth_session: axum_login::AuthSession<AuthnBackend0>) -> Json<String> {
    println!("### handler3 PROTECTED with_state, state: {}", state.x);
    // ...
    Json("bla-bla 22".to_string())
}
async fn handler_4_open_with_state(state: Extension<Arc<State22>>, auth_session: axum_login::AuthSession<AuthnBackend0>) -> Json<String> {
    println!("### handler4 OPEN with_state, state: {}", state.x);
    // ...
    Json("bla-bla 23".to_string())
}
async fn handler22(auth_session: axum_login::AuthSession<AuthnBackend0>) -> Json<String> {
    println!("### handler22");
    // ...
    Json("bla-bla 22".to_string())
}


#[derive(Clone)]
struct State22 {
    x: &'static str,
}

// use axum::middleware::Next;
// use axum::response::Response;
// use axum::extract::Request;
// use axum::http::StatusCode;


async fn temp_my_middleware(req: axum::extract::Request, next: axum::middleware::Next) -> Result<axum::response::Response, (axum::http::StatusCode, &'static str)> {
    println!("### temp_my_middleware");
    let response = next.run(req).await;
    Ok(response)

    // match has_permission(&req) {
    //     Ok(_) => {
    //         let response = next.run(req).await;
    //         Ok(response)
    //     }
    //     Err(_) => Err((StatusCode::BAD_REQUEST, "bad request")),
    // }
}

// let app = Router::new()
// ...
// .layer(axum::middleware::from_fn(auth_middleware))



pub async fn temp_handler() {

    /*
    let app = protected::router()
        .route_layer(login_required!(Backend, login_url = "/login"))
        .merge(auth::router())
        .merge(oauth::router())
        .layer(auth_layer);
    */


    let auth_layer: axum_login::AuthManagerLayer<AuthnBackend0, axum_login::tower_sessions::MemoryStore> = auth_manager_layer();

    // !!! WORKING router !!!
    // let app_router = Router::new()
    //     .route("/temp_handler", get(handler22_with_state))
    //     .route_layer(login_required!(AuthnBackend0, login_url = "/login"))
    //     // .layer(axum::middleware::from_fn(temp_my_middleware))
    //     .layer(Extension(Arc::new(State22 { x: "963" })))
    //     .layer(auth_layer);

    // Also working
    // let app_router = Router::new()
    //     .route("/temp_handler", get(handler22_with_state))
    //     .route_layer(login_required!(AuthnBackend0, login_url = "/login"))
    //     .layer(
    //         ServiceBuilder::new()
    //             .layer(auth_layer)
    //             .layer(Extension(Arc::new(State22 { x: "963" })))
    //     );

    // Also working
    // let app_router = Router::new()
    //     .route("/temp_handler", get(handler22_with_state))
    //     .route_layer(login_required!(AuthnBackend0, login_url = "/login"))
    //     .layer(
    //         ServiceBuilder::new()
    //             .layer(TraceLayer::new_for_http())
    //             .layer(axum::middleware::from_fn(temp_my_middleware))
    //             .layer(auth_layer)
    //             .layer(Extension(Arc::new(State22 { x: "963" })))
    //     );

    let app_router = Router::new()
        .route("/temp_handler1", get(handler_1_protected_with_state))
        .route("/temp_handler3", get(handler_3_protected_with_state))
        // Note that the middleware is only applied to existing routes. So you have to
        // first add your routes (and / or fallback) and then call `route_layer`
        // afterwards. Additional routes added after `route_layer` is called will not have
        // the middleware added.
        // .route_layer(login_required!(AuthnBackend0, login_url = "/login"))
        // .route_layer(login_required!(AuthnBackend0))
        .route_layer(login_required22!(AuthnBackend0))

        .route("/temp_handler2", get(handler_2_open_with_state))
        .route("/temp_handler4", get(handler_4_open_with_state))

        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn(temp_my_middleware))
                .layer(auth_layer)
                .layer(Extension(Arc::new(State22 { x: "963" })))
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    // axum::serve(listener, app_router).await.unwrap();
    axum::serve(listener, app_router.into_make_service()).await.unwrap();
    // axum::serve(listener, app_router.into_make_service()).await.unwrap();
}


// use axum::middleware::Next;
// use axum::response::Response;
// use axum::extract::Request;
// use axum::http::StatusCode;
//
// async fn auth_middleware(req: Request, next: Next) -> Result<Response, (StatusCode, &'static str)> {
//     match has_permission(&req) {
//         Ok(_) => {
//             let response = next.run(req).await;
//             Ok(response)
//         }
//         Err(_) => Err((StatusCode::BAD_REQUEST, "bad request")),
//     }
// }
//
// let app = Router::new()
// ...
// .layer(axum::middleware::from_fn(auth_middleware))
// ...
