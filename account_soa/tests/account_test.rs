use chrono::{ FixedOffset, Utc };
use mvv_account_soa::entity::{
    prelude::{ AccountId, UserId },
    account::{ self, Account },
};
use mvv_common::entity::amount::Amount;
use mvv_common::test::{ TestResultUnwrap };
//--------------------------------------------------------------------------------------------------



#[test]
#[allow(unused_variables)]
fn readonly_field_test() {
    let account = account_01();

    let id = &account.id;
    // let mut id = &account.id;
    // id.0 = "443";

    let as_id: Result<&AccountId, _> = TryInto::<&AccountId>::try_into(&account.id);
    println!("### as_id: {:?}", as_id);

    // account.id = Id::from_str("54545").unwrap();

    use core::str::FromStr;
    let t = chrono::NaiveDateTime::from_str("2024-05-31T22:29:57").test_unwrap();
    println!("### t: {}", t);

    let t = chrono::NaiveDateTime::default();
    println!("### t: {}", t); // ### t: 1970-01-01 00:00:00

    // let t = chrono_tz::
    // let t = chrono::DateTime::<chrono_tz::Poland>::default();
    let t = chrono::DateTime::<FixedOffset>::default();
    println!("### t: {}", t); // 1970-01-01 00:00:00 +00:00

    let t = chrono::DateTime::<FixedOffset>::from_str("2024-05-31 22:29:57 +02:00").test_unwrap();
    println!("### t: {}", t);

    let t = chrono::DateTime::<FixedOffset>::from_str("2024-05-31T22:29:57 +02:00").test_unwrap();
    println!("### t: {}", t);

    // assert!(false, "To shoe stdout");
}

fn account_01() -> Account {
    let account = Account::new(account::new::Args {
        // id:
        id: AccountId::from_str("1").test_unwrap(),
        user_id: UserId::from_str("2").test_unwrap(),
        amount: Amount::from_str("123.44 USD").test_unwrap(),
        created_at: datetime_from_str("2022-05-31 10:29:30 +02:00"),
        updated_at: datetime_from_str("2024-05-31 22:29:57 +02:00"),
    });
    account
}

fn datetime_from_str(str: &str) -> chrono::DateTime<Utc> {
    use core::str::FromStr;
    // chrono::DateTime::<FixedOffset>::from_str(str).test_unwrap()
    chrono::DateTime::<FixedOffset>::from_str(str).test_unwrap().to_utc()
}


/*
/// Returned when `RefCell::try_borrow` fails.
pub struct BorrowError { _inner: () }

/// Returned when `RefCell::try_borrow_mut` fails.
pub struct BorrowMutError { _inner: () }

trait Borrow222<T> {
    /// Tries to immutably borrows the value. This returns `Err(_)` if the cell
    /// was already borrowed mutably.
    pub fn try_borrow(&self) -> Result<Ref<T>, BorrowError> { ... }

    /// Tries to mutably borrows the value. This returns `Err(_)` if the cell
    /// was already borrowed.
    pub fn try_borrow_mut(&self) -> Result<RefMut<T>, BorrowMutError> { ... }
}
*/


#[allow(dead_code, unused_mut, unused_variables)]
mod mutability_check {
    use std::borrow::Cow;
    use std::cell::RefCell;
    use mvv_common::test::TestSringOps;
    // use std::ops::{Deref, DerefMut };
    // use project01::util::TestResultUnwrap;


    #[test]
    fn test_qwerty() {
        let mut s = SSS { x: 123 };

        // let aa: Result<&i32, _> = RefCell::new(&s.x).try_into();
        // println!("aa: {:?}", aa);

        // let aa: Result<& mut i32, _> = RefCell::new(&s.x).try_into();
        // println!("aa: {:?}", aa);

        // assert!(false, "To show output");
    }


    #[derive(Debug)]
    struct SSS {
        x: i32,
    }

    #[derive(Debug)]
    struct SSS44 {
        // pub(&)
        pub x: i32,
    }
    // #[allow(non_camel_case_types)]
    // #[derive(Debug)]
    // #[readonly::make]
    // struct SSS_RO {
    //     x: i32,
    // }

    fn aaa() {
        let mut s = SSS { x: 123 };
        s.x = 124;

        use std::borrow::Borrow;

        let s = SSS { x: 123 };
        #[allow(noop_method_call)]
        let sb: &i32 = (&s.x).borrow();
        // let pm: & mut i32 = & mut s.x;

        let mut s = SSS { x: 123 };
        let pm: &mut i32 = &mut s.x;

        // let mut s = SSS { x: 123 };
        // let sb: &mut i32 = (& mut s.x).borrow_mut();
        // let sb: &mut i32 = (& mut s.x).borrow_mut();
    }


    trait TryToBorrow where Self: Sized {
        fn is_borrowable_mut(&self) -> bool;
    }

    impl<'a, T> TryToBorrow for &'a T {
        fn is_borrowable_mut(&self) -> bool { false }
    }

    impl<'a, T> TryToBorrow for &'a mut T {
        fn is_borrowable_mut(&self) -> bool { true }
    }

    enum RefOption<'a, T: Sized> {
        None,
        SomeImmutable(&'a T),
        SomeMutable(&'a mut T),
    }


    trait TryToBorrow33<R, F> where Self: Sized, R: Sized, F: FnOnce(&mut Self) -> &R {
        fn borrowable_mut33(&mut self, ) -> RefOption<R>;
    }


    // trait TryToBorrow2<T> where Self: Sized {
    //     fn is_borrowable2_mut(&self) -> bool;
    // }
    // impl<T> TryToBorrow2<T> for T {
    //     fn is_borrowable2_mut(&self) -> bool { false }
    // }


    #[test]
    fn aaaa() {
        let v = 123;
        assert_eq!((&v).is_borrowable_mut(), false);
        // assert_eq!((& mut v).is_borrowable_mut(), false);

        let mut v = 123;
        assert_eq!((&v).is_borrowable_mut(), false);
        assert_eq!((&mut v).is_borrowable_mut(), true);

        let mut v = 123;
        // (&v).deref();
        // (&mut v).deref_mut();

        // let rc = RefCell::new(SSS { x: 123});
        let mut rc: Box<SSS> = Box::new(SSS { x: 123 });
        // assert!(rc.as_ref().is_borrowable_mut(), "Should be accessed as mutable.");
        assert!(rc.as_mut().is_borrowable_mut(), "Should be accessed as mutable.");

        // assert!(rc.as_mut().is_borrowable_mut(), "Should be accessed as mutable.");
        // rc.also_ref(|ptr|{ assert!(ptr.is_borrowable_mut(), "Should be accessed as mutable.")});

        // let aaaa = rc.into() as &SSS;
        // let aaaa: &mut SSS = rc.into(); // as &mut SSS;
        // let aaaa: Result<& mut SSS, _> = rc.into(); // as &mut SSS;
        // let aaaa = rc.into()as &SSS;
        // let aaaa = <Box<SSS> as Into<&SSS>>::into(rc);
        // let aaaa = <Box<SSS> as From<&SSS>>::from(rc);
        // let aaaa: &SSS = rc.from();
        // let aaaa: &SSS = Box::<SSS>::from(rc);
        // let aaaa: Box<SSS> = Box::<SSS>::from(rc);
        // let aaaa: &SSS = rc.deref_mut();
        // let aaaa: &SSS = rc.deref();

        // rc.also_ref_mut(|v|{});

        // unsafe {
        //     rc.as_ptr().as_ref_mut().map(|v| v.borrow_mut());
        // }

        let mut rc: RefCell<SSS> = RefCell::new(SSS { x: 123 });
        // let aa: Ref<SSS> = rc.try_borrow().test_unwrap();

        // let mut fdfdf = rc.try_borrow_mut().test_unwrap();
        // let x_ref: &mut i32 = &mut fdfdf.x;
        // let x_ref: &mut i32 = &mut fdfdf.x;
        // *x_ref = 124;
        // forget(x_ref);

        // let mut fdfdf = rc.try_borrow_mut().map(|el| el.x).test_unwrap();
        // fdfdf = 124;

        // forget(fdfdf);

        // println!("### s: {:?}", fdfdf);
        // let aa: i32 = rc.try_borrow().map(|el|el.x).test_unwrap();
        // let aa = rc.try_borrow_mut().map(|ref el|&el.x);

        // let mut rc: RefCell<SSS_RO> = RefCell::new(SSS_RO { x: 123});
        // let mut fdfdf = rc.try_borrow_mut().map(|el|el.x).test_unwrap();
        // fdfdf = 124;
        // println!("### s: {:?}", fdfdf);

        let s = "eggplant".to_test_string();
        let s2 = "eggplant".to_test_string();
        assert_eq!(Cow::from(s), Cow::<'static, str>::Owned(s2));

        let mut v = 123;
        let mut v = "123";
        let cow: Cow<str> = Cow::from(v);
        let mut v = SSS { x: 123 };
        let mut v = v;
        let mut v = v;

        // // let cow = Cow::from(v);
        // let cow: Cow<'_, SSS> = Cow::Owned(v);
        // match cow {
        //     Cow::Borrowed(ref v) => { assert!(false) }
        //     Cow::Owned(ref v)    => { assert!(true)  }
        // };

        //assert!(false, "To see output");
    }


    // pub enum Movable<'a, B: ?Sized + 'a>
    //     // where B: core::marker::Unpin
    // {
    //     Borrowed(&'a B),
    //
    //     /// Owned data.
    //     Moved(<B as ToOwned>::Owned),
    // }
}