


#[macro_export] macro_rules! pg_column_name {
    // postgres needs lowercase (Oracle - uppercase, so on)
    ($column_name:literal) => { const_str::convert_ascii_case!(lower, $column_name) };
}
