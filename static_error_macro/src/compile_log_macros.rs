

// #[allow(dead_code)]
// #[allow(unused_macros)]
// macro_rules! compile_warn {
//     ($($x:tt)*) => { println!($($x)*) }
// }


// copy of https://stackoverflow.com/questions/71985357/whats-the-best-way-to-write-a-custom-format-macro
// Thanks a lot to nebulaeandstars :-)
//
// #[macro_export] // Not allowed there => 'cannot export macro_rules! macros from a `proc-macro` crate type currently'
#[allow(dead_code)]
#[allow(unused_macros)]
macro_rules! compile_warn {

    ($fmt_str:literal) => {{
        eprintln!(concat!("Compile WARN [{}:{}] ", $fmt_str), file!(), line!());
    }};

    ($fmt_str:literal, $($args:expr),*) => {{
        eprintln!(concat!("Compile WARN [{}:{}] ", $fmt_str), file!(), line!(), $($args),*);
    }};
}


/*
macro_rules! compile_warn {
    ($($x:tt)*) => { println!($($x)*) }
}

// see https://www.reddit.com/r/rust/comments/tzmn6d/best_way_to_make_a_conditional_println_macro/
#[cfg(debug_assertions)]
macro_rules! debug_println {
    ($($x:tt)*) => { println!($($x)*) }
}

#[cfg(not(debug_assertions))]
macro_rules! debug_println {
    ($($x:tt)*) => ({})
}


macro_rules! vprint {
    ($($x:tt)*) => { if VERBOSE { println!($($x)*); } }
}
*/
