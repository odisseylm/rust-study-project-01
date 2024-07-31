
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
