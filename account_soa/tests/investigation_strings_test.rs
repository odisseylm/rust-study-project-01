use indoc::indoc;


#[test]
fn test_multiline_string_01_without_ident() {
    let a: &'static str = indoc! { "
        <div class=\"advice\">
            Raw strings are useful for some situations.
        </div>
        " };
    println!("{}", a);
}


#[test]
fn test_multiline_string_02_without_ident_and_without_linebreaks2() {
    let a = "\
        <div class=\"advice\">\
            Raw strings are useful for some situations.\
        </div>
        ";
    println!("{}", a);
}


// +++
#[test]
#[allow(non_snake_case)]
fn test_multiline_string_03_without_ident_and_with_linebreaks_as_ONE_LINE() {
    let a: &str = "\
        <div class=\"advice\">\n\
            Raw strings are useful for some situations.\n\
        </div>
        ";
    println!("{}", a);
}


#[test]
fn test_raw_multiline_string_01_with_idents() {
    let a: &str = r#"
        <div class="advice">
            Raw strings are useful for some situations.
        </div>
        "#;
    println!("{}", a);
}


// +++
#[test]
#[allow(non_snake_case)]
fn test_raw_multiline_string_02_without_idents_THE_BEST_in_most_cases() {
    let a: &str = indoc! { r#"
        <div class="advice">
            Raw strings are useful for some situations.
        </div>
        "# };
    println!("{}", a);
}
