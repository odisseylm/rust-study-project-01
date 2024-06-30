
struct AAA {
    val1: &'static str,
    val2: i32,
}

impl AAA {
    fn fn1(&self) {
        let internal_var = "internal 987";
        use tuple_heter_iter_macro::for_each_by_ref;

        // Faked (really unused) variable to shut up Idea error notification.
        #[allow(dead_code, unused_variables)]
        let item456 = &self.val1;

        for_each_by_ref! { item456, self.val1, (self.val2), {
            println!("print from for_each_by_ref2 {{ internal_var: {:?} }}", internal_var);
            println!("print from for_each_by_ref2 {{ iterated value: {:?} }}", item456);
        }}
    }
}


#[test]
fn test_1_for_each_by_ref() {

    use tuple_heter_iter_macro::for_each_by_ref;
    // for_each_by_ref2!("fdf");

    let s = AAA {
        val1: "John",
        val2: 567,
    };

    // to test access from 'for each' body
    let internal_var = "internal 789";

    let mut result = Vec::<String>::new();

    // Faked (really unused) variable to shut up Idea error notification.
    #[allow(dead_code, unused_variables)]
    let item_ref = &s.val1;

    let _var_before_for_each_by_ref2 = 345;
    for_each_by_ref! { $item_ref, s.val1, s.val2, {
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
fn test_literals_for_each_by_ref() {

    use tuple_heter_iter_macro::for_each_by_ref;

    // to test access from 'for each' body
    let internal_var = "internal 789";

    let mut result = Vec::<String>::new();

    // Faked (really unused) variable to shut up Idea error notification.
    #[allow(dead_code, unused_variables)]
    let item_ref = &"";

    for_each_by_ref! { $item_ref, "John", 568, {
        println!("### print from for_each_by_ref2 {{ internal_var: {:?} }}", internal_var);
        println!("### print from for_each_by_ref2 {{ iterated value: {:?} }}", item_ref);
        result.push(item_ref.to_string());
    }}

    assert_eq!(result, vec!("John", "568"));
}

#[allow(dead_code)]
const fn tuple_len<T1,T2,T3>(_tuple: &(T1,T2,T3)) -> usize {
    3
}


#[test]
fn test_for_each_in_tuple_by_ref() {

    macro_rules! tuple_size {
        () => (3)
    }

    use tuple_heter_iter_macro::tuple_for_each_by_ref;

    // to test access from 'for each' body
    let internal_var = "internal 789";

    // const tuple_var: (&str, i32, &str) = ("John", 568, "Smith");
    let tuple_var: (&str, i32, &str) = ("John", 568, "Smith");

    let mut result = Vec::<String>::new();

    // use tuple_len::tuple_len;
    use tuple_heter_iter::TupleLen;
    // use tuple_heter_iter::assert_tuple_len_is_63;
    #[allow(unused_imports)]
    use tuple_heter_iter::assert_tuple_len_is_15;
    // println!("### tuple length: {}", tuple_len!(tuple_var));
    println!("### tuple length: {}", tuple_var.tuple_len());
    println!("### tuple_size: {}", tuple_size!());

    // use tuple_len::TupleLen;
    // use tuple_length::TupLen;
    // static_assertions::const_assert_eq!(tuple_len!(tuple_var), 3);
    // static_assertions::const_assert_eq!(tuple_var.len(), 3);
    // static_assertions::const_assert_eq!(tuple_len(&tuple_var), 3);

    // Faked (really unused) variable to shut up Idea error notification.
    #[allow(dead_code, unused_variables)]
    let iteeem_var = &tuple_var.0;

    // for_each_in_tuple_by_ref! { tuple_var, tuple_len!(tuple_var), {
    // for_each_in_tuple_by_ref! { tuple_var, tuple_size!(), {
    tuple_for_each_by_ref! { $iteeem_var, tuple_var, 3, {
        println!("### print from for_each_by_ref2 {{ internal_var: {:?} }}", internal_var);
        println!("### print from for_each_by_ref2 {{ iterated value: {:?} }}", iteeem_var);
        result.push(iteeem_var.to_string());
    }}

    assert_eq!(result, vec!("John", "568", "Smith"));

    // assert!(false, "To see output");
}


#[test]
fn test_const_condition() {
    let t = ("1", 2, );

    println!("t.0: {:?}", t.0);
    println!("t.1: {:?}", t.1);

    // #[cfg(not(foo))]
    #[cfg(yryryuru)]
    #[cfg_attr(all(1==2))]
    {
        println!("t.2: {:?}", t.2);
    }

    // #[cfg(1 = 2)]
    // {
    //     println!("t.2: {:?}", t.2);
    // }
    //
    // if cfg!(foo) {
    //     println!("t.2: {:?}", t.2);
    // }


}