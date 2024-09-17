use log::info;
use reqwest::Certificate;
use mvv_auth::client::basic_auth_headers_by_client_cfg;
use mvv_common::{
    soa::RestCallError,
    cfg::{DependencyConnectConf, client::to_reqwest_tls_cert},
    soa::improve_prog_err,
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

    async fn get_client_accounts(&self, client_id: &str) -> Result<Vec<Account>, RestCallError> {
        let r = self.client.get_client_accounts(client_id).await
            .map_err(improve_prog_err) ?;
        Ok(r.into_inner())
    }

    // async fn get_client_account(&self, client_id: &str, account_id: &str) -> anyhow::Result<Account> {
    //     let r = self.client.get_client_account(client_id, account_id).await
    //         .map_err(improve_prog_err) ?;
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
    //     .map_err(improve_prog_err)
    // }
}


pub fn create_account_service<Cfg: DependencyConnectConf>(cfg: &Cfg) -> anyhow::Result<AccountServiceImpl> {

    info!("Creating account service base on config [{cfg:?}]");
    let client = create_reqwest_client(cfg) ?;

    let client = AccountSoaRestClient::new_with_client(cfg.base_url().as_str(), client);
    let account_service = AccountServiceImpl { client };
    Ok(account_service)
}


fn create_reqwest_client<Cfg: DependencyConnectConf>(cfg: &Cfg) -> anyhow::Result<reqwest::Client> {

    let mut client = reqwest::Client::builder()
        .default_headers(basic_auth_headers_by_client_cfg(cfg));

    let cert = cfg.ca_cert().as_ref()
        .or(cfg.server_cert().as_ref());
    let cert: Option<Certificate> = to_reqwest_tls_cert(cert) ?;

    if let Some(cert) = cert {
        client = client.add_root_certificate(cert);
    }
    let client = client.build() ?;

    Ok(client)
}
