
use derive_more::Display;

#[derive(Debug, Display, Clone, serde::Deserialize)]
#[display(fmt = "PathUserId({})", user_id)]
pub struct UserId {
    user_id: String,
}


#[derive(Debug, Display, Clone, serde::Deserialize)]
#[display(fmt = "PathAccountId({})", account_id)]
pub struct AccountId {
    pub account_id: String,
}
