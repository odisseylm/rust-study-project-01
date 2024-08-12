use std::sync::Arc;
use bigdecimal::BigDecimal;
use chrono::Utc;
use log::{ debug, info };
use sqlx::Transaction;
use sqlx_postgres::Postgres;
use mvv_common::{
    entity::{
        amount::Amount, bd::{BigDecimalWrapper }, amount::ops::AmountOpsError, AmountParts,
    },
};
use mvv_common::backtrace::{backtrace, BacktraceCell};
use crate::entity::{
    account::{ self },
    IbanWrapper, IbanRefWrapper, prelude::{ Account, AccountId },
    ClientId,
};
//--------------------------------------------------------------------------------------------------


#[derive(Debug, Clone)]
pub enum AccountIdWrapper {
    Id(AccountId),
    Iban(iban::Iban),
}


#[derive(
    Debug,
    thiserror::Error,
    mvv_error_macro::ThisErrorFromWithBacktrace,
    mvv_error_macro::ThisErrorBacktraceSource,
)]
pub enum AccountProcessError {
    #[error("AccountNotFound")]
    AccountNotFound(AccountIdWrapper, BacktraceCell),
    #[error("NotEnoughBalance")]
    NotEnoughBalance(AccountIdWrapper, BacktraceCell),
    #[error("AmountOpsError {{ {0} }}")]
    AmountOpsError(#[from] AmountOpsError),
    #[error("Internal")]
    Internal(#[source] anyhow::Error),
    #[error("Sqlx error")]
    Sqlx(#[from_with_bt] sqlx::Error, BacktraceCell),
}


// #[trait_variant::make(SendAccountService: Send)]
#[trait_variant::make(Send)]
// or #[async_trait] // https://github.com/dtolnay/async-trait#dyn-traits
pub trait AccountService: Send + Sync {
    async fn get_client_accounts(&self, client_id: ClientId) -> Result<Vec<Account>, AccountProcessError>;
    async fn get_client_account_by_id(&self, client_id: ClientId, account_id: AccountId) -> Result<Account, AccountProcessError>;
    async fn get_client_account_by_iban(&self, client_id: ClientId, iban: iban::Iban) -> Result<Account, AccountProcessError>;
    async fn transfer_by_iban(&self, client_id: ClientId, from_account: iban::Iban, to_account: iban::Iban, amount: Amount) -> Result<(), AccountProcessError>;
    async fn transfer_by_id(&self, client_id: ClientId, from_account: AccountId, to_account: AccountId, amount: Amount) -> Result<(), AccountProcessError>;
}

pub struct AccountServiceImpl {
    pub database_connection: Arc<sqlx_postgres::PgPool>,
}

// ??? Hm... cannot use there AccountServiceSafe !?
impl AccountService for AccountServiceImpl {

    async fn get_client_accounts(&self, client_id: ClientId) -> Result<Vec<Account>, AccountProcessError> {

        info!("### Loading user ACCOUNTS of user [{}] from database", client_id);

        let res= sqlx::query_as(
            "select \
                 ID, IBAN, CLIENT_ID, NAME, \
                 AMOUNT, CUR, \
                 CREATED_AT, UPDATED_AT \
                 from ACCOUNTS \
                 where CLIENT_ID = $1 order by CREATED_AT ")
            .bind(&client_id.into_inner())
            .fetch_all(&*self.database_connection)
            .await
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

    async fn get_client_account_by_id(&self, client_id: ClientId, account_id: AccountId)
        -> Result<Account, AccountProcessError> {
        info!("### Loading user ACCOUNT [{account_id}] of client [{client_id}] from database");
        let mut tx: Transaction<Postgres> = self.database_connection.begin().await ?;
        let res = self.get_client_account_by_id_impl(&mut tx, &client_id, &account_id).await;
        // tx.rollback().await ?;
        res
    }

    async fn get_client_account_by_iban(&self, client_id: ClientId, iban: iban::Iban)
        -> Result<Account, AccountProcessError> {
        info!("### Loading user ACCOUNT [{iban}] of client [{client_id}] from database");
        let mut tx: Transaction<Postgres> = self.database_connection.begin().await ?;
        let res = self.get_client_account_by_iban_impl(&mut tx, &client_id, &iban).await;
        // tx.rollback().await ?;
        res
    }

    async fn transfer_by_iban(&self, client_id: ClientId, from_account_id: iban::Iban, to_account_id: iban::Iban, amount: Amount)
        -> Result<(), AccountProcessError> {

        info!("### transfer from ACCOUNT [{from_account_id}] to [{to_account_id}] of client [{client_id}] from database");

        let mut tx: Transaction<Postgres> = self.database_connection.begin().await ?;

        let from_account = self.get_client_account_by_iban_impl(&mut tx, &client_id, &from_account_id).await ?;
        let to_account = self.get_client_account_by_iban_impl(&mut tx, &client_id, &to_account_id).await ?;

        let new_from_account_amount = (&from_account.amount - &amount) ?;
        let new_to_account_amount = (&to_account.amount + &amount) ?;

        let zero = BigDecimal::from(0i32);
        if new_from_account_amount.value.le(&zero) {
            return Err(AccountProcessError::NotEnoughBalance(
                AccountIdWrapper::Iban(from_account_id), backtrace()));
        }

        self.update_account_by_iban_impl(&mut tx, &client_id, &from_account_id, new_from_account_amount).await ?;
        self.update_account_by_iban_impl(&mut tx, &client_id, &to_account_id, new_to_account_amount).await ?;

        tx.commit().await ?;
        Ok(())
    }

    async fn transfer_by_id(&self, client_id: ClientId, from_account_id: AccountId, to_account_id: AccountId, amount: Amount)
        -> Result<(), AccountProcessError> {

        info!("### transfer from ACCOUNT [{from_account_id}] to [{to_account_id}] of client [{client_id}] from database");

        let mut tx: Transaction<Postgres> = self.database_connection.begin().await ?;

        let from_account = self.get_client_account_by_id_impl(&mut tx, &client_id, &from_account_id).await ?;
        let to_account = self.get_client_account_by_id_impl(&mut tx, &client_id, &to_account_id).await ?;

        let new_from_account_amount = (&from_account.amount - &amount) ?;
        let new_to_account_amount = (&to_account.amount + &amount) ?;

        if new_from_account_amount.value.le(&BigDecimal::from(0i32)) {
            return Err(AccountProcessError::NotEnoughBalance(
                AccountIdWrapper::Id(from_account_id), backtrace()));
        }

        self.update_account_by_id_impl(&mut tx, &client_id, &from_account_id, new_from_account_amount).await ?;
        self.update_account_by_id_impl(&mut tx, &client_id, &to_account_id, new_to_account_amount).await ?;

        tx.commit().await ?;
        Ok(())
    }
}


impl AccountServiceImpl {

    async fn get_client_account_by_id_impl(
        &self, tx: &mut Transaction<'_, Postgres>,
        client_id: &ClientId, account_id: &AccountId,
    ) -> Result<Account, AccountProcessError> {

        debug!("### Loading user ACCOUNT [{account_id}] of client [{client_id}] from database");

        let res = sqlx::query_as(
            "select \
                 ID, IBAN, CLIENT_ID, NAME, \
                 AMOUNT, CUR, \
                 CREATED_AT, UPDATED_AT \
                 from ACCOUNTS \
                 where CLIENT_ID = $1 and ID = $2 ")
            .bind(client_id.inner_ref())
            .bind(account_id.inner_ref())
            .fetch_one(&mut **tx)
            .await
            // .map_err(Self::Error::Sqlx)?)
            .map_err(From::<sqlx::Error>::from);
        res
    }

    async fn get_client_account_by_iban_impl(
        &self, tx: &mut Transaction<'_, Postgres>,
        client_id: &ClientId, iban: &iban::Iban,
    ) -> Result<Account, AccountProcessError> {

        debug!("### Loading user ACCOUNT [{iban}] of client [{client_id}] from database");

        let res = sqlx::query_as(
            "select \
                 ID, IBAN, CLIENT_ID, NAME, \
                 AMOUNT, CUR, \
                 CREATED_AT, UPDATED_AT \
                 from ACCOUNTS \
                 where CLIENT_ID = $1 and IBAN = $2 ")
            .bind(client_id)
            .bind(&IbanRefWrapper(iban))
            .fetch_one(&mut **tx) // &*self.database_connection)
            .await
            // .map_err(Self::Error::Sqlx)?)
            .map_err(From::<sqlx::Error>::from);
        res
    }

    async fn update_account_by_iban_impl(
        &self, tx: &mut Transaction<'_, Postgres>,
        client_id: &ClientId, iban: &iban::Iban, amount: Amount,
    ) -> Result<(), AccountProcessError> {

        debug!("### Loading user ACCOUNT [{iban}] of client [{client_id}] from database");

        // let now: chrono::DateTime::<Utc> = chrono::DateTime::<Utc>::default();
        let now: chrono::DateTime<Utc> = chrono::Local::now().to_utc();
        let AmountParts { value: amount, currency } = amount.into_parts();

        let update_res = sqlx::query(
            " update ACCOUNTS \
                 set AMOUNT = $4, UPDATED_AT = $5 \
                 where CLIENT_ID = $1 and IBAN = $2 and CUR = $3 ")
            .bind(client_id)
            .bind(&IbanRefWrapper(iban))
            .bind(&currency)
            .bind(BigDecimalWrapper(amount)) // or &BigDecimalRefWrapper(&amount.value)
            .bind(&now)
            .execute(&mut **tx) // &*self.database_connection)
            .await
            .map_err(|err|AccountProcessError::Sqlx(err, backtrace())) ?;

        let updated_count = update_res.rows_affected();
        if updated_count == 1 {
            Ok(())
        } else {
            Err(AccountProcessError::Internal(
                anyhow::anyhow!("Error of updating account [{iban}] (updated_count: {updated_count}).")))
        }
    }

    async fn update_account_by_id_impl(
        &self, tx: &mut Transaction<'_, Postgres>,
        client_id: &ClientId, id: &AccountId, amount: Amount,
    ) -> Result<(), AccountProcessError> {

        debug!("### Loading user ACCOUNT [{id}] of client [{client_id}] from database");

        let now: chrono::DateTime<Utc> = chrono::Local::now().to_utc();
        let AmountParts { value: amount, currency } = amount.into_parts();

        let update_res = sqlx::query(
            " update ACCOUNTS \
                 set AMOUNT = $4, UPDATED_AT = $5 \
                 where CLIENT_ID = $1 and ID = $2 and CUR = $3 ")
            .bind(client_id)
            .bind(&id)
            .bind(&currency)
            .bind(BigDecimalWrapper(amount)) // or .bind(&BigDecimalRefWrapper(&amount.value))
            .bind(&now)
            .execute(&mut **tx) // &*self.database_connection)
            .await
            .map_err(|err|AccountProcessError::Sqlx(err, backtrace())) ?;

        let updated_count = update_res.rows_affected();
        if updated_count == 1 {
            Ok(())
        } else {
            Err(AccountProcessError::Internal(
                anyhow::anyhow!("Error of updating account [{id}] (updated_count: {updated_count}).")))
        }
    }

}


impl sqlx::FromRow<'_, sqlx_postgres::PgRow> for Account {
    fn from_row(row: &sqlx_postgres::PgRow) -> sqlx::Result<Self> {
        use sqlx::Row;
        use mvv_common::pg_column_name as col_name;

        let account = Account::new(account::new::Args {
            id: row.try_get(col_name!("ID")) ?,
            iban: row.try_get::<IbanWrapper,_>(col_name!("IBAN")) ?.0,
            client_id: row.try_get(col_name!("CLIENT_ID")) ?,
            name: row.try_get(col_name!("NAME")) ?,
            amount: Amount::new(
                row.try_get::<BigDecimalWrapper,_>(col_name!("AMOUNT")) ?.0,
                row.try_get(col_name!("CUR")) ?,
            ),
            created_at: row.try_get(col_name!("CREATED_AT")) ?,
            updated_at: row.try_get(col_name!("UPDATED_AT")) ?,
        });

        Ok(account)
    }
}
