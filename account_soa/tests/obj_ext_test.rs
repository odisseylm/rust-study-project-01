
use mvv_common::obj_ext::*;


#[test]
fn test_also() {
    println!("var: {}", true);
    println!("var: {}", true.also(|v| println!("from 'also' (v: {})", v)));
}

#[test]
fn test_also_ref() {
    let val: bool = true;
    let val_ref: &bool = &val;

    println!("var: {}", val_ref);
    println!("var: {}", val_ref.also(|v|{
        println!("from 'also' (v: {})", v);
        // let vv: bool = v; // compile error: mismatched types, expected `bool`, found `&&bool`
        // println!("from 'also' (vv: {})", vv);
        let vv: &bool = v;
        println!("from 'also' (vv: {})", vv);
    }));
    println!("var: {}", val_ref.also_ref(|v|{
        println!("from 'also' (v: {})", v);
        // let vv: bool = v; // compile error: mismatched types, expected `bool`, found `&bool`
        // println!("from 'also' (vv: {})", vv);
        let vv: &bool = v;
        println!("from 'also' (vv: {})", vv);
    }));
}


