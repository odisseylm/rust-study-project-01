use std::f64::consts::PI;
use project01::util::TestOptionUnwrap;
// use tuple_length::TupLen;
use tuple_heter_iter;


#[test]
fn tuple_iter_test_125() {
    use tuple_heter_iter::TupleOps;

    let tuple_v = (123u8, "John", "Vovan".to_string(), PI);
    let tuple_of_refs = (&tuple_v.0, &tuple_v.1, &tuple_v.2, &tuple_v.3);

    tuple_heter_iter_macro::for_each_in_tuple_by_ref_2! (tuple_of_refs, {
        //noinspection RsUnresolvedPath
        println!("### tuple_iter_test_125: {:?}", item_ref); // How to fix 'item_ref' properly
    });

    tuple_heter_iter::assert_tuple_len_is_4(&tuple_v);
    tuple_heter_iter::assert_tuple_len_is_4(&tuple_of_refs);

    assert_eq!(tuple_v.0, 123u8);
    assert_eq!(*tuple_v._0().test_unwrap(), 123u8);
    assert_eq!(tuple_v.len(), 4);
    assert_eq!(tuple_v.tuple_len(), 4);

    // assert!(false, "To see output");
}

