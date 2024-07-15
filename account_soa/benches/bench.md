




https://stackoverflow.com/questions/76242130/how-to-fix-use-of-unstable-library-feature-test-bench-is-a-part-of-custom-t


1. Compile using nightly version or set nightly to default:
 To compile using nightly version:
  `cargo +nightly bench ...`
 To set nightly as default:
  `rustup default nightly`
 
2. Add test feature
 To do this, add 2 lines to the top of your root file. (Even above imports)
 ```
 #![feature(test)]
 extern crate test;
 
 use...
 ```

 This will allow you to use the `#[bench]` feature.
