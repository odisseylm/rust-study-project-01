
pub trait PasswordComparator {
    fn passwords_equal(user_password: &str, credentials_password: &str) -> bool;
}


#[derive(Clone)]
pub struct PlainPasswordComparator;
impl PasswordComparator for PlainPasswordComparator {
    fn passwords_equal(user_password: &str, credentials_password: &str) -> bool {
        user_password == credentials_password
    }
}
