use std::ffi::OsString;
use mvv_common::backtrace::is_anyhow_backtrace_enabled;

#[test]
#[ignore]
fn test_is_anyhow_backtrace_enabled() {
    mvv_common::backtrace::enable_backtrace();
    // project01::util::disable_backtrace(); // it ?works? in test under Idea, BUT does not work under `cargo` ???

    is_anyhow_backtrace_enabled();
    is_anyhow_backtrace_enabled();

    let is_anyhow_bt_enabled = is_anyhow_backtrace_enabled();
    println!("is_anyhow_bt_enabled: {}", is_anyhow_backtrace_enabled());

    let bt_var_os = std::env::var_os("RUST_BACKTRACE").unwrap_or(OsString::new());
    let rust_backtrace_enabled = bt_var_os.to_str().map(|s| s == "1" || s == "full").unwrap_or(false);

    // assert_eq!(is_anyhow_bt_enabled, true, "anyhow backtrace is not enabled")
    assert_eq!(is_anyhow_bt_enabled, rust_backtrace_enabled, "anyhow backtrace is not enabled")
}
