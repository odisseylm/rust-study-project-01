use mvv_account_web::rest_dependencies::account_soa::test_account_soa_client;
use mvv_common::unchecked::UncheckedResultUnwrap;

#[tokio::main]
async fn main() {
    // mvv_account_soa::investigation_web_server::main_web::run_web_1().await
    // mvv_account_soa::investigation_web_server::main_web::run_web_2().await
    // mvv_account_soa::rweb_server::main_web::run_web_2().await
    // mvv_account_soa::rest::axum_login_investigation::temp_handler().await

    //mvv_account_soa::rest::web_app::web_app_main().await
    //    .expect("Oops! Failed")

    // crate::rest_dependencies::test_account_soa_client() ?;
    test_account_soa_client().await.unchecked_unwrap()
}
