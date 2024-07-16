use std::sync::Arc;
use mvv_common::entity::amount::Amount;
use crate::entity::prelude::{ Account, AccountId, UserId };


#[derive(thiserror::Error, Debug)]
pub enum AccountError {
    #[error("AccountNotFound")]
    AccountNotFound,
    #[error("Internal")]
    Internal,
}


// #[trait_variant::make(SendAccountService: Send)]
#[trait_variant::make(Send)]
// or #[async_trait] // https://github.com/dtolnay/async-trait#dyn-traits
pub trait AccountService: Send + Sync {
    async fn get_user_accounts(&self, user_id: UserId) -> Result<Vec<Account>, AccountError>;
    async fn get_user_account(&self, account_id: AccountId, user_id: UserId) -> Result<Account, AccountError>;
    async fn get_account(&self, account_id: AccountId, user_id: UserId) -> Result<Account, AccountError>;
}

pub struct AccountServiceImpl {
    pub database_connection: Arc<sqlx_postgres::PgPool>,
}

// ??? Hm... cannot use there AccountServiceSafe !?
impl AccountService for AccountServiceImpl {

    async fn get_user_accounts(&self, user_id: UserId) -> Result<Vec<Account>, AccountError> {
        use chrono::*;
        use core::str::FromStr;
        use crate::entity::account;

        let accounts = vec!(Account::new( account::new::Args {
            id: AccountId::from_str("345").unwrap(),
            user_id,
            amount: Amount::from_str("135.79 EUR").unwrap(),
            updated_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 22:29:57 +02:00").unwrap().to_utc(),
            created_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 15:31:09 +02:00").unwrap().to_utc(),
        }));
        Ok(accounts)
    }

    async fn get_user_account(&self, account_id: AccountId, user_id: UserId) -> Result<Account, AccountError> {
        use chrono::*;
        use core::str::FromStr;
        use crate::entity::account;

        let account = Account::new( account::new::Args {
            id: account_id,
            user_id,
            amount: Amount::from_str("136.79 EUR").unwrap(),
            updated_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 22:29:57 +02:00").unwrap().to_utc(),
            created_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 15:31:09 +02:00").unwrap().to_utc(),
        });
        Ok(account)
    }

    async fn get_account(&self, account_id: AccountId, user_id: UserId) -> Result<Account, AccountError> {
        use chrono::*;
        use core::str::FromStr;
        use crate::entity::account;

        let account = Account::new( account::new::Args {
            id: account_id,
            user_id,
            amount: Amount::from_str("137.79 EUR").unwrap(),
            updated_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 22:29:57 +02:00").unwrap().to_utc(),
            created_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 15:31:09 +02:00").unwrap().to_utc(),
        });
        Ok(account)
    }
}
