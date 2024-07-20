use core::fmt;
use std::path::PathBuf;

pub fn env_var_or_else <F: Fn()->String> (env_var_name: &'static str, or_string: F) -> Result<String, EnvVarError> {
    use std::env::VarError;

    let port_env = std::env::var(env_var_name);
    match port_env {
        Ok(port_env) => Ok(port_env),
        Err(err) => {
            match err {
                VarError::NotPresent => Ok(or_string()),
                VarError::NotUnicode(ref _err) => Err(EnvVarError { var_name: env_var_name, source: err }),
            }
        }
    }
}


pub fn env_var (env_var_name: &'static str) -> Result<Option<String>, EnvVarError> {
    use std::env::VarError;

    let port_env = std::env::var(env_var_name);
    match port_env {
        Ok(port_env) => Ok(Some(port_env)),
        Err(err) => {
            match err {
                VarError::NotPresent => Ok(None),
                VarError::NotUnicode(ref _err) => Err(EnvVarError { var_name: env_var_name, source: err }),
            }
        }
    }
}


#[derive(Debug, Clone, thiserror::Error)]
pub struct EnvVarError {
    pub var_name: &'static str,
    pub source: std::env::VarError,
}

impl fmt::Display for EnvVarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::env::VarError;

        match self.source {
            VarError::NotPresent => write!(f, "Env var [{}] is not found.", self.var_name),
            VarError::NotUnicode(_) => write!(f, "Env var [{}] is broken.", self.var_name),
        }
    }
}


#[extension_trait::extension_trait]
pub impl EnvVarOps for Option<String> {
    #[track_caller]
    fn val_or_not_found_err(self, var_name: &'static str) -> Result<String, EnvVarError> {
        match self {
            None => Err(EnvVarError { var_name, source: std::env::VarError::NotPresent }),
            Some(var_value) => Ok(var_value)
        }
    }
}



pub fn process_env_load_res(dotenv_res: Result<PathBuf, dotenv::Error>) -> Result<(), anyhow::Error> {
    // We cannot put `dotenv::from_filename()` just there because logger is not initialized yet.

    match dotenv_res {
        Ok(ref path) =>
            log::info!("Env vars are loaded from [{:?}]", path),
        Err(dotenv::Error::Io(ref io_err))
        if io_err.kind() == std::io::ErrorKind::NotFound => {
            log::info!("Env vars are not loaded from [{env_filename}] file.");
        }
        Err(ref _err) => {
            log::error!("Error of loading .env file.");
            anyhow::bail!("Error of loading .env file.");
        }
    }
    Ok(())
}
