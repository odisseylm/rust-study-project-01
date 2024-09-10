use core::fmt;

pub trait PasswordComparator : fmt::Debug {
    fn passwords_equal(&self, user_psw_or_psw_hash: &str, credentials_password: &str) -> bool;
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
