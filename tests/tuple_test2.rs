use std::f64::consts::PI;
use project01::util::TestOptionUnwrap;
// use tuple_length::TupLen;
use tuple_heter_iter;


#[test]
fn tuple_iter_test_125() {
    use tuple_heter_iter::TupleOps;

    let tuple_v = (123u8, "John", "Vovan".to_string(), PI);
    let tuple_of_refs = (&tuple_v.0, &tuple_v.1, &tuple_v.2, &tuple_v.3);

    tuple_heter_iter_macro::for_each_in_tuple_by_ref_2! ($item_ref, tuple_of_refs, {
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


#[test]
fn forr_tuple_iter_test_125() {
    use tuple_heter_iter::TupleOps;

    let tuple_v = (123u8, "John", "Vovan".to_string(), PI);
    let tuple_of_refs = (&tuple_v.0, &tuple_v.1, &tuple_v.2, &tuple_v.3);

    use forr::forr;

    // forr! { $ in (123u8, "John", "Vovan".to_string(), PI) {
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
    //     println!("### in forr: {:?}", $val.to_string());
    // }

    forr! { $val:expr, $i:idx in [(), (2,), (2,3)] $*
        println!("### in forr: {:?} - tuple len {}", $val, $val.tuple_len());
    }

    // let tuple_v2 = ((), (2,), (2,3));
    // forr! { $val:expr, $i:idx in tuple_v2 $*
    //     println!("### in forr: {:?}", $val);
    // }

    // forr! { $i:idx, $val:expr in [1, 2]
    // forr! { $i:idx, ($val:expr, $vbl:ty) in [(1, i32), (2, i32)]
    // forr! { ($val:expr, $vbl:ty), $i:idx in [(1, i32), (2, i32)]
    // forr! { ($val:expr, $i:idx, $vbl:ty) in [(1, i32), (2, i32)]

    assert!(false, "To see output");
}
