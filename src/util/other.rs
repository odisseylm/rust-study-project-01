use core::fmt;


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
