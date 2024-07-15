/*
#![feature(test)]
extern crate test;

#[allow(soft_unstable)]
#[allow(incomplete_features)]

// #![feature(test)]
// extern crate test;


// #[deny(soft_unstable)]
// #[allow(soft_unstable)]
// #[allow(incomplete_features)]
#[bench]
fn bench_factorial(b: &mut test::Bencher) {
    b.iter(|| {
        let result = factorial(15);
        assert_eq!(result, 1_307_674_368_000);
    });
}
*/




/*
// https://doc.rust-lang.org/unstable-book/library-features/test.html

#![feature(test)]

extern crate test;

pub fn add_two(a: i32) -> i32 { a + 2 }

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn it_works() {
        assert_eq!(4, add_two(2));
    }

    #[bench]
    fn bench_add_two(b: &mut Bencher) {
        b.iter(|| add_two(2));
    }
}
*/


/*
// https://doc.rust-lang.org/unstable-book/library-features/test.html

#![feature(test)]

extern crate test;
use test::Bencher;

#[bench]
fn bench_xor_1000_ints(b: &mut Bencher) {
    b.iter(|| {
        (0..1000).fold(0, |old, new| old ^ new);
    });
}
*/
