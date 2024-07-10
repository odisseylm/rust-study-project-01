use core::fmt::{ self, Display };


#[inline]
pub fn iterable_to_display <
    'a,
    T: Display + 'a,
    SourceType,
> (
    iterable_ref: &'a SourceType,
    name: &'static str,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result
    // where for<'b> &'b SourceType: IntoIterator<Item=&'b T> {
    where &'a SourceType: IntoIterator<Item = &'a T> {

    let ref_iter = <&SourceType as IntoIterator>::into_iter(iterable_ref);
    iter_to_display(ref_iter, name, f)
}


pub fn iter_to_display <
    'a,
    T: Display,
    Iter: Iterator<Item=T>,
> (
    iter: Iter,
    name: &'static str,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {

    // // The shortest approach, but not efficient by memory usage.
    // //
    // use itertools::Itertools;
    // let perms_as_str = iter.join(", ");
    // write!(f, "{name} {{ {perms_as_str} }}")

    // No additional heap allocation.
    //
    write!(f, "{name} {{ ") ?;
    let mut first = true;
    for ref perm in iter {
        if !first { write!(f, ", ") ?; }
        write!(f, "{}", perm) ?;
        first = false;
    }
    write!(f, " }}")
}


#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt;
    use std::collections::HashSet;
    use assertables::{ assert_contains, assert_contains_as_result };
    use crate::util::test_unwrap::TestSringOps;

    struct TestIterVec(Vec<i32>);
    impl Display for TestIterVec {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            iter_to_display(self.0.iter(), "TestIterVec", f)
        }
    }
    struct TestIterSet(HashSet<i32>);
    impl Display for TestIterSet {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            iter_to_display(self.0.iter(), "TestIterSet", f)
        }
    }

    struct TestIntoIterVec(Vec<i32>);
    impl Display for TestIntoIterVec {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            iterable_to_display(&self.0, "TestIntoIterVec", f)
        }
    }
    struct TestIntoIterSet(HashSet<i32>);
    impl Display for TestIntoIterSet {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            iterable_to_display(&self.0, "TestIntoIterSet", f)
        }
    }

    #[test]
    fn test_iter_to_display() {

        let v = TestIterVec(vec!(1, 2, 3));
        assert_eq!(v.to_test_string(), "TestIterVec { 1, 2, 3 }");

        let s = TestIterSet(HashSet::from([3, 4, 5]));
        // It is unstable test since order in hash set is unpredictable
        // assert_eq!(s.to_test_string(), "TestIterSet { 5, 4, 3 }")
        let display_str = s.to_test_string();
        assert_contains!(display_str, "TestIterSet { ");
        assert_contains!(display_str, "3");
        assert_contains!(display_str, "4");
        assert_contains!(display_str, "5");
    }

    #[test]
    fn test_into_iter_to_display() {

        let v = TestIntoIterVec(vec!(1, 2, 3));
        assert_eq!(v.to_test_string(), "TestIntoIterVec { 1, 2, 3 }");

        let s = TestIntoIterSet(HashSet::from([3, 4, 5]));
        // It is unstable test since order in hash set is unpredictable
        // assert_eq!(s.to_test_string(), "TestIntoIterSet { 5, 4, 3 }")
        let display_str = s.to_test_string();
        assert_contains!(display_str, "TestIntoIterSet { ");
        assert_contains!(display_str, "3");
        assert_contains!(display_str, "4");
        assert_contains!(display_str, "5");
    }
}
