

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
    println!("test_ignorance_02");
}
