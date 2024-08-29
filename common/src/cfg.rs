use std::path::PathBuf;
use crate::env::env_var;
use crate::exe::current_exe_dir;
use crate::secure::SecureString;
//--------------------------------------------------------------------------------------------------


pub fn load_url_from_env_var(var_name: &'static str) -> anyhow::Result<String> {
    let first_url = load_urls_from_env_var(var_name) ?
        .pop().ok_or_else(|| anyhow::anyhow!("Var name [{var_name}] has no any URL.")) ?;
    Ok(first_url)
}

pub fn load_urls_from_env_var(var_name: &'static str) -> anyhow::Result<Vec<String>> {
    let urls = super::env::required_env_var(var_name) ?;
    let urls: Vec<String> = urls.split(',').map(|s|s.to_owned()).collect::<Vec<_>>();
    // let urls: Vec<String> = Vec::new();

    if urls.is_empty() {
        return Err(anyhow::anyhow!("Var name [{var_name}] has no any URL."))
    }

    for url in urls.iter() {
        url::Url::parse(url)
            .map_err(|_|anyhow::anyhow!("Env var [{var_name}] has bad formed URL [{url}]."))?;
    }
    Ok(urls)
}


pub fn load_path_from_env_var(var_name: &'static str) -> anyhow::Result<PathBuf> {
    let path = env_var(var_name) ?;
    let path = path.ok_or_else(|| anyhow::anyhow!("Var name [{var_name}] is empty.")) ?;

    let path = fix_path(path) ?;
    Ok(PathBuf::from(&path))
}


pub fn load_path_from_env_vars<const N: usize>(var_names: [&'static str; N]) -> anyhow::Result<PathBuf> {

    for var_name in var_names {
        let path = env_var(var_name) ?;
        let path = match path {
            None => continue,
            Some(path) => path,
        };

        let path = fix_path(path)?;
        return Ok(PathBuf::from(&path))
    }

    anyhow::bail!("No path found in env vars [{var_names:?}]")
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