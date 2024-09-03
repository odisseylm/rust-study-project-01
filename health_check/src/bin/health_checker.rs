use std::process::exit;
use std::time::Duration;


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let url_param: Option<String> = std::env::args().skip(1).next();
    let url = if let Some(url) = url_param {
        url
    } else {
        let (schema, port) = get_schema_and_port() ?;
        format!("{schema}://localhost:{port}/healthcheck")
    };

    let is_ok = health_check(&url) ?;

    if !is_ok {
        exit(2);
        // return Err(Box::new(std::io::Error::new(
        //     std::io::ErrorKind::Other,
        //     format!("Result status is {status}")
        // )));
    }

    Ok(())
}


fn health_check(url: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let mut client = reqwest::blocking::Client::builder();
    if url.starts_with("https:") {
        client = client.danger_accept_invalid_certs(true);
    }

    let client = client.build() ?;

    let req = client.get(url)
        .timeout(Duration::from_secs(5))
        .send() ?;

    let status = req.status();

    // if status != http::StatusCode {
    //     return Err(Box::new(std::io::Error::new(
    //         std::io::ErrorKind::Other,
    //         format!("Result status is {status}")
    //     )));
    // }

    Ok(status == 200)
}


fn get_schema_and_port() -> Result<(&'static str, u16), Box<dyn std::error::Error>> {
    let port_str: String = std::env::var("SERVER_PORT") // 8080 8443
        .unwrap_or_else(|_| "8443".to_owned());
    let port: u16 = port_str.parse()
        .map_err(|_|format!("env SERVER_PORT [{port_str}] has wrong format")) ?;

    let schema = match port {
        80  | 8080 => "http",
        443 | 8443 => "https",
        _ => {
            let has_ssl_conf: bool = std::env::var("SERVER_SSL_KEY_PATH").is_ok()
                || std::env::var("SSL_KEY_PATH").is_ok();
            if has_ssl_conf { "https" } else { "http" }
        },
    };

    Ok((schema, port))
}