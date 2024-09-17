
#[tokio::main]
async fn main() {
    mvv_account_soa::rest::web_app::web_app_main().await
        .map_err(|err|{ eprintln!("Error: {err:?}"); err})
        .expect("Oops! Failed")
}
