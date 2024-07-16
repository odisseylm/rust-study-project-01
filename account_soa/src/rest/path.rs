
#[derive(Debug, derive_more::Display, Clone, serde::Deserialize)]
#[display(fmt = "PathUserId({})", user_id)]
pub struct UserId {
    user_id: String,
}


#[derive(Debug, derive_more::Display, Clone, serde::Deserialize)]
#[display(fmt = "PathAccountId({})", account_id)]
pub struct AccountId {
    pub account_id: String,
}

#[derive(Debug, derive_more::Display, Clone, serde::Deserialize)]
#[display(fmt = "PathClientId({})", client_id)]
pub struct ClientId {
    pub client_id: String,
}
