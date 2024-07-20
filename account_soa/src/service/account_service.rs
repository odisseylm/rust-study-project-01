use std::sync::Arc;
use log::info;
use mvv_common::entity::{ amount::Amount, bd::BigDecimalWrapper };
use crate::entity::{
    account::{ self },
    IbanWrapper, prelude::{ Account, AccountId }
};
use crate::entity::ClientId;
//--------------------------------------------------------------------------------------------------



#[derive(thiserror::Error, Debug)]
pub enum AccountError {
    #[error("AccountNotFound")]
    AccountNotFound,
    #[error("Internal")]
    Internal,
    #[error(transparent)]
    Sqlx(sqlx::Error),
}

impl From<sqlx::Error> for AccountError {
    fn from(value: sqlx::Error) -> Self {
        AccountError::Sqlx(value)
    }
}

// #[trait_variant::make(SendAccountService: Send)]
#[trait_variant::make(Send)]
// or #[async_trait] // https://github.com/dtolnay/async-trait#dyn-traits
pub trait AccountService: Send + Sync {
    async fn get_client_accounts(&self, client_id: ClientId) -> Result<Vec<Account>, AccountError>;
    async fn get_client_account_by_id(&self, client_id: ClientId, account_id: AccountId) -> Result<Account, AccountError>;
    async fn get_client_account_by_iban(&self, client_id: ClientId, iban: iban::Iban) -> Result<Account, AccountError>;
    async fn transfer(&self, client_id: ClientId, from_account: iban::Iban, to_account: iban::Iban, amount: Amount) -> Result<(), AccountError>;
}

pub struct AccountServiceImpl {
    pub database_connection: Arc<sqlx_postgres::PgPool>,
}

// ??? Hm... cannot use there AccountServiceSafe !?
impl AccountService for AccountServiceImpl {

    async fn get_client_accounts(&self, client_id: ClientId) -> Result<Vec<Account>, AccountError> {

        info!("### Loading user ACCOUNTS of user [{}] from database", client_id);

        let res= sqlx::query_as(
            "select \
                 ID, IBAN, CLIENT_ID, NAME, \
                 AMOUNT, CUR, \
                 CREATED_AT, UPDATED_AT \
                 from ACCOUNTS \
                 where CLIENT_ID = $1 ")
            .bind(&client_id.into_inner())
            .fetch_all(&*self.database_connection)
            .await
            .map_err(|err_to_log|{
                log::error!("### SQLX error: {:?}", err_to_log);
                err_to_log
            })
            // .map_err(Self::Error::Sqlx)?)
            .map_err(From::<sqlx::Error>::from);

        res
        /*
        let accounts = vec!(Account::new( account::new::Args {
            id: AccountId::from_str("345").unwrap(),
            user_id,
            amount: Amount::from_str("135.79 EUR").unwrap(),
            updated_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 22:29:57 +02:00").unwrap().to_utc(),
            created_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 15:31:09 +02:00").unwrap().to_utc(),
        }));
        Ok(accounts)
        */
    }

    async fn get_client_account_by_id(&self, client_id: ClientId, account_id: AccountId) -> Result<Account, AccountError> {

        info!("### Loading user ACCOUNT [{account_id}] of client [{client_id}] from database");

        let res = sqlx::query_as(
            "select \
                 ID, IBAN, CLIENT_ID, NAME, \
                 AMOUNT, CUR, \
                 CREATED_AT, UPDATED_AT \
                 from ACCOUNTS \
                 where CLIENT_ID = $1 and ACCOUNT_ID = $2 ")
            .bind(&client_id.into_inner())
            .bind(&account_id.into_inner())
            .fetch_one(&*self.database_connection)
            .await
            .map_err(|err_to_log|{
                log::error!("### SQLX error: {:?}", err_to_log);
                err_to_log
            })
            // .map_err(Self::Error::Sqlx)?)
            .map_err(From::<sqlx::Error>::from);
        res
    }

    async fn get_client_account_by_iban(&self, client_id: ClientId, iban: iban::Iban) -> Result<Account, AccountError> {

        info!("### Loading user ACCOUNT [{iban}] of client [{client_id}] from database");

        let res = sqlx::query_as(
            "select \
                 ID, IBAN, CLIENT_ID, NAME, \
                 AMOUNT, CUR, \
                 CREATED_AT, UPDATED_AT \
                 from ACCOUNTS \
                 where CLIENT_ID = $1 and IBAN = $2 ")
            .bind(&client_id)
            // .bind(&client_id.into_inner().to_string())
            .bind(&IbanWrapper(iban))
            .fetch_one(&*self.database_connection)
            .await
            .map_err(|err_to_log|{
                // TODO: if not found, return 404
                log::error!("### SQLX error: {:?}", err_to_log);
                err_to_log
            })
            // .map_err(Self::Error::Sqlx)?)
            .map_err(From::<sqlx::Error>::from);
        res
    }

    async fn transfer(&self, _client_id: ClientId, _from_account: iban::Iban, _to_account: iban::Iban, _amount: Amount) -> Result<(), AccountError> {
        todo!()
    }
}


impl sqlx::FromRow<'_, sqlx_postgres::PgRow> for Account {
    fn from_row(row: &sqlx_postgres::PgRow) -> sqlx::Result<Self> {

        use sqlx::Row;
        macro_rules! col_name {
            // postgres needs lowercase (Oracle - uppercase, so on)
            ($column_name:literal) => { const_str::convert_ascii_case!(lower, $column_name) };
        }

        let account = Account::new(account::new::Args {
            id: row.try_get(col_name!("ID")) ?,
            // TODO: How do cast properly/shortly ??
            iban: { let iban: IbanWrapper = row.try_get(col_name!("IBAN")) ?; iban.0 },
            client_id: row.try_get(col_name!("CLIENT_ID")) ?,
            name: row.try_get(col_name!("NAME")) ?,
            amount: Amount::new(
                // How do cast properly/shortly ??
                { let amount: BigDecimalWrapper = row.try_get(col_name!("AMOUNT")) ?; amount.0 },
                row.try_get(col_name!("CUR")) ?,
            ),
            created_at: row.try_get(col_name!("CREATED_AT")) ?,
            updated_at: row.try_get(col_name!("UPDATED_AT")) ?,
        });

        Ok(account)
    }
}
