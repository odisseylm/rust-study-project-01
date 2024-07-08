#![allow(unused_imports, unused)]

use core::convert::Infallible;
use std::sync::Arc;
use axum::{routing::get, Extension, Router, Json};
use axum::body::Body;
use axum::extract::{Request, State};
use axum::handler::Handler;
use axum::middleware::FromFnLayer;
use axum::response::Response;
use axum::routing::any_service;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Basic;
use axum_extra::TypedHeader;
use tower_http::{trace::TraceLayer};
use tower::{ service_fn, ServiceBuilder };
use super::super::auth::CompositeAuthBackend;
use mvv_auth::examples::composite_auth::CompositeAuthnBackendExample;
use mvv_auth::route::validate_authentication_chain;
use crate::rest::auth::{
    RequiredAuthenticationExtension,
    auth_layer::composite_auth_manager_layer,
};
use crate::util::TestResultUnwrap;
use super::super::auth;

type AuthSession = axum_login::AuthSession<CompositeAuthnBackendExample>;

#[inline]
pub async fn validate_auth_temp(
    auth_session: AuthSession,
    req: Request,
    next: axum::middleware::Next,
) -> http::Response<Body> {
    validate_authentication_chain(auth_session, req, next).await
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



//#[macro_export]
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

macro_rules! login_required23 {
    ($backend_type:ty) => {{

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
                    // or redirect to login page
                    $crate::rest::error_rest::unauthenticated_401_response()
                }
            },
        )
    }};
}


async fn handler(state22: State<State22>) {}
async fn handler_1_protected_with_state(state: Extension<Arc<State22>>, auth_session: AuthSession) -> Json<&'static str> {
    println!("### handler1 PROTECTED with_state, state: {}", state.x);
    // ...
    Json("bla-bla 22")
}
async fn handler_2_open_with_state(state: Extension<Arc<State22>>, _auth_session: AuthSession) -> Json<&'static str> {
    println!("### handler2 OPEN state, state: {}", state.x);
    // ...
    Json("bla-bla 23")
}
async fn handler_3_protected_with_state(state: State<State11>, _auth_session: AuthSession) -> Json<&'static str> {
    println!("### handler3 PROTECTED with_state, state11: {}", state.x);
    // ...
    Json("bla-bla 22")
}
async fn handler_4_open_with_state(state: Extension<Arc<State22>>, _auth_session: AuthSession) -> Json<&'static str> {
    println!("### handler4 OPEN with_state, state: {}", state.x);
    // ...
    Json("bla-bla 23")
}
async fn handler22(auth_session: AuthSession) -> Json<&'static str> {
    println!("### handler22");
    // ...
    Json("bla-bla 22")
}


#[derive(Clone)]
struct State22 {
    x: &'static str,
}
#[derive(Clone)]
struct State11 {
    x: &'static str,
}

// use axum::middleware::Next;
// use axum::response::Response;
// use axum::extract::Request;
// use axum::http::StatusCode;


async fn temp_my_middleware(req: Request, next: axum::middleware::Next) -> Result<Response, (http::StatusCode, &'static str)> {
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

    use crate::rest::auth::user_perm_provider::in_memory_test_users;

    let auth_layer: axum_login::AuthManagerLayer<CompositeAuthBackend, axum_login::tower_sessions::MemoryStore> =
        composite_auth_manager_layer(Arc::new(in_memory_test_users().test_unwrap())).await.test_unwrap();

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
        .merge(
            Router::new()
                .route("/temp_handler3", get(handler_3_protected_with_state))
                .with_state(State11 { x: "101" })
                .route_layer(axum::middleware::from_fn(validate_auth_temp))
        )
        .nest(
            "/also-protected",
            Router::new()
                .route("/temp_handler3", get(handler_3_protected_with_state))
                .with_state(State11 { x: "101" })
                //.route_layer(axum::middleware::from_fn(validate_auth))
                .authn_required()
        )
        .nest(
            "/not-protected",
            Router::new()
                .route("/temp_handler3", get(handler_3_protected_with_state))
                .with_state(State11 { x: "101" })
        )

        // .route("/temp_handler3", get(handler_3_protected_with_state))
        // // .with_state(Arc::new(State11 { x: "101" }))
        // .with_state(State11 { x: "101" })
        // // Note that the middleware is only applied to existing routes. So you have to
        // // first add your routes (and / or fallback) and then call `route_layer`
        // // afterwards. Additional routes added after `route_layer` is called will not have
        // // the middleware added.
        // // .route_layer(login_required!(AuthnBackend0, login_url = "/login"))
        // // .route_layer(login_required!(AuthnBackend0))
        //
        // // .route_layer(login_required22!(AuthnBackend0))
        // // .route_layer(login_required23!(AuthnBackend0))
        // .route_layer(axum::middleware::from_fn(validate_auth))

        .route("/temp_handler2", get(handler_2_open_with_state))
        .route("/temp_handler4", get(handler_4_open_with_state))

        // .route("/temp_handler4", get(handler_4_open_with_state))
        .route(
            // Any request to `/` goes to a service
            "/temp_handler5",
            // Services whose response body is not `axum::body::BoxBody`
            // can be wrapped in `axum::routing::any_service` (or one of the other routing filters)
            // to have the response body mapped
            any_service(service_fn(|_: Request| async {
                let res = Response::new(Body::from("Hi from `GET /`"));
                Ok::<_, Infallible>(res)
            }))
        )


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
