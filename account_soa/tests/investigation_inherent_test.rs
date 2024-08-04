


mod m1 {

    //use inherent::inherent;

    pub trait Trait {
        fn f(self);
    }

    pub struct Struct;

    // #[inherent]
    #[inherent::inherent]
    impl Trait for Struct {
        pub fn f(self) {}
    }
}

#[test]
#[allow(unused_imports)]
fn test_01_both_imported_but_no_ambiguous_problem() {
    use m1::Trait;
    // m1::Trait is not in scope, but method can be called.
    m1::Struct.f();
}

#[test]
fn test_01() {
    // m1::Trait is not in scope, but method can be called.
    m1::Struct.f();
}
