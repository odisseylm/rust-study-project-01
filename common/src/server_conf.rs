use anyhow::anyhow;
use crate::env::{ env_var, env_var_2 };
use crate::obj_ext::ValExt;
//--------------------------------------------------------------------------------------------------



pub fn get_server_port(service_name: &'static str) -> Result<u16, anyhow::Error> {
    let precise_app_port_env_name = format!("{}_SERVER_PORT", service_name.to_uppercase());
    let general_app_port_env_name_1 = "SERVER_PORT";
    let general_app_port_env_name_2 = "PORT";
    let default_port: u16 = 3000;

    let port_env = env_var_2(&precise_app_port_env_name) ?;
    let port_env_alt_1 = env_var(general_app_port_env_name_1) ?;
    let port_env_alt_2 = env_var(general_app_port_env_name_2) ?;

    let port_env = port_env.or(port_env_alt_1).or(port_env_alt_2);

    match port_env {
        None => Ok(default_port),
        Some(ref port_env) => {
            use core::str::FromStr;
            let port: u16 = FromStr::from_str(port_env)
                .map_err(|_|anyhow!("SERVER_PORT value [{}] has wrong format.", port_env)) ?;
            Ok(port)
        }
    }
}


#[derive(Debug, Clone, Copy, strum::Display)]
pub enum ConnectionType {
    Plain,
    Ssl
}


pub fn get_server_connection_type(service_name: &'static str) -> Result<ConnectionType, anyhow::Error> {
    let precise_app_ssl_key_env_name = format!("{}_SSL_KEY_PATH", service_name.to_uppercase());
    let general_app_ssl_key_env_name_1 = "SERVER_SSL_KEY_PATH";
    let general_app_ssl_key_env_name_2 = "SSL_KEY_PATH";

    let is_ssl_1 = env_var_2(&precise_app_ssl_key_env_name) ?.map(|s|!s.is_empty());
    let is_ssl_2 = env_var(general_app_ssl_key_env_name_1) ?.map(|s|!s.is_empty());
    let is_ssl_3 = env_var(general_app_ssl_key_env_name_2) ?.map(|s|!s.is_empty());

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
