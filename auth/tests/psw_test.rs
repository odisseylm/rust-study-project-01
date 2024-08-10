use mvv_auth::{clear_string_chars, SecureString};
use mvv_auth::util::test_unwrap::{TestResultUnwrap, TestSringOps};
//--------------------------------------------------------------------------------------------------


#[test]
fn clear_string_chars_test() {
    println!("---------------------------------------------------------------");

    let mut str = "qwerty".to_owned();
    println!("{} / {}", str, str.len());
    clear_string_chars(&mut str);
    println!("{} / {}", str, str.len());
    assert_eq!(str, "\0\0\0\0\0\0");

    let mut str = "Вован".to_owned();
    println!("{} / {}", str, str.len());
    clear_string_chars(&mut str);
    println!("{} / {}", str, str.len());
    assert_eq!(str, "\0\0\0\0\0\0\0\0\0\0");

    println!("---------------------------------------------------------------");
    // assert!(false, "Error to see console");
}


#[test]
#[ignore] // Impossible to write test since heap is already reallocated/taken
          // by some other code (even in single test!) => it contains new (random) data.
fn password_string_cleaning_after_drop_test() {
    let as_raw_ptr: *const u8;
    let len: usize;

    let mut as_raw_bytes = Vec::<u8>::new();
    let mut as_raw_bytes_after_cleaning = Vec::<u8>::new();

    {
        let psw_str: SecureString = "qwerty".into();
        assert_eq!("Secure[...]", psw_str.to_test_display_string());
        assert_eq!("Secure[...]", psw_str.to_test_debug_string());

        // let aa = unsafe { psw_str.as_bytes().as_ptr_range() };
        let as_bytes = psw_str.as_bytes();
        len = as_bytes.len();
        as_raw_ptr = as_bytes.as_ptr();

        for i in 0..len {
            as_raw_bytes.push(unsafe { *as_raw_ptr.add(i) });
        }
        assert_eq!(
            std::str::from_utf8(as_raw_bytes.as_slice()).test_unwrap(),
            "qwerty");
    }

    for i in 0..len {
        as_raw_bytes_after_cleaning.push(unsafe { *as_raw_ptr.add(i) });
    }
    println!("After attempt to read freed heap memory");

    assert_ne!(
        as_raw_bytes_after_cleaning.as_slice(),
        b"qwerty");

    // Impossible to write test since heap is already reallocated/taken
    // by some other code (even in single test!) => it contains new (random) data.
    //
    // assert_eq!(
    //     as_raw_bytes_after_cleaning.as_slice(),
    //     b"\0\0\0\0\0\0");

    // assert!(false, "To see console output");
}