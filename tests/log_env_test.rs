use std::env;
use log::{debug, error, info, trace, warn};
use project01::util::TestResultUnwrap;

#[test]
#[ignore]
fn test_log_env_01() {

    // env_logger::init();
    env_logger::builder()
        .filter(None, log::LevelFilter::Debug)
        .init();

    use log::{ trace, debug, info, warn, error, log_enabled, Level };

    println!("-----------------------------\ntest_log_env\n");
    trace!("### test_log_env => trace: {}", 123);
    debug!("### test_log_env => debug: {}", 123);
    info! ("### test_log_env => info : {}", 123);
    warn! ("### test_log_env => warn : {}", 123);
    error!("### test_log_env => error: {}", 123);

    if log_enabled!(Level::Info) {
        let x = 3 * 4; // expensive computation
        info!("the answer was: {}", x);
    }

    println!("\n\n");
    assert!(false, "Test failure");
}

#[test]
#[ignore]
fn test_log_env_01_01_with_no_logging_stdout() {

    env_logger::builder()
        .is_test(true) // it hides log in stdout
        .filter(None, log::LevelFilter::Debug)
        .init();

    use log::{ trace, debug, info, warn, error };

    println!("-----------------------------\ntest_log_env\n");
    trace!("### test_log_env => trace: {}", 123);
    debug!("### test_log_env => debug: {}", 123);
    info! ("### test_log_env => info : {}", 123);
    warn! ("### test_log_env => warn : {}", 123);
    error!("### test_log_env => error: {}", 123);
}


#[test]
#[ignore]
fn test_log_env_02() {

    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    use log::{ trace, debug, info, warn, error };

    println!("-----------------------------\ntest_log_env\n");
    trace!("### test_log_env => trace: {}", 123);
    debug!("### test_log_env => debug: {}", 123);
    info! ("### test_log_env => info : {}", 123);
    warn! ("### test_log_env => warn : {}", 123);
    error!("### test_log_env => error: {}", 123);

    println!("\n\n");
    assert!(false, "Test failure");
}


#[test]
#[ignore]
fn test_log_env_03() {

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    use log::{ trace, debug, info, warn, error };

    println!("-----------------------------\ntest_log_env\n");
    trace!("### test_log_env => trace: {}", 123);
    debug!("### test_log_env => debug: {}", 123);
    info! ("### test_log_env => info : {}", 123);
    warn! ("### test_log_env => warn : {}", 123);
    error!("### test_log_env => error: {}", 123);

    println!("\n\n");
    assert!(false, "Test failure");
}


#[test]
#[ignore]
fn test_log4rs_01() {
    // See https://github.com/estk/log4rs/blob/main/docs/Configuration.md

    // log4rs::init_file("log4rs.yml", Default::default()).test_unwrap();
    log4rs::init_file("../test_resources/log4rs.yml", Default::default()).test_unwrap();

    println!("-----------------------------\ntest_log_env\n");
    trace!("### test_log => trace: {}", 123);
    debug!("### test_log => debug: {}", 123);
    info! ("### test_log => info : {}", 123);
    warn! ("### test_log => warn : {}", 123);
    error!("### test_log => error: {}", 123);

    println!("\n\n");
    assert!(false, "Test failure");
}


#[test]
#[ignore]
fn test_loading_env_from_env_file() {
    println!("MY_VAR1 = {:?}", env::var("MY_VAR1"));

    println!("\n\n");
    assert!(false, "Test failure");
}


#[allow(dead_code)]
enum Enum1 {
    Variant1,
    Variant2(i32),
    Variant3(i32, & 'static str),
    Variant4 { _x: i32 },
    Variant5 { _x: i32, _y: i32 },
}


#[test]
fn test_matches_macro() {
    let v1: Option<i32> = Some(123);

    let v11 = match v1 {
        None    => { false }
        Some(_) => { true  }
    };
    println!("v11: {:?}", v11);

    let v21 = matches!(v1, Some(_));
    println!("v21: {:?}", v21);

    // let v22 = matches!(v1, Err(_));
    // println!("v22: {:?}", v22);
    //
    // let v23 = matches!(v1, Enum1::Variant1);
    // println!("v23: {:?}", v23);
    //
    // let v24 = matches!(v1, Enum1::Variant2(_));
    // println!("v24: {:?}", v24);
}
