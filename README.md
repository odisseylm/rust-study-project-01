



Investigate

 - crate typewit
 - crate derivative
   - try to replace manual written Debug/Display by macros
# Build

 - `cargo build`
 - See step by step https://medium.com/@hmquan08011996/dockerfile-for-rust-6d13dadca84d
   - `docker build --file docker/Dockerfile_01 .`
   - `docker build --file docker/Dockerfile_02 --tag rust-mvv-webapp-02 --no-cache .`
   - `docker build --file docker/Dockerfile_02 --tag rust-mvv-webapp-02 .`
   - `docker build --file docker/Dockerfile_04 --tag rust-mvv-webapp-04 .`
   - `docker run rust-mvv-webapp-04`
   - `docker rmi $(docker images -f "dangling=true" -q)`

 - Docker https://docs.docker.com/language/rust/
   - `docker build -t rust_docker .`
   - `docker run -it --rm --name my-running-app rust_docker`
   - Links
     - ??? https://www.docker.com/blog/simplify-your-deployments-using-the-rust-official-image/
     - https://habr.com/ru/companies/T1Holding/articles/766620/
     - https://hub.docker.com/_/rust

 - `cargo expand entities::amount_parse_old --lib`
 - `cargo expand entities::amount::parse --lib`
 - `cargo expand entities::account --lib`
 - `cargo expand another_static_error_macro_test::parse_amount_another_01 --lib`
 - `cargo expand auth::login_form_auth --lib`
 - `cargo expand auth::internal_delegatable_traits::aaa --lib`
 - `cargo expand auth::examples::usage --lib`
 - `cargo expand entities::id --lib`
 - `cargo expand --test tuple_test2`
 - `cargo expand --lib  --ugly > temp_all_app.rs`

 - `cargo tree`
   - `cargo install cargo-expand`
   - `cargo tree --edges features`
   - `cargo tree -e features`
   - `cargo tree -f "{p} {f}"`
   - ? `cargo tree -e features -i serde_json`

# Run

 - ``
 - http://localhost:3000/api/account/all
 - http://localhost:3000/api/account/345

## Cargo make

 - Install
   - `cargo install cargo-make`
   - `cargo install --force cargo-make`
   - `cargo install --no-default-features --force cargo-make`
 - Run tasks
   - `cargo-make make -- my-flow`
   - If submodules do not have own Makefiles
     - `cargo-make make --no-workspace -- my-flow`

 - Predefined vars
   - https://github.com/sagiegurari/cargo-make#global
 - Predefined tasks
   - https://github.com/sagiegurari/cargo-make/blob/master/docs/cargo_make_task_list.md

# Build notes

 - To patch 'progenitor'
   - `cargo install cargo-patch-subdep-ver`
     - or `cargo install --git https://github.com/odisseylm/cargo_patch_subdep`
     - or `cargo install --path cargo-patch-subdep-ver`
   - Apply cargo command (in project dir)
     - `cargo patch-subdep-ver`

 ??? - Libraries should ignore Cargo.lock but binaries/applications should check-in Cargo.lock.
   - https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html


# Tests

 - Run tests: `cargo test`
 - Run tests with output: `cargo test -- --nocapture`
 - Run tests with output: `cargo test test_log_env --test log_env_test -- --nocapture --exact`
 - Run tests with output: `cargo test test_log_env --test log_env_test -- --nocapture`  # ??? --exact
 - `docker exec -it DOCKER_CONTAINER /bin/bash`
 - `psql postgresql://rust_mvvbank:psw@localhost:5432/rust_mvvbank`
 - `psql --dbname=rust-mvvbank  --username=rust-mvvbank`
 - `psql rust-mvvbank rust-mvvbank`
   - `\q` - quit
   - `\l` - list all databases
   - `\dt` - list tables
   - `\dv` - list views
   - `\d <table-name>` - describe table
   - `\dn` - list schemas
   - `\df` - list functions
   - `\i <file-name>` - run commands from file
   - See also
     - https://hasura.io/blog/top-psql-commands-and-flags-you-need-to-know-postgresql
     - https://www.oreilly.com/library/view/practical-postgresql/9781449309770/ch04s01.html
     - https://www.postgresql.org/docs/current/app-psql.html
 - `curl -H "Authorization: Basic dm92YW4tcmVhZDpxd2VydHk=" http://localhost:3000/api/current_user/account/456`
 - `curl -H "Authorization: Basic dm92YW4tcmVhZDpxd2VydHk=" http://localhost:8101/api/current_user/account/456`
 - `curl --verbose -X POST -H "Content-Type: application/json" -d '{"email_parent_filed":{"email33":"a@b@c"}}' http://localhost:3000/api/current_user/validate_test/input_validate_1`


Docs test
 - https://medium.com/@AlexanderObregon/testing-in-rust-unit-tests-integration-tests-and-documentation-tests-ae7c10bbb4a6

Should public functions be tested in internal or integration tests ??
 - https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/second-edition/ch11-03-test-organization.html


TODO: investigate
 - string-builder ?? How overloading is done there??
 - try to use SmallVec instead of Vec
 - try to use fixedstr string types