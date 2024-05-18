use project01::util::backtrace::NewBacktracePolicy;
use project01::util::BacktraceInfo;


#[allow(dead_code)]
#[derive(Debug)]
enum Suit {
    Heart(i32),
    Diamond,
    Spade,
    Club,
}


#[test]
fn test_print_enum() {

    // println!("{}", Suit::Heart(1));

    // output: Heart(1)
    println!("{:?}", Suit::Heart(1));
    println!("{:?}", Suit::Diamond);

    // output:
    // Heart(
    //     1,
    // )
    println!("{:#?}", Suit::Heart(1)); // hm
    println!("{:#?}", Suit::Diamond); // hm
}


#[derive(Debug, strum_macros::Display)]
#[allow(dead_code)]
enum Suit2 {
    Heart(i32),
    Diamond,
    Spade,
    #[strum(to_string = "saturation is {sat}")]
    Club { sat: BacktraceInfo },
    // Club(BacktraceInfo),
}


#[test]
fn test_print_enum_with_scrum_dependency() {

    println!("{}", Suit2::Heart(1));
    println!("{}", Suit2::Diamond);
    // println!("{}", Suit2::Club(BacktraceInfo::new()));
    println!("{}", Suit2::Club { sat:BacktraceInfo::new_by_policy(NewBacktracePolicy::Capture) });
}
