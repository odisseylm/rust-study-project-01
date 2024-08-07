use http::HeaderMap;
use mvv_common::{
    cfg::load_url_from_env_var,
    soa::RestCallError,
};
use crate::rest_dependencies::account_soa_client::{
    Client as AccountSoaRestClient,
    types::{
        Account, // Amount, TransferAmountRequest,
    }
};
//--------------------------------------------------------------------------------------------------



#[async_trait::async_trait]
pub trait AccountService {
    // async fn get_client_accounts<'a>(&'a self, client_id: String) -> anyhow::Result<Vec<Account>>;
    async fn get_client_accounts(&self, client_id: &str) -> Result<Vec<Account>, RestCallError>;
    // async fn get_client_account(&self, client_id: &str, account_id: &str) -> anyhow::Result<Account>;
    // async fn transfer_amount(&self, client_id: &str, from_account_id: String, to_account_id: String, amount: Amount)
    //     -> anyhow::Result<()>;
}


pub struct AccountServiceImpl {
    client: AccountSoaRestClient,
}


#[axum::async_trait]
impl AccountService for AccountServiceImpl {
    // async fn get_client_accounts<'a>(&'a self, client_id: String) -> anyhow::Result<Vec<Account>> {
    //     let r = self.client.get_client_accounts::<'a>(client_id.as_str()).await ?;
    //     Ok(r.into_inner())
    // }

    async fn get_client_accounts(&self, client_id: &str) -> Result<Vec<Account>, RestCallError> {
        let r = self.client.get_client_accounts(client_id).await ?;
            // .map_err(improve_prog_err) ?;
        Ok(r.into_inner())
    }

    // async fn get_client_accounts(&self, client_id: String) -> anyhow::Result<Vec<Account>> {
    //     let client: Client = client();
    //     let r = client.get_client_accounts(client_id.as_str()).await ?;
    //     Ok(r.into_inner())
    // }

    // async fn get_client_account(&self, client_id: &str, account_id: &str) -> anyhow::Result<Account> {
    //     let r = self.client.get_client_account(client_id, account_id).await ?;
    //     Ok(r.into_inner())
    // }
    //
    // async fn transfer_amount(&self, client_id: &str, from_account_id: String, to_account_id: String, amount: Amount) -> anyhow::Result<()> {
    //     self.client.transfer_amount(client_id, &TransferAmountRequest {
    //         from_account: from_account_id,
    //         to_account: to_account_id,
    //         amount: amount.value,
    //         currency: amount.currency,
    //     }).await
    // }
}


// - DEPENDENCIES_ACCOUNTSOA_REST_BASEURLS=https://account-soa/account-soa/api
// - DEPENDENCIES_ACCOUNTSOA_REST_BASEURLTEMPLATE=https://bank-plugin-account-soa-REPLICA_NUMBER/account-soa/api
// - DEPENDENCIES_ACCOUNTSOA_REST_CONTEXTPATH=/account-soa/api
// - DEPENDENCIES_ACCOUNTSOA_REST_REPLICACOUNT=${DOCKER_COMPOSE_SCALE_REPLICA_COUNT}
// - DEPENDENCIES_ACCOUNTSOA_REST_REPLICAIDTYPE=OneBasedIncremented
// - DEPENDENCIES_ACCOUNTSOA_REST_PINGTIMEOUT=5s
// - DEPENDENCIES_ACCOUNTSOA_REST_BALANCERTYPE=ribbon
//
#[derive(Debug)]
pub struct AccountSoaConnectCfg {
    pub base_url: String,
    pub user: String,
    pub psw: String,
}
impl AccountSoaConnectCfg {
    pub fn load_from_env() -> anyhow::Result<Self> {
        Ok(AccountSoaConnectCfg {
            // In general there may be several URLs with client balancing,
            // but now we use only 1st url
            base_url:
                load_url_from_env_var("DEPENDENCIES_ACCOUNTSOA_REST_BASEURLS") ?,
            user:
                mvv_common::env::required_env_var("DEPENDENCIES_ACCOUNTSOA_USER") ?,
            psw:
                mvv_common::env::required_env_var("DEPENDENCIES_ACCOUNTSOA_PSW") ?,
        })
    }
}

pub fn create_account_service(cfg: &AccountSoaConnectCfg) -> anyhow::Result<AccountServiceImpl> {

    use axum_extra::headers::{ Authorization, authorization::Credentials };

    let auth = Authorization::basic(&cfg.user, &cfg.psw);

    let client = reqwest::Client::builder()
        .default_headers({
            let mut headers = HeaderMap::new();
            // headers.insert("Authorization", HeaderValue::from_str(&basic_auth_creds.as_http_header()) ?);
            headers.insert("Authorization", auth.0.encode());
            headers
        })
        .build() ?;

    // let client = AccountSoaRestClient::new(&cfg.base_url);
    let client = AccountSoaRestClient::new_with_client(&cfg.base_url, client);
    let account_service = AccountServiceImpl { client };
    Ok(account_service)
}
