use core::future::Future;
use axum::{ routing::get, routing::post, Router, Json };
use tokio::signal;
//--------------------------------------------------------------------------------------------------



pub async fn run_web_1() {
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


pub async fn run_web_2() {

    let app = Router::new()
        .route("/", get(root))
        .route("/foo1", get(get_foo).post(post_foo))
        // .route("/foo", post_service(post_foo()))
        // .route("/foo", post(||post_foo()))
        .route("/dsd", post(||async { "dsd" }))
        .route("/post_foo_02", post(post_foo_02))
        // .route("/foo3", post(post_foo_03))
        .route("/post_foo_04", post(post_foo_04))
        .route("/post_foo_05", post(post_foo_05))
        // .route("/foo3", post(|| post_foo_03()))
        .route("/post_foo", post(post_foo))
        .route("/get_json_foo_01", get(get_json_foo_01))
        .route("/post_json_foo_02", post(post_json_foo_02))
        // .route("/foo", chained_handler_fn!(post_foo, POST))
        // .route("/foo", post(post_foo))
        // .route("/foo", post(post_foo))
        // .route("/foo/bar", get(foo_bar))
        ;

    // which calls one of these handlers
    // async fn root() { async { "GET root" } }
    // async fn get_foo() { async { "GET foo" } }
    // async fn post_foo() { async { "POST foo" } }
    // async fn foo_bar() { async { "GET foo_bar" } }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    // axum::serve(listener, app).await.unwrap();

    println!("### with_graceful_shutdown");
    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();}

async fn root() -> & 'static str { "GET root" }
async fn get_foo() -> & 'static str { "GET foo" }
async fn post_foo_02() -> & 'static str { "GET foo" }
fn post_foo_04() -> impl Future<Output = & 'static str> { async { "POST foo" } }
fn post_foo_05() -> impl Future<Output = & 'static str> { core::future::ready("POST foo") }
// async fn post_foo_06() -> & 'static str { core::future::ready("POST foo") }
async fn post_foo() -> & 'static str { async { "POST foo" }.await }

async fn get_json_foo_01() -> Json<& 'static str> { Json("GET foo") }
async fn post_json_foo_02(input: Json<String>) -> Json<String> { Json(format!("GET foo <= {}", input.0)) }


async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = core::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c    => { panic!("### select ctrl_c")    },
        _ = terminate => { panic!("### select terminate") },
    }
}