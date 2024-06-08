use crate::entities::prelude::UserId;
use crate::util::UncheckedResultUnwrap;
use super::dto::{Account as AccountDTO, Amount};


// #[static_init::constructor]
#[static_init::dynamic]
static TEMP_CURRENT_USER_ID: UserId = UserId::from_str("11").unchecked_unwrap();

struct AccountRest <AS: crate::service::account_service::AccountService> {
    account_service: AS,
}

impl<AS: crate::service::account_service::AccountService> AccountRest<AS> {
    async fn current_user_id(&self) -> UserId {
        TEMP_CURRENT_USER_ID.clone()
    }

    pub async fn get_current_user_accounts(&self) -> Result<Vec<AccountDTO>, anyhow::Error> {
        let accounts = self.account_service.get_user_accounts(
            self.current_user_id().await).await;

        let aa = accounts.map(|acs|acs.iter().map(move |ac| AccountDTO {
            id: ac.id.to_string(), // TODO: use moving
            user_id: ac.user_id.to_string(),
            amount: Amount { value: ac.amount.value.clone(), currency: ac.amount.currency.to_string() },
            created_at: ac.created_at,
            updated_at: ac.updated_at,
        }).collect::<Vec<_>>()) ?;
        Ok(aa)
    }
}
