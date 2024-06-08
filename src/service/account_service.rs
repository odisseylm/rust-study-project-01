use std::rc::Rc;
use std::sync::Arc;
use crate::entities::amount::Amount;
use crate::entities::prelude::{ Account, AccountId, UserId };


// TODO: temp
struct DatabaseConnection {
}



#[derive(thiserror::Error, Debug)]
pub enum AccountsError {
    #[error("AccountNotFound")]
    AccountNotFound,
    #[error("Internal")]
    Internal,
}


// #[trait_variant::make(SendAccountService: Send)]
#[trait_variant::make(Send)]
pub trait AccountService {
    async fn get_user_accounts(&self, user_id: UserId) -> Result<Vec<Account>, AccountsError>;
    async fn get_user_account(&self, account_id: AccountId, user_id: UserId) -> Result<Account, AccountsError>;
    async fn get_account(&self, account_id: AccountId, user_id: UserId) -> Result<Account, AccountsError>;
}

pub struct AccountServiceImpl {
    database_connection: Arc<DatabaseConnection>,
}

// ??? Hm... cannot use there AccountServiceSafe !?
impl AccountService for AccountServiceImpl {

    async fn get_user_accounts(&self, user_id: UserId) -> Result<Vec<Account>, AccountsError> {
        use chrono::*;
        use core::str::FromStr;
        use crate::entities::account;

        let accounts = vec!(Account::new( account::new::Args {
            id: AccountId::from_str("345").unwrap(),
            user_id,
            amount: Amount::from_str("135.79 EUR").unwrap(),
            updated_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 22:29:57 +02:00").unwrap().to_utc(),
            created_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 15:31:09 +02:00").unwrap().to_utc(),
        }));
        Ok(accounts)
    }

    async fn get_user_account(&self, account_id: AccountId, user_id: UserId) -> Result<Account, AccountsError> {
        use chrono::*;
        use core::str::FromStr;
        use crate::entities::account;

        let account = Account::new( account::new::Args {
            id: account_id,
            user_id,
            amount: Amount::from_str("136.79 EUR").unwrap(),
            updated_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 22:29:57 +02:00").unwrap().to_utc(),
            created_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 15:31:09 +02:00").unwrap().to_utc(),
        });
        Ok(account)
    }

    async fn get_account(&self, account_id: AccountId, user_id: UserId) -> Result<Account, AccountsError> {
        use chrono::*;
        use core::str::FromStr;
        use crate::entities::account;

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
