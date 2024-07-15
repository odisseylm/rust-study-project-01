



#[allow(unused_macros)]
macro_rules! say_hello {
    () => (
        println!("### Hello, world!");
    );
}
#[allow(unused_macros)]
macro_rules! create_function {
    ($func_name:ident) => (
        fn $func_name() {
            println!("You called {:?}()", stringify!($func_name));
        }
    );
}

#[allow(unused_macros)]
macro_rules! assert_equal_len {
    ($a:expr, $b:expr, $func:ident, $op:tt) => {
        assert!($a.len() == $b.len(),
                "{:?}: dimension mismatch: {:?} {:?} {:?}",
                stringify!($func),
                ($a.len(),),
                stringify!($op),
                ($b.len(),));
    };
}

#[allow(unused_macros)]
macro_rules! do_thing {
    (print $metavar:literal) => {
        println!("{}", $metavar)
    };
}
//
// do_thing!(print 3);  => println!("{}", 3);


#[allow(unused_macros)]
macro_rules! foo {
    (_ bool) => {
        println!("got bool");
    };
    (_ Result<i32>) => {
        println!("got Result<i32>");
    };
    (_ $tp:ty) => {
        println!("fallback to type: {}", stringify!($tp));
    };
    // ($($tp:tt)*) => {
    //     foo!(_ $($tp)*);
    // };
}

#[allow(unused_macros)]
macro_rules! foo2 {
    ($tp:ty) => {
        foo!(_ $tp);
    };
    (_ bool) => {
        println!("got bool");
    };
    (_ Result<i32>) => {
        println!("got Result<i32>");
    };
    (_ $tp:ty) => {
        println!("fallback to type: {}", stringify!($tp));
    };

}




#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    #[ignore]
    fn test_temp() {
        // say_hello!();
        // create_function!(rrr);
        // rrr();

        // fn local_fn() { println!("### local_fn") }
        // assert_equal_len!("a", "aa", local_fn, +);
        // assert_equal_len!("a", "a", local_fn, +);

        // foo!(true bool);
        foo!(_ bool);
        foo2!(_ bool);
        //foo!(bool);
        foo2!(bool);
    }
}
