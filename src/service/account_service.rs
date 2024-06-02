use std::rc::Rc;
use crate::entities::account::Account;
use crate::entities::id::Id;


// TODO: temp
struct DatabaseConnection {
}


struct AccountService {
    database_connection: Rc<DatabaseConnection>,
}

impl AccountService {

    async fn find_account(account_id: Id) -> Account {
        todo!()
    }
}
