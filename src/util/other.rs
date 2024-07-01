// use std::env::VarError;

pub fn env_var_or_else <F: Fn()->String> (env_var_name: &str, or_string: F) -> Result<String, std::env::VarError> {
    use std::env::VarError;

    let port_env = std::env::var(env_var_name);
    match port_env {
        Ok(port_env) => Ok(port_env),
        Err(err) => {
            match err {
                VarError::NotPresent => Ok(or_string()),
                VarError::NotUnicode(ref _err) => Err(err),
            }
        }
    }
}

