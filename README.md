



# Build

 - `cargo expand entities::amount_parse_old --lib`
 - `cargo expand another_static_error_macro_test::parse_amount_another_01 --lib`

# Build notes

 - Libraries should ignore Cargo.lock but binaries/applications should check-in Cargo.lock.
   - https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html


# Tests

 - Run tests: `cargo test`
 - Run tests with output: `cargo test -- --nocapture`


Docs test
 - https://medium.com/@AlexanderObregon/testing-in-rust-unit-tests-integration-tests-and-documentation-tests-ae7c10bbb4a6

Should public functions be tested in internal or integration tests ??
 - https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/second-edition/ch11-03-test-organization.html



