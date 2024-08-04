use std::f64::consts::PI;
use mvv_common::test::{ TestDisplayStringOps, TestOptionUnwrap };
// use tuple_length::TupLen;
use mvv_tuple_heter_iter;


#[test]
#[allow(unused_qualifications)]
fn test_123() {

    let tuple_v = (1, "str", "string".to_test_string(), std::f64::consts::PI);
    // T O D O: add such function
    let tuple_of_refs = (&tuple_v.0, &tuple_v.1, &tuple_v.2, &tuple_v.3, );

    use mvv_tuple_heter_iter::TupleAccess;

    println!("### tupl test => _0: {:?}", tuple_of_refs._0());
    println!("### tupl test => _1: {:?}", tuple_of_refs._1());
    println!("### tupl test => _2: {:?}", tuple_of_refs._2());
    println!("### tupl test => _3: {:?}", tuple_of_refs._3());
    println!("### tupl test => _4: {:?}", tuple_of_refs._4());
    println!("### tupl test => _5: {:?}", tuple_of_refs._5());
}


#[test]
#[allow(unused_qualifications)]
fn test_124() {

    let tuple_v = (1, "str", "string".to_test_string(), std::f64::consts::PI);
    // T O D O: add such function
    let tuple_of_refs = (&tuple_v.0, &tuple_v.1, &tuple_v.2, &tuple_v.3, );

    // Faked (really unused) variable to shut up Idea error notification.
    #[allow(dead_code, unused_variables)]
    let item = tuple_of_refs.0;

    mvv_tuple_heter_iter_macro::tuple_for_each_by_ref! ($item, tuple_of_refs, {
        println!("### test_124: {:?}", item); // How to fix 'item_ref' properly
    });
}


#[test]
fn test_for_each_in_tuple_by_ref() {
    use mvv_tuple_heter_iter::{ TupleAccess, TupleLen };

    let tuple_v = (123u8, "John", "Vovan".to_test_string(), PI);
    let tuple_of_refs = (&tuple_v.0, &tuple_v.1, &tuple_v.2, &tuple_v.3);

    {
        let mut res = Vec::<String>::new();

        // Faked (really unused) variable to shut up Idea error notification.
        #[allow(dead_code, unused_variables)]
            let item_ref22 = &tuple_of_refs.0;

        mvv_tuple_heter_iter_macro::tuple_for_each_by_ref! { item_ref22, tuple_of_refs, {
            println!("### tuple_iter_test_125: {:?}", item_ref22); // How to fix 'item_ref' properly??
            res.push(item_ref22.to_test_string());
        }}

        assert_eq!(res, vec!("123".to_test_string(), "John".to_test_string(), "Vovan".to_test_string(), PI.to_test_string()));
    }

    {
        let mut res = Vec::<String>::new();

        // Faked (really unused) variable to shut up Idea error notification.
        #[allow(dead_code, unused_variables)]
            let item_ref22 = &tuple_of_refs.0;

        mvv_tuple_heter_iter_macro::tuple_for_each_by_ref! { item_ref22, tuple_of_refs, 4, {
            println!("### tuple_iter_test_125: {:?}", item_ref22); // How to fix 'item_ref' properly??
            res.push(item_ref22.to_test_string());
        }}

        assert_eq!(res, vec!("123".to_test_string(), "John".to_test_string(), "Vovan".to_test_string(), PI.to_test_string()));
    }

    mvv_tuple_heter_iter::assert_tuple_len_is_4(&tuple_v);
    mvv_tuple_heter_iter::assert_tuple_len_is_4(&tuple_of_refs);

    assert_eq!(tuple_v.0, 123u8);
    assert_eq!(*tuple_v._0().test_unwrap(), 123u8);
    assert_eq!(tuple_v.len(), 4);
    assert_eq!(tuple_v.tuple_len(), 4);

    // assert!(false, "To see output");
}


#[test]
fn forr_tuple_iter_test_125() {
    use mvv_tuple_heter_iter::TupleLen;

    // let tuple_v = (123u8, "John", "Vovan".to_test_string(), PI);
    // let tuple_of_refs = (&tuple_v.0, &tuple_v.1, &tuple_v.2, &tuple_v.3);

    use forr::forr;

    // forr! { $ in (123u8, "John", "Vovan".to_test_string(), PI) {
    //
    //     }
    // }

    // forr! { $val:expr in [(1, i32), (Ok(2 + 4), Result<u8, ()>), (20.0, f32)] $: {
    //     println!("### in forr: {:?}", $val)
    //     }
    // }

    // forr! { $val:expr, $i:idx in [1, 2] $: {
    //     println!("### in forr: {:?}", $val:expr)
    //     }
    // }

    forr! { $val:expr, $i:idx in [(), (2,), (2,3), ""] $*
        println!("### in forr: {:?}", $val);
    }

    // forr! { $val:expr, $i:idx in [(2,), (2,3), ""] $*
    //     // println!("### in forr: {:?}", $val.tuple_len());
    //     println!("### in forr: {:?}", $val.to_test_string());
    // }

    let mut res = Vec::new();

    forr! { $val:expr, $i:idx in [(), (2,), (2,3)] $*
        println!("### in forr: {:?} - tuple len {}", $val, $val.tuple_len());
        res.push($val.tuple_len());
    }

    assert_eq!(res, vec!(0, 1, 2));

    // let tuple_v2 = ((), (2,), (2,3));
    // forr! { $val:expr, $i:idx in tuple_v2 $*
    //     println!("### in forr: {:?}", $val);
    // }

    // forr! { $i:idx, $val:expr in [1, 2]
    // forr! { $i:idx, ($val:expr, $vbl:ty) in [(1, i32), (2, i32)]
    // forr! { ($val:expr, $vbl:ty), $i:idx in [(1, i32), (2, i32)]
    // forr! { ($val:expr, $i:idx, $vbl:ty) in [(1, i32), (2, i32)]

    // assert!(false, "To see output");
}
