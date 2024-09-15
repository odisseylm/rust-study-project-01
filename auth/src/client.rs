use http::HeaderMap;
use mvv_common::cfg::DependencyConnectConf;
//--------------------------------------------------------------------------------------------------



pub fn basic_auth_headers_by_client_cfg<Cfg: DependencyConnectConf>(cfg: &Cfg) -> HeaderMap {
    let mut headers = HeaderMap::new();
    add_basic_auth_header_by_client_cfg(cfg, &mut headers);
    headers
}

pub fn add_basic_auth_header_by_client_cfg<Cfg: DependencyConnectConf>(cfg: &Cfg, headers: &mut HeaderMap) {
    if let Some(ref user) = cfg.user() {
        let psw = cfg.password().as_ref()
            .map(|psw|psw.as_str()).unwrap_or("");

        use axum_extra::headers::{ Authorization, authorization::Credentials };
        let auth = Authorization::basic(user.as_str(), psw);
        headers.insert("Authorization", auth.0.encode());
    }
}
