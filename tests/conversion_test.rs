use core::fmt::Debug;

#[derive(Debug)]
#[derive(Default)]
struct User {
    pub name: & 'static str,
}

#[derive(Debug)]
#[derive(Default)]
struct Moderator {
    user: User,
}


fn do_smt_with_user(user: &User) {
    println!("Username: {}", user.name)
}

// !!! Not compiled !!!
// fn do_smt_with_user_using_as_ref_2(&user: AsRef<User>) {
//     println!("Username: {}", user.as_ref().name)
// }

fn do_smt_with_user_using_as_ref<U: AsRef<User>>(user: U) {
    println!("Username: {}", user.as_ref().name)
}

impl AsRef<User> for User {   fn as_ref(&self) -> &User { &self }   }
impl AsRef<User> for Moderator {
    fn as_ref(&self) -> &User { &self.user }
}


#[test]
fn test_as_ref_01() {
    let user = User { name: "Vovan" };
    let moderator = Moderator { user: User { name: "John" } };

    do_smt_with_user(&user);
    do_smt_with_user(&moderator.user);
    do_smt_with_user(&moderator.as_ref());

    do_smt_with_user_using_as_ref(user);
    do_smt_with_user_using_as_ref(moderator);
}
