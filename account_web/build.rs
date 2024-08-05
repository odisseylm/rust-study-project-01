
fn main() {
    // println!("cargo:rerun-if-changed=Cargo.toml");
    // println!("cargo:rerun-if-changed=patches/");

    generate_account_soa_open_api_doc();
    generate_rest_client();
}


fn generate_account_soa_open_api_doc() {
    use std::process::Command;

    let build_env = BuildEnv::new();

    let account_open_api_doc = build_env.target_dir.join("mvv_account_soa-openapi.json");
    let account_open_api_doc = account_open_api_doc.as_path();

    if !account_open_api_doc.exists() {
        let exec_cmd = build_env.target_profile_dir.join("mvv_account_soa");
        let exec_cmd = exec_cmd.as_path();

        if !exec_cmd.exists() {
            panic!("[{exec_cmd:?}] does not exist.");
        }

        Command::new(exec_cmd)
            .current_dir(build_env.target_dir)
            .arg("--generate-open-api")
            .status().unwrap();
    }
}

fn generate_rest_client() {

    let build_env = BuildEnv::new();

    let account_open_api_doc = build_env.target_dir.join("mvv_account_soa-openapi.json");
    let account_open_api_doc = account_open_api_doc.as_path();
    println!("cargo:rerun-if-changed={}", account_open_api_doc.to_string_lossy().as_ref());

    let mut generator = progenitor::Generator::default();

    if !account_open_api_doc.exists() {
        panic!("account_soa OpenAPI [{account_open_api_doc:?}] does not exist.")
    }

    let file = std::fs::File::open(account_open_api_doc).unwrap();
    let spec = serde_json::from_reader(file).unwrap();

    let tokens = generator.generate_tokens(&spec).unwrap();
    let ast = syn::parse2(tokens).unwrap();
    let generated_content = prettyplease::unparse(&ast);

    let generated_content = generated_content.replace(
        "\nimpl Client {",
        "\n#[allow(unused_imports, unused_qualifications)]\nimpl Client {");

    // let mut out_file = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).to_path_buf();
    let generated_file_dir = build_env.project_dir.join("src/rest_dependencies");
    std::fs::create_dir_all(&generated_file_dir);
    let generated_file_path = generated_file_dir.join("account_soa_client.rs");
    let generated_file_path = generated_file_path.as_path();

    let to_write: bool = if generated_file_path.exists() {
        let prev_generated_content = std::fs::read_to_string(generated_file_path).unwrap();
        prev_generated_content != generated_content
    } else {
        true
    };

    if to_write {
        std::fs::write(generated_file_path, generated_content).unwrap();
    }
}


struct BuildEnv {
    project_dir: std::path::PathBuf,
    target_dir: std::path::PathBuf,
    target_profile_dir: std::path::PathBuf,
}

impl BuildEnv {
    fn new() -> Self {

        // CARGO_TARGET_DIR
        // CARGO_BUILD_TARGET_DIR - default
        // CARGO_BUILD_DEP_INFO_BASEDIR
        // CARGO_MANIFEST_DIR
        // OUT_DIR — If the package has a build script, this is set to the folder where the build script should place its output.
        //

        use core::str::FromStr;

        let project_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let project_dir = std::path::PathBuf::from_str(&project_dir).unwrap();
        // let project_dir = std::path::Path::new(&project_dir);

        let target_dir = std::env::var("CARGO_TARGET_DIR")
            .or(std::env::var("CARGO_CRATE_TARGET_DIR")) // not documented ??
            .or(std::env::var("CARGO_BUILD_TARGET_DIR"))
            // It is very strange, but both CARGO_TARGET_DIR/CARGO_BUILD_TARGET_DIR are not present...
            .unwrap_or_else(|_err|project_dir.as_path().join("../target").to_string_lossy().to_string());
        let target_dir = std::path::PathBuf::from_str(&target_dir).unwrap();

        // PROFILE
        let profile = std::env::var("PROFILE").unwrap();
        let profile = profile.as_str();

        let target_sub_dir = if profile == "release" || profile == "prod" {
            "release"
        } else if profile == "dev" || profile == "development" || profile == "debug" {
            "debug"
        } else {
            panic!("Unexpected value of PROFILE [{profile}]")
        };
        let target_profile_dir = target_dir.join(target_sub_dir);


        BuildEnv {
            project_dir,
            target_dir,
            target_profile_dir,
        }
    }
}
