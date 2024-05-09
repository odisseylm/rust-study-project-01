

https://zerotomastery.io/blog/complete-guide-to-testing-code-in-rust/


## Integration Tests

 The other common form of test included with a Rust project is integration tests, held under tests/.
 Each file in that directory is run as a separate test program that executes all of the functions marked with #[test].

 Integration tests do not have access to crate internals and so act as behavior tests that can exercise only the public API of the crate.

