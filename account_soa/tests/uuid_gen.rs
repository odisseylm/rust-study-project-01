
#[test]
#[ignore]
fn gen_uuid() {
    // uuid::uuid!();
    let uuid = uuid::Uuid::new_v4();
    println!("uuid: {uuid}");

    assert!(false, "To se output in console.")
}
