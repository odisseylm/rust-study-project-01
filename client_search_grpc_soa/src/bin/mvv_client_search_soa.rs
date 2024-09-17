


#[tokio::main]
async fn main() {
    mvv_client_search_soa::app::grpc_app_main().await
        .map_err(|err|{ eprintln!("Error: {err:?}"); err})
        .expect("Oops! Failed")
}
