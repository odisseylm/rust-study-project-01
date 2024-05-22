
// #[cfg(all(test, not(feature = "ignore_foo")))]
#[cfg(all(test, feature = "performance_tests"))]
mod arc_test {
    use std::backtrace::Backtrace;
    use std::time::SystemTime;
    use project01::util::{enable_backtrace, TestResultUnwrap};


    // RUST_TEST_THREADS=1
    static MUTEX: std::sync::Mutex<i32> = std::sync::Mutex::new(666);

    const N: u32 = 1000;

    #[test]
    // #[no_parallel]
    fn capture_backtrace_performance_simple_test() {
        enable_backtrace();

        let _guard = MUTEX.lock().unwrap();

        let start = SystemTime::now();

        let mut _bt: Backtrace = Backtrace::disabled();
        for _ in 1..N {
            _bt = Backtrace::force_capture();
        }

        let end = SystemTime::now();
        println!("spent {}mcs", end.duration_since(start).test_unwrap().as_micros() / (N as u128))
    }

    #[test]
    fn capture_backtrace_performance_test() {
        test_impl(N, "capture backtrace", ||{ let _ = Backtrace::force_capture(); });
    }

    #[test]
    fn capture_and_to_string_backtrace_performance_test() {
        test_impl(N, "capture backtrace and convert to string", ||{
            let bt = Backtrace::force_capture();
            bt.to_string();
        });
    }

    #[test]
    fn capture_and_to_display_backtrace_performance_test() {
        use std::fmt::Write;
        test_impl(N, "capture backtrace and convert to display string", ||{
            let bt = Backtrace::force_capture();
            let mut str = String::new();
            write!(str, "{}", bt).test_unwrap();
        });
    }

    #[test]
    fn capture_and_to_debug_backtrace_performance_test() {
        use std::fmt::Write;
        test_impl(N, "capture backtrace and convert to debug string", ||{
            let bt = Backtrace::force_capture();
            let mut str = String::new();
            write!(str, "{:?}", bt).test_unwrap();
        });
    }

    #[inline]
    fn test_impl<F: Fn()>(n: u32, label: &str, f: F) {
        let _guard = MUTEX.lock().unwrap();

        enable_backtrace();

        let start = SystemTime::now();

        for _ in 1..N {
            f();
        }

        let end = SystemTime::now();
        println!("{} => spent {}mcs", label, end.duration_since(start).test_unwrap().as_micros() / (n as u128))
    }

}
