

#[test]
fn test_123() {

    let tuple_v = (1, "str", "string".to_string(), std::f64::consts::PI);
    // TODO: add such function
    let tuple_of_refs = (tuple_v.0, tuple_v.1, tuple_v.2, tuple_v.3, );

    use tuple_heter_iter::TupleAccess;

    println!("### tupl test => _0: {:?}", tuple_of_refs._0());
    println!("### tupl test => _1: {:?}", tuple_of_refs._1());
    println!("### tupl test => _2: {:?}", tuple_of_refs._2());
    println!("### tupl test => _3: {:?}", tuple_of_refs._3());
    println!("### tupl test => _4: {:?}", tuple_of_refs._4());
    println!("### tupl test => _5: {:?}", tuple_of_refs._5());

    // assert!(false, "To see output");
}


#[test]
fn test_124() {

    let tuple_v = (1, "str", "string".to_string(), std::f64::consts::PI);
    // TODO: add such function
    let tuple_of_refs = (tuple_v.0, tuple_v.1, tuple_v.2, tuple_v.3, );

    // use tuple_heter_iter::TupleAccess;

    // Faked (really unused) variable to shut up Idea error notification.
    #[allow(dead_code, unused_variables)]
    let item = &tuple_of_refs.0;

    tuple_heter_iter_macro::tuple_for_each_by_ref! ($item, tuple_of_refs, {
        println!("### test_124: {:?}", item); // How to fix 'item_ref' properly
    });

    // assert!(false, "To see output");
}
