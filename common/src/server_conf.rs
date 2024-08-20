use anyhow::anyhow;
use crate::env::{ env_var, env_var_2 };
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
