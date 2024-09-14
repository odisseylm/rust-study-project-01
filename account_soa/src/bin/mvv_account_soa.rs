
#[tokio::main]
async fn main() {
    mvv_account_soa::rest::web_app::web_app_main().await
        .expect("Oops! Failed")
}
