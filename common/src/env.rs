use core::fmt;
use std::path::PathBuf;
use crate::backtrace2::BacktraceCell;
use crate::string::StaticRefOrString;
//--------------------------------------------------------------------------------------------------



pub fn env_var_or_else <F: Fn()->String> (env_var_name: &'static str, or_string: F) -> Result<String, EnvVarError> {
    use std::env::VarError;

    let port_env = std::env::var(env_var_name);
    match port_env {
        Ok(port_env) => Ok(port_env),
        Err(err) => {
            match err {
                VarError::NotPresent => Ok(or_string()),
                VarError::NotUnicode(ref _err) =>
                    Err(EnvVarError::new(env_var_name.into(), err)),
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
                VarError::NotUnicode(ref _err) =>
                    Err(EnvVarError::new(env_var_name.into(), err)),
            }
        }
    }
}


pub fn env_var_2 (env_var_name: &str) -> Result<Option<String>, EnvVarError> {
    use std::env::VarError;

    let port_env = std::env::var(env_var_name);
    match port_env {
        Ok(port_env) => Ok(Some(port_env)),
        Err(err) => {
            match err {
                VarError::NotPresent => Ok(None),
                VarError::NotUnicode(ref _err) =>
                    Err(EnvVarError::new(env_var_name.to_owned().into(), err)),
            }
        }
    }
}


pub fn required_env_var (env_var_name: &'static str) -> Result<String, EnvVarError> {
    env_var(env_var_name) ?
        .ok_or_else(|| EnvVarError::new(env_var_name.into(), std::env::VarError::NotPresent))
}

#[derive(Debug, thiserror::Error)]
pub struct EnvVarError {
    pub var_name: StaticRefOrString, // &'static str,
    pub source: std::env::VarError,
    pub backtrace: BacktraceCell,
}
impl EnvVarError {
    pub fn new(var_name: StaticRefOrString, source: std::env::VarError) -> Self {
        Self {
            var_name, source,
            backtrace: BacktraceCell::capture_backtrace(),
        }
    }
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
            None => Err(EnvVarError::new(var_name.into(), std::env::VarError::NotPresent)),
            Some(var_value) => Ok(var_value)
        }
    }
}



pub fn process_env_load_res(env_filename: &str, dotenv_res: Result<PathBuf, dotenv::Error>) -> Result<(), anyhow::Error> {
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


#[cfg(test)]
mod tests {
    use assertables::{
        assert_contains, assert_contains_as_result,
        assert_not_contains, assert_not_contains_as_result,
    };
    use crate::env::EnvVarError;
    use crate::test::{ TestDisplayStringOps, TestDebugStringOps };

    #[test]
    fn test_print() {
        let err = EnvVarError::new(
            "var_name_1".into(), std::env::VarError::NotPresent);

        // println!("\n------------------------------------------\n");
        // println!("err as display: {err}");
        let display_str = err.to_test_display_string();
        assert_contains!(display_str, r#"Env var ["var_name_1"] is not found."#);
        assert_not_contains!(display_str, "mvv_common::env::EnvVarError::new");
        assert_not_contains!(display_str, "env.rs:");

        // println!("\n------------------------------------------\n");
        // println!("err as debug: {err:?}");
        let debug_str = err.to_test_debug_string();
        println!("### EnvVarError (debug): {debug_str}");
        assert_contains!(debug_str, r#"EnvVarError { var_name: Ref("var_name_1"), source: NotPresent, backtrace:"#);
        assert_contains!(debug_str, "mvv_common::env::EnvVarError::new");
        assert_contains!(debug_str, "env.rs:");

        // assert!(false, "To see output");
    }
}
