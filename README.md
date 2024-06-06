



# Build

 - `cargo build`
 - See step by step https://medium.com/@hmquan08011996/dockerfile-for-rust-6d13dadca84d
   - `docker build --file docker/Dockerfile_01 .`
   - `docker build --file docker/Dockerfile_02 --tag rust-mvv-webapp-02 --no-cache .`
   - `docker build --file docker/Dockerfile_02 --tag rust-mvv-webapp-02 .`

 - Docker https://docs.docker.com/language/rust/
   - `docker build -t rust_docker .`
   - `docker run -it --rm --name my-running-app rust_docker`
   - Links
     - ??? https://www.docker.com/blog/simplify-your-deployments-using-the-rust-official-image/
     - https://habr.com/ru/companies/T1Holding/articles/766620/
     - https://hub.docker.com/_/rust

 - `cargo expand entities::amount_parse_old --lib`
 - `cargo expand entities::account --lib`
 - `cargo expand another_static_error_macro_test::parse_amount_another_01 --lib`

 - `cargo tree`
   - `cargo tree --edges features`
   - `cargo tree -e features`
   - `cargo tree -f "{p} {f}"`
   - ? `cargo tree -e features -i serde_json`


# Build notes

 - Libraries should ignore Cargo.lock but binaries/applications should check-in Cargo.lock.
   - https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html


# Tests

 - Run tests: `cargo test`
 - Run tests with output: `cargo test -- --nocapture`
 - Run tests with output: `cargo test test_log_env --test log_env_test -- --nocapture --exact`
 - Run tests with output: `cargo test test_log_env --test log_env_test -- --nocapture`  # ??? --exact


Docs test
 - https://medium.com/@AlexanderObregon/testing-in-rust-unit-tests-integration-tests-and-documentation-tests-ae7c10bbb4a6

Should public functions be tested in internal or integration tests ??
 - https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/second-edition/ch11-03-test-organization.html



