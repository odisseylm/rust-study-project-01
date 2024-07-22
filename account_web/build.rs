

fn main() {
    let src = "/home/volodymyr/projects/rust/rust-study-project-01/target/mvv_account_soa-openapi.json";
    println!("cargo:rerun-if-changed={}", src);

    let mut generator = progenitor::Generator::default();

    let file = std::fs::File::open(src).unwrap();
    let spec = serde_json::from_reader(file).unwrap();

    let tokens = generator.generate_tokens(&spec).unwrap();
    let ast = syn::parse2(tokens).unwrap();
    let content = prettyplease::unparse(&ast);

    let mut out_file = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).to_path_buf();
    out_file.push("account_soa_client_gen.rs");

    std::fs::write(out_file, content).unwrap();
}
