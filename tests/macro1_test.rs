

// Seems tests are not launched automatically from' macro' subproject (at least under Idea).
// As quick fix it is included there to be picked up.
//
include!("./../static_error_macro/tests/macro1_test.rs");

#[test]
fn to_have_idea_run_tests_menu_on_this_file() { assert_eq!(1, 1); }
