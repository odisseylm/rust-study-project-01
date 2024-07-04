use crate::entities::user::{User, UserId};


trait UserService {
    async fn get_user(user_id: UserId) -> Result<User, anyhow::Error>;
}


struct UserServiceImpl {

}

impl UserService for UserServiceImpl {
    async fn get_user(user_id: UserId) -> Result<User, anyhow::Error> {
        Ok(User {
            id: user_id,
            username: "Cheburan".to_string(),
        })
    }
}
