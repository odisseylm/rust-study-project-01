
fn main() {
    generate_patch()
}


fn generate_patch() {
    use std::process::Command;

    let build_env = BuildEnv::new();
    let patch_dir = build_env.target_dir.join("patch-override-sub-dep");

    if !patch_dir.exists() {
        Command::new("cargo")
            .current_dir(build_env.project_dir)
            .arg("patch-subdep-ver")
            .status().unwrap();
    } else {
        println!("### xTask generate_patch => patch dir already exists");
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
