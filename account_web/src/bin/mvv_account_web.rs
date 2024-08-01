

#[tokio::main]
async fn main() {
    mvv_account_web::web_app::web_app_main().await
        .expect("Oops! Failed")
}
