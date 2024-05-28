


#[allow(unused_macros, dead_code)]
macro_rules! compile_log_level_error { () => (1) }
#[allow(unused_macros, dead_code)]
macro_rules! compile_log_level_warn  { () => (2) }
#[allow(unused_macros, dead_code)]
macro_rules! compile_log_level_info  { () => (3) }
#[allow(unused_macros, dead_code)]
macro_rules! compile_log_level_debug { () => (4) }
#[allow(unused_macros, dead_code)]
macro_rules! compile_log_level_trace { () => (5) }


#[allow(unused_macros, dead_code)]
macro_rules! compile_log_level { () => { compile_log_level_warn!() } }


// It is not called 'compile_warn' because there is no support for 'lint', for allow/warn macros.
// See possible problems at 'Pre-RFC - Add compile_warning! macro' https://internals.rust-lang.org/t/pre-rfc-add-compile-warning-macro/9370
//
//
// copy of https://stackoverflow.com/questions/71985357/whats-the-best-way-to-write-a-custom-format-macro
// Thanks a lot to nebulaeandstars :-)
//
// #[macro_export] // Not allowed there => 'cannot export macro_rules! macros from a `proc-macro` crate type currently'
#[allow(unused_macros, dead_code)]
macro_rules! compile_log_warn {

    ($fmt_str:literal) => {{
        if compile_log_level!() >= compile_log_level_warn!() { eprintln!(concat!("Compile WARN  [{}:{}] ", $fmt_str), file!(), line!()); };
    }};

    ($fmt_str:literal, $($args:expr),*) => {{
        if compile_log_level!() >= compile_log_level_warn!() { eprintln!(concat!("Compile WARN  [{}:{}] ", $fmt_str), file!(), line!(), $($args),*); };
    }};
}

#[allow(unused_macros, dead_code)]
macro_rules! compile_log_error {
    ($fmt_str:literal) => {{
        if compile_log_level!() >= compile_log_level_error!() { eprintln!(concat!("Compile ERROR [{}:{}] ", $fmt_str), file!(), line!()); };
    }};
    ($fmt_str:literal, $($args:expr),*) => {{
        if compile_log_level!() >= compile_log_level_error!() { eprintln!(concat!("Compile ERROR [{}:{}] ", $fmt_str), file!(), line!(), $($args),*); };
    }};
}

#[allow(unused_macros, dead_code)]
macro_rules! compile_log_info {
    ($fmt_str:literal) => {{
        if compile_log_level!() >= compile_log_level_info!() { println!(concat!("Compile INFO  [{}:{}] ", $fmt_str), file!(), line!()); };
    }};
    ($fmt_str:literal, $($args:expr),*) => {{
        if compile_log_level!() >= compile_log_level_info!() { println!(concat!("Compile INFO  [{}:{}] ", $fmt_str), file!(), line!(), $($args),*); };
    }};
}

#[allow(unused_macros, dead_code)]
macro_rules! compile_log_debug {
    ($fmt_str:literal) => {{
        if compile_log_level!() >= compile_log_level_debug!() { println!(concat!("Compile DEBUG [{}:{}] ", $fmt_str), file!(), line!()); };
    }};
    ($fmt_str:literal, $($args:expr),*) => {{
        if compile_log_level!() >= compile_log_level_debug!() { println!(concat!("Compile DEBUG [{}:{}] ", $fmt_str), file!(), line!(), $($args),*); };
    }};
}

#[allow(unused_macros, dead_code)]
macro_rules! compile_log_trace {
    ($fmt_str:literal) => {{
        if compile_log_level!() >= compile_log_level_trace!() { println!(concat!("Compile TRACE [{}:{}] ", $fmt_str), file!(), line!()); };
    }};
    ($fmt_str:literal, $($args:expr),*) => {{
        if compile_log_level!() >= compile_log_level_trace!() { println!(concat!("Compile TRACE [{}:{}] ", $fmt_str), file!(), line!(), $($args),*); };
    }};
}



/*
macro_rules! compile_log_warn {
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
