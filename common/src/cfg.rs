use std::path::PathBuf;
use crate::{
    env::env_var,
    exe::current_exe_dir,
    secure::SecureString,
    server_conf::{get_server_connection_type, get_server_port},
};
use crate::env::required_env_var;

pub mod client;
pub mod server;
//--------------------------------------------------------------------------------------------------


pub fn load_url_from_env_var(var_name: &'static str) -> anyhow::Result<String> {
    let first_url = load_urls_from_env_var(var_name) ?
        .pop().ok_or_else(|| anyhow::anyhow!("Var name [{var_name}] has no any URL.")) ?;
    Ok(first_url)
}

pub fn load_url_from_env_vars<const N: usize>(var_names: [&str;N]) -> anyhow::Result<String> {

    let mut urls = var_names.iter()
        .filter_map(|var_name| load_urls_from_env_var(var_name).ok())
        .flatten()
        .collect::<Vec<String>>();

    let first_url = urls.pop()
        .ok_or_else(|| anyhow::anyhow!("Var name [{var_names:?}] has no any URL.")) ?;
    Ok(first_url)
}

pub fn load_urls_from_env_var(var_name: &str) -> anyhow::Result<Vec<String>> {
    let urls = required_env_var(var_name) ?;
    let urls: Vec<String> = urls.split(',').map(|s|s.to_owned()).collect::<Vec<_>>();

    if urls.is_empty() {
        anyhow::bail!("Var name [{var_name}] has no any URL.")
    }

    for url in urls.iter() {
        url::Url::parse(url)
            .map_err(|_|anyhow::anyhow!("Env var [{var_name}] has bad formed URL [{url}]."))?;
    }
    Ok(urls)
}


pub fn load_path_from_env_var(var_name: &str) -> anyhow::Result<PathBuf> {
    let path = env_var(var_name) ?;
    let path = path.ok_or_else(|| anyhow::anyhow!("Var name [{var_name}] is empty.")) ?;

    let path = fix_path(path) ?;
    Ok(PathBuf::from(&path))
}


pub fn load_optional_path_from_env_vars<const N: usize>(var_names: [&str; N]) -> anyhow::Result<Option<PathBuf>> {

    for var_name in var_names {
        let path = env_var(var_name) ?;
        let path = match path {
            None => continue,
            Some(path) => path,
        };

        let path = fix_path(path)?;
        return Ok(Some(PathBuf::from(&path)))
    }

    Ok(None)
}


pub fn load_path_from_env_vars<const N: usize>(var_names: [&str; N]) -> anyhow::Result<PathBuf> {
    let optional_path = load_optional_path_from_env_vars(var_names) ?;
    match optional_path {
        None => anyhow::bail!("No path found in env vars [{var_names:?}]"),
        Some(path) => Ok(path),
    }
}

fn fix_path(path: String) -> anyhow::Result<String> {
    let path =
        if path.contains("${EXE_PATH_DIR}") {
            let exe_path_dir = current_exe_dir() ?;
            let exe_path_dir = exe_path_dir.to_string_lossy();
            path.replace("${EXE_PATH_DIR}", exe_path_dir.as_ref())
        } else {
            path
        };
    Ok(path)
}

#[derive(Clone, Debug)]
pub enum SslConfValue {
    Path(PathBuf), // Path(SecureString),
    Value(SecureString),
}

pub use server::ServerConf;
pub use server::BaseServerConf;

pub use client::DependencyType;
pub use client::DependencyConnectConf;
pub use client::BaseDependencyConnectConf;