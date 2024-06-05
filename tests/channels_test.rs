use std::sync::mpsc::RecvError;
use project01::util::TestResultUnwrap;

#[derive(Debug)]
#[allow(dead_code)]
struct S {
    name: String,
}

#[test]
fn aaa() {
    let (s, r) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        s.send(S { name: "John".to_string() }).test_unwrap();
        // s.send("John".to_string()).test_unwrap();
    });

    let r: Result<S, RecvError> = r.recv();
    println!("result: {r:?}");
}

