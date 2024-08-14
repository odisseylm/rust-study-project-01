

pub trait PasswordComparator {
    fn passwords_equal(&self, user_password: &str, credentials_password: &str) -> bool;
}


#[derive(Debug, Clone)]
pub struct PlainPasswordComparator;
impl PlainPasswordComparator {
    pub fn new() -> Self {
        PlainPasswordComparator
    }
}
impl PasswordComparator for PlainPasswordComparator {
    fn passwords_equal(&self, user_password: &str, credentials_password: &str) -> bool {
        user_password == credentials_password
    }
}



//--------------------------------------------------------------------------------------------------
