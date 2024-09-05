


#[tokio::main]
async fn main() {
    // mvv_account_soa::investigation_web_server::main_web::run_web_1().await
    // mvv_account_soa::investigation_web_server::main_web::run_web_2().await
    // mvv_account_soa::rweb_server::main_web::run_web_2().await
    // mvv_account_soa::rest::axum_login_investigation::temp_handler().await

    mvv_client_search_soa::app::grpc_app_main().await
        .expect("Oops! Failed")
}
