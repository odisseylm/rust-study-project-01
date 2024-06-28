
// #[cfg(something)]
mod tuples22 {
    tuple_heter_iter::generate_all_tuple_ops!(12);
}

#[cfg(something)]
mod tuples22 {
}




#[test]
fn aaaa_123() {

    let tuple_v = (1, "str", "string".to_string(), std::f64::consts::PI);
    // TODO: add such function
    let tuple_of_refs = (tuple_v.0, tuple_v.1, tuple_v.2, tuple_v.3, );

    //noinspection RsUnresolvedPath
    use tuples22::TupleOps;

    println!("### tupl test => _0: {:?}", tuple_of_refs._0());
    println!("### tupl test => _1: {:?}", tuple_of_refs._1());
    println!("### tupl test => _2: {:?}", tuple_of_refs._2());
    println!("### tupl test => _3: {:?}", tuple_of_refs._3());
    println!("### tupl test => _4: {:?}", tuple_of_refs._4());
    println!("### tupl test => _5: {:?}", tuple_of_refs._5());

    assert!(false, "To see output");
}