
struct AAA {
    val1: &'static str,
    val2: i32,
}

impl AAA {
    //noinspection RsUnresolvedPath // for 'item_ref'
    fn fn1(&self) {
        let internal_var = "internal 987";
        use static_error_macro::for_each_by_ref;

        for_each_by_ref! { self.val1, (self.val2), {
            println!("print from for_each_by_ref2 {{ internal_var: {:?} }}", internal_var);
            println!("print from for_each_by_ref2 {{ iterated value: {:?} }}", item_ref);
        }}
    }
}


#[test]
//noinspection RsUnresolvedPath // for 'item_ref'
fn test_1_for_each_by_ref() {

    use static_error_macro::for_each_by_ref;
    // for_each_by_ref2!("fdf");

    let s = AAA {
        val1: "John",
        val2: 567,
    };

    // to test access from for each body
    let internal_var = "internal 789";

    let mut result = Vec::<String>::new();

    let _var_before_for_each_by_ref2 = 345;
    for_each_by_ref! { s.val1, s.val2, {
        println!("### print from for_each_by_ref2 {{ internal_var: {:?} }}", internal_var);
        println!("### print from for_each_by_ref2 {{ iterated value: {:?} }}", item_ref);
        result.push(item_ref.to_string());
    }}
    let _var_after_for_each_by_ref2 = 346;

    assert_eq!(result, vec!("John", "567"));

    // just compilation test
    s.fn1();
}


#[test]
//noinspection RsUnresolvedPath // for 'item_ref'
fn test_literals_for_each_by_ref() {

    use static_error_macro::for_each_by_ref;

    // to test access from for each body
    let internal_var = "internal 789";

    let mut result = Vec::<String>::new();

    for_each_by_ref! { "John", 568, {
        println!("### print from for_each_by_ref2 {{ internal_var: {:?} }}", internal_var);
        println!("### print from for_each_by_ref2 {{ iterated value: {:?} }}", item_ref);
        result.push(item_ref.to_string());
    }}

    assert_eq!(result, vec!("John", "568"));
}
