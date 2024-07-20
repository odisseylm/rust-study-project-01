use anyhow::anyhow;
use crate::env::env_var;
//--------------------------------------------------------------------------------------------------



pub fn get_server_port() -> Result<u32, anyhow::Error> {
    use core::str::FromStr;

    let port_env = env_var("SERVER_PORT") ?;
    let port_env = port_env.as_deref().unwrap_or("3000");
    let port: u32 = FromStr::from_str(port_env)
        .map_err(|_|anyhow!("SERVER_PORT value [{}] has wrong format.", port_env)) ?;
    Ok(port)
}
