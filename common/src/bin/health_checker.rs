use std::time::Duration;
use http::StatusCode;
use mvv_common::unchecked::UncheckedResultUnwrap;


fn main() {

    let url_param: Option<String> = std::env::args().skip(1).next();
    let url = if let Some(url) = url_param {
        url
    } else {
        let port_str: String = std::env::var("SERVER_PORT") // 8080 8443
            .unwrap_or_else(|_| "8443".to_owned());
        let port: u16 = port_str.parse()
            .map_err(|_|format!("env SERVER_PORT [{port_str}] has wrong format"))
            .unchecked_unwrap();

        format!("https://localhost:{port}/healthcheck")
    };

    let mut client = reqwest::blocking::Client::builder();
    if url.starts_with("https:") {
        client = client.danger_accept_invalid_certs(true);
    }

    let client = client.build().unchecked_unwrap();

    let req = client.get(&url)
        .timeout(Duration::from_secs(5))
        .send()
        .unchecked_unwrap();

    let status = req.status();

    if status != StatusCode::OK {
        panic!("Result status is {status}");
    }
}
