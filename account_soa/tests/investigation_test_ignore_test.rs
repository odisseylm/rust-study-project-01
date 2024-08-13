
// ++
// #[cfg_attr(not(feature = "expensive_tests"), ignore)]


// #[cfg(all(test, feature = "performance_tests"))]
mod arc_test {

    #[test]
    fn test_ignorance_01() {
        println!("test_ignorance_01");
    }
}



use ::macro_rules_attribute::apply;

macro_rules! special_integration_test {( $fn:item ) => (
    // #[test]
    #[cfg_attr(not(feature = "special-integration-tests"),
        ignore,
    )]
    $fn
)} pub(in crate) use special_integration_test;


#[test]
/// This is ignored unless `--feature special-integration-test` is added to `cargo test`
#[apply(special_integration_test)]
fn test_ignorance_02() {
    println!("### test_ignorance_02");
    // assert!(false, "Test failure to see that test is picked up")
}

#[test]
#[cfg(special_integration_test)]
fn integration_test_222() {
    println!("### integration_test_222");
    assert!(false, "Test failure to see that test is picked up")
}

#[test]
// #[cfg_attr(not(feature = "test-type-two"), ignore)]
#[cfg_attr(not(feature = "test-type-two"), ignore)]
fn integration_test_223() {
    println!("### integration_test_223");
    assert!(false, "Test failure to see that test is picked up")
}

#[test]
#[cfg_attr(not(any(feature = "test-type-two", feature = "test-type-one")), ignore)]
fn integration_test_224() {
    println!("### integration_test_223");
    assert!(false, "Test failure to see that test is picked up")
}
