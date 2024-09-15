use anyhow::anyhow;
use crate::env::{env_var_static, env_var};
use crate::net::ConnectionType;
use crate::obj_ext::ValExt;
//--------------------------------------------------------------------------------------------------



pub fn get_server_port(service_name: &str) -> Result<u16, anyhow::Error> {
    let precise_app_port_env_name_1 = format!("{}_SERVER_PORT", service_name.to_uppercase());
    let precise_app_port_env_name_2 = format!("{}SERVER_PORT", service_name.to_uppercase());
    let general_app_port_env_name_1 = "SERVER_PORT";
    let general_app_port_env_name_2 = "PORT";
    let default_port: u16 = 8080;

    // TODO: Use 'for' for it
    let port_env_1 = env_var(&precise_app_port_env_name_1) ?;
    let port_env_2 = env_var(&precise_app_port_env_name_2) ?;
    let port_env_alt_1 = env_var_static(general_app_port_env_name_1) ?;
    let port_env_alt_2 = env_var_static(general_app_port_env_name_2) ?;

    let str_port = port_env_1.or(port_env_2).or(port_env_alt_1).or(port_env_alt_2);

    match str_port {
        None => Ok(default_port),
        Some(ref str_port) => {
            use core::str::FromStr;
            let port: u16 = FromStr::from_str(str_port)
                .map_err(|_|anyhow!("SERVER_PORT value [{str_port}] has wrong format.")) ?;
            Ok(port)
        }
    }
}



pub fn get_server_connection_type(service_name: &str) -> Result<ConnectionType, anyhow::Error> {
    let precise_app_ssl_key_env_name = format!("{}_SSL_KEY_PATH", service_name.to_uppercase());
    let general_app_ssl_key_env_name_1 = "SERVER_SSL_KEY_PATH";
    let general_app_ssl_key_env_name_2 = "SSL_KEY_PATH";

    let is_ssl_1 = env_var(&precise_app_ssl_key_env_name) ?.map(|s|!s.is_empty());
    let is_ssl_2 = env_var_static(general_app_ssl_key_env_name_1) ?.map(|s|!s.is_empty());
    let is_ssl_3 = env_var_static(general_app_ssl_key_env_name_2) ?.map(|s|!s.is_empty());

    let is_ssl = [is_ssl_1, is_ssl_2, is_ssl_3].contains(&Some(true));
    if is_ssl {
        return Ok(ConnectionType::Ssl);
    }

    let port = get_server_port(service_name) ?;
    if port.is_one_of2(443, 8443) {
        return Ok(ConnectionType::Ssl);
    }

    Ok(ConnectionType::Plain)
}
