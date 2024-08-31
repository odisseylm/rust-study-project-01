use std::path::PathBuf;
use crate::{
    env::{env_var, env_var_2},
    exe::current_exe_dir,
    net::ConnectionType,
    secure::SecureString,
    server_conf::{get_server_connection_type, get_server_port},
    string::StaticRefOrString,
};
//--------------------------------------------------------------------------------------------------


pub fn load_url_from_env_var(var_name: &'static str) -> anyhow::Result<String> {
    let first_url = load_urls_from_env_var(var_name) ?
        .pop().ok_or_else(|| anyhow::anyhow!("Var name [{var_name}] has no any URL.")) ?;
    Ok(first_url)
}

pub fn load_urls_from_env_var(var_name: &'static str) -> anyhow::Result<Vec<String>> {
    let urls = super::env::required_env_var(var_name) ?;
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


pub fn load_path_from_env_var(var_name: &'static str) -> anyhow::Result<PathBuf> {
    let path = env_var(var_name) ?;
    let path = path.ok_or_else(|| anyhow::anyhow!("Var name [{var_name}] is empty.")) ?;

    let path = fix_path(path) ?;
    Ok(PathBuf::from(&path))
}


pub fn load_optional_path_from_env_vars<const N: usize>(var_names: [&str; N]) -> anyhow::Result<Option<PathBuf>> {

    for var_name in var_names {
        let path = env_var_2(var_name) ?;
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


pub trait ServerConf {
    fn server_name(&self) -> &StaticRefOrString;
    /// Should be uppercase.
    fn server_env_name(&self) -> &StaticRefOrString;
    fn connection_type(&self) -> ConnectionType;
    /// Main server port.
    fn server_port(&self) -> u16;
    fn server_ssl_key(&self) -> Option<&SslConfValue>;
    fn server_ssl_cert(&self) -> Option<&SslConfValue>;
}


pub struct BaseServerConf {
    pub server_name: StaticRefOrString,
    pub server_env_name: StaticRefOrString,
    pub server_port: u16,
    pub connection_type: ConnectionType,
    pub server_ssl_key: Option<SslConfValue>,
    pub server_ssl_cert: Option<SslConfValue>,
}

impl BaseServerConf {
    pub fn load_from_env(server_name: StaticRefOrString, server_env_name: StaticRefOrString)
        -> anyhow::Result<Self> where Self: Sized {

        let server_name = server_name.clone();
        let server_env_name = server_env_name.clone();
        let server_port = get_server_port(server_env_name.as_str())?;
        let connection_type = get_server_connection_type(server_env_name.as_str())?;

        let server_ssl_key: Option<SslConfValue>;
        let server_ssl_cert: Option<SslConfValue>;

        if let ConnectionType::Ssl = connection_type {
            server_ssl_key = Some(SslConfValue::Path(load_path_from_env_vars([
                // for local dev testing with single config env file
                &format!("{server_env_name}SSL_KEY_PATH"), &format!("{server_env_name}_SSL_KEY_PATH"),
                // for prod/docker
                "SERVER_SSL_KEY_PATH", "SSL_KEY_PATH"])?));

            server_ssl_cert = Some(SslConfValue::Path(load_path_from_env_vars([
                // for local dev testing with single config env file
                &format!("{server_env_name}SSL_CERT_PATH"), &format!("{server_env_name}_SSL_CERT_PATH"),
                // for prod/docker
                "SERVER_SSL_CERT_PATH", "SSL_CERT_PATH"])?));
        } else {
            server_ssl_key = None;
            server_ssl_cert = None;
        }

        Ok(Self {
            server_name,
            server_env_name,
            server_port,
            connection_type,
            server_ssl_key,
            server_ssl_cert,
        })
    }
}

impl ServerConf for BaseServerConf {
    fn server_name(&self) -> &StaticRefOrString { &self.server_name }
    fn server_env_name(&self) -> &StaticRefOrString { &self.server_env_name }
    fn connection_type(&self) -> ConnectionType { self.connection_type }
    fn server_port(&self) -> u16 { self.server_port }
    fn server_ssl_key(&self) -> Option<&SslConfValue> { self.server_ssl_key.as_ref() }
    fn server_ssl_cert(&self) -> Option<&SslConfValue> { self.server_ssl_cert.as_ref() }
}
