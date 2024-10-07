use log::warn;


pub struct BuildEnv {
    pub project_dir: std::path::PathBuf,
    pub target_dir: std::path::PathBuf,
    pub target_profile_dir: std::path::PathBuf,
}

impl BuildEnv {
    pub fn try_new() -> anyhow::Result<Self> {

        // CARGO_TARGET_DIR
        // CARGO_BUILD_TARGET_DIR - default
        // CARGO_BUILD_DEP_INFO_BASEDIR
        // CARGO_MANIFEST_DIR
        // OUT_DIR — If the package has a build script, this is set to the folder where the build script should place its output.
        //

        use core::str::FromStr;
        use crate::env::required_env_var_static as env_var;

        // My custom vars
        // ROOT_PROJECT_DIR
        // PROJECT_DIR
        //
        // RUSTFLAGS: -C instrument-coverage

        // Interesting
        // CARGO_MAKE_BINARY_EXECUTABLE_NAME: mvv_account_soa
        // CARGO_MAKE_CARGO_ALL_FEATURES: --all-features
        // CARGO_MAKE_CARGO_BUILD_TEST_FLAGS: --all-features
        // CARGO_MAKE_CARGO_PUBLISH_ALLOW_DIRTY: --allow-dirty
        // CARGO_MAKE_MAKEFILE_PATH: /home/volodymyr/projects/rust/rust-study-project-01/account_soa/Makefile.toml
        // CARGO_MAKE_RUST_VERSION: 1.79.0
        // CARGO_MAKE_TASK: it-tests-covered

        // CARGO_MAKE_USE_WORKSPACE_PROFILE: true
        // CARGO_MAKE_PROFILE: development
        // CARGO_MAKE_CARGO_PROFILE: dev

        // CARGO_PKG_NAME: mvv_account_soa
        // CARGO_MAKE_PROJECT_NAME: mvv_account_soa
        // CARGO_MAKE_WORKING_DIRECTORY: /home/volodymyr/projects/rust/rust-study-project-01/account_soa
        //

        // ???
        // CARGO_MAKE_TEST_COVERAGE_BINARY_FILTER: mvv_account_soa-[a-z0-9]*$\|test_[_a-z0-9]*-[a-z0-9]*$\|[a-z0-9]*_test-[_a-z0-9]*
        // CARGO_MAKE_TEST_COVERAGE_DEFAULT_BINARY_FILTER: mvv_account_soa-[a-z0-9]*$\|test_[_a-z0-9]*-[a-z0-9]*$\|[a-z0-9]*_test-[_a-z0-9]*


        let project_dir = env_var("CARGO_MANIFEST_DIR") ?;
        let project_dir = std::path::PathBuf::from_str(&project_dir) ?;

        // TODO: write func required_one_of_env_var
        let target_dir = env_var("CARGO_TARGET_DIR")
            .or_else(|_| env_var("CARGO_CRATE_TARGET_DIR")) // not documented ??
            .or_else(|_| env_var("CARGO_BUILD_TARGET_DIR"))
            // It is very strange, but both CARGO_TARGET_DIR/CARGO_BUILD_TARGET_DIR are not present...
            .unwrap_or_else(|_err|project_dir.as_path().join("../target").to_string_lossy().to_string());
        let target_dir = std::path::PathBuf::from_str(&target_dir) ?;

        // for (ref var_name, ref var_value) in std::env::vars() {
        //     println!("### {var_name}: {var_value}");
        // }

        let profile = env_var("PROFILE")
            .or_else(|_err| env_var("CARGO_MAKE_PROFILE"))
            .or_else(|_err| env_var("CARGO_MAKE_CARGO_PROFILE"))
            .unwrap_or_else(|_err|{
                warn!("Profile env var is not found. Probably it is launched from IDE. Let's consider profile as debug/dev.");
                "dev".to_owned()
            });
            // .map_err(|_err| anyhow!("profile is not found")) ?;
        let profile = profile.as_str();

        let target_sub_dir = if profile == "release" || profile == "prod" {
            "release"
        } else if profile == "dev" || profile == "development" || profile == "debug" {
            "debug"
        } else {
            panic!("Unexpected value of PROFILE [{profile}]")
        };
        let target_profile_dir = target_dir.join(target_sub_dir);

        Ok(BuildEnv {
            project_dir,
            target_dir,
            target_profile_dir,
        })
    }
}
