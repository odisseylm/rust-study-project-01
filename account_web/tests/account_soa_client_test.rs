
// include!(format!("{}/{}", env!("OUT_DIR"), "/codegen.rs"));
// include!("${OUT_DIR}/codegen.rs");
// include!(concat!(env!("OUT_DIR"), "/account_soa_client_gen.rs"));


// /home/volodymyr/projects/rust/rust-study-project-01/target/debug/build/mvv_account_web-91efa79a5a6a8866/out/generated.rs
// /home/volodymyr/projects/rust/rust-study-project-01/target/debug/build/mvv_account_web-91efa79a5a6a8866/out/codegen.rs

use http::HeaderMap;
use mvv_account_web::rest_dependencies::account_soa_client::*;


#[tokio::test]
pub async fn test_account_soa_client() -> Result<(), anyhow::Error> {

    // let basic_auth_creds = http_auth_basic::Credentials::new("vovan-read", "qwerty");
    use axum_extra::headers::{ Authorization, authorization::Credentials };

    let auth = Authorization::basic("vovan-read", "qwerty");

    let client = reqwest::Client::builder()
        // .basic_auth(user_name, password)
        .default_headers({
            let mut headers = HeaderMap::new();
            // headers.insert("Authorization", HeaderValue::from_str(&basic_auth_creds.as_http_header()) ?);
            headers.insert("Authorization", auth.0.encode());
            headers
        })
        .build() ?;

    // let response = client
    //     .get("https://httpbin.org/")
    //     .basic_auth("vovan-read", Some("qwerty"))
    //     .send();

    // client.basic_auth(user_name, password)

    // let client = Client::new("http://localhost:3000");
    let client = Client::new_with_client("http://localhost:3000", client);

    let accounts = client.get_client_accounts("00000000-0000-0000-0000-000000000001").await ?;

    println!("### accounts: {accounts:?}");

    Ok(())
}
