use core::{
    fmt::Debug,
    future::Future,
    time::Duration,
};
use std::{
    ffi::OsString,
    path::PathBuf,
    error,
};
use anyhow::anyhow;
use itertools::Itertools;
use log::{error, info};
use rustainers::compose::{ComposeContainers, ComposeRunOption, ToRunnableComposeContainers};
use mvv_common::test::{current_sub_project_dir};
use crate::{docker_compose::{docker_compose_down}, prepare_docker_compose, PrepareDockerComposeCfg};
use crate::coverage::is_code_coverage_enabled_for;
use crate::docker_compose::{get_docker_compose_file, preload_docker_compose_images};
use crate::docker_compose_util::get_docker_compose_services;
use crate::files::CopyCfg;
//--------------------------------------------------------------------------------------------------



pub async fn wait_rustainers<C>(
    res_docker_compose_dir: PathBuf,
    docker_compose_display_name: &str,
    fut: impl Future<Output=Result<ComposeContainers<C>, rustainers::runner::RunnerError>>,
    pause_duration: Duration,
) -> anyhow::Result<(PathBuf, ComposeContainers<C>)>
where
    C: ToRunnableComposeContainers + Debug,
    ComposeContainers<C>: Debug,
{
    let compose_containers_res = tokio::time::timeout(pause_duration, fut).await;

    match compose_containers_res {
        Ok(res) => {
            match res {
                Ok(res) => {
                    info!("SUCCESS of [{docker_compose_display_name}] launch => {res:?}");
                    Ok( (res_docker_compose_dir, res) )
                },
                Err(err) => {
                    error!("FAILURE of [{docker_compose_display_name}] launch => Error {{ {err:?} }}");
                    let _ = docker_compose_down(&res_docker_compose_dir);
                    Err(anyhow!(err))
                },
            }
        },
        Err(err) => {
            error!("FAILURE of [{docker_compose_display_name}] launch => Error {{ {err:?} }}");
            info!("Shut down docker compose manually...");
            let _ = docker_compose_down(&res_docker_compose_dir);
            Err(anyhow!(err))
        },
    }
}



pub fn is_integration_tests_enabled() -> bool {

    let is_it_1 = std::env::var("INTEGRATION_TEST").is_ok();
    let is_it_2 = std::env::var("INTEGRATION_TESTS").is_ok();
    let is_exact = std::env::args_os().contains(&OsString::from("--exact"));

    let test_enabled = is_it_1 || is_it_2 || is_exact;

    test_enabled
}



#[derive(Debug)]
#[non_exhaustive]
#[allow(dead_code)]
pub struct ComposeContainersState<Cfg: ToRunnableComposeContainers>
    where
        Cfg: Debug,
        <Cfg as ToRunnableComposeContainers>::AsPath: Debug,
{
    pub docker_compose_dir: PathBuf,
    pub compose_containers: ComposeContainers<Cfg>,
    // It some name similar to subproject or docker-compose service
    // It is used only as comment/label/description.
    pub container_name_label: String,
    pub is_code_coverage_enabled: bool,
    #[doc(hidden)]
    __non_exhaustive: (),
}

#[derive(Debug, Clone)]
// #[non_exhaustive] // Not used now, since it does not allow to use '..Default::default()` with it.
pub struct LaunchSoaDockerComposeParams {
    /// Use it to explicitly specify code-coverage is enabled.
    /// If code-coverage is enabled, *.profraw files from docker container will be copied
    /// to local machine (just to make sure that profraw files are flushed,
    /// containers will/should be restarted).
    pub code_coverage_enabled: Option<bool>,
    /// Optional, if present it is used to verify whether code-coverage is enabled.
    pub exe_file: Option<PathBuf>,
    /// By default, "/appuser/code-coverage"
    /// Please do not change/use it now, since it is hardcoded in
    /// fn copy_code_coverage_files_for_all_containers() (or implement it there).
    pub container_code_coverage_dir: String,
    // Internal field. I cannot do it private because at that moment rust
    // does not allow to use Default with private fields... Strange language design...
    // Please do not init this field manually, use `..Default::default()` instead.
    #[doc(hidden)]
    pub __non_exhaustive: (),
}
impl Default for LaunchSoaDockerComposeParams {
    fn default() -> Self {
        Self {
            code_coverage_enabled: None,
            exe_file: None,
            container_code_coverage_dir: "/appuser/code-coverage".to_owned(),
            __non_exhaustive: (),
        }
    }
}

/// Its is enough complicated code. If your SOA requires customization, just copy this function
/// to your SOA.
///
pub async fn launch_soa_docker_compose<Cfg: ToRunnableComposeContainers>(
    params: &LaunchSoaDockerComposeParams,
) -> anyhow::Result<ComposeContainersState<Cfg>>
where
    Cfg: Debug + TryFrom<PathBuf>,
    <Cfg as ToRunnableComposeContainers>::AsPath: Debug,
    <Cfg as TryFrom<PathBuf>>::Error: error::Error + Send + Sync + 'static,
{
    // let build_env = BuildEnv::try_new() ?;

    let LaunchSoaDockerComposeParams {
        ref code_coverage_enabled,
        ref exe_file,
        .. } = params;

    let is_code_coverage_enabled: bool =
        if let Some(code_coverage_enabled) = code_coverage_enabled {
            *code_coverage_enabled
        } else if let Some(exe_file) = exe_file {
            is_code_coverage_enabled_for(exe_file) ?
        } else {
            false
        };

    let tests_session_id = chrono::Local::now().timestamp();
    let sub_project_dir = current_sub_project_dir() ?;

    let cfg = PrepareDockerComposeCfg {
        tests_session_id,
        copy: CopyCfg {
            base_from_dir: "".into(), // sub_project_dir.clone(),
            copy: vec!(
                /*
                Copy { from: p("docker/docker-compose.env"), to: p("docker-compose.env") },
                Copy { from: p("docker/docker-compose.yml"), to: p("docker-compose.yml") },
                Copy { from: p("test_resources/postgres"), to: p("test_resources/postgres") },
                Copy { from: p("../target/generated-test-resources/ssl"), to: p("generated-test-resources/ssl") },
                */
            ),
        },
        replace: vec!(
            /*
            // It is mainly example. Just exactly this substitution is done automatically.
            Replace::by_str(
                p("docker-compose.yml"),
                ["${DOCKER_IMAGE_PROFILE_SUFFIX}"], [docker_image_profile_suffix],
            ),
            */
        ),
        ..Default::default()
    };

    let temp_docker_compose_dir = prepare_docker_compose(&sub_project_dir, &cfg) ?;

    // Preload external images to avoid unexpected pause (and tests failure due to timeout)
    let docker_compose_file = get_docker_compose_file(&temp_docker_compose_dir) ?;
    preload_docker_compose_images(&docker_compose_file) ?;

    let cur_sub_prj_dir = current_sub_project_dir() ?;

    // Instead of hardcoding for every module
    // Or from env vars:
    //  CARGO_PKG_NAME: mvv_account_soa
    //  CARGO_MAKE_PROJECT_NAME: mvv_account_soa
    //
    let cur_sub_prj_name = cur_sub_prj_dir.file_name()
        .expect(&format!("[{cur_sub_prj_dir:?}] should be project dir"))
        .to_string_lossy().to_string();

    let mut envs = indexmap::IndexMap::<String, String>::new();

    if is_code_coverage_enabled {
        let LaunchSoaDockerComposeParams { ref container_code_coverage_dir, .. } = params;
        let now_timestamp = chrono::Local::now().timestamp();

        let services = get_docker_compose_services(&docker_compose_file) ?;
        for service in services {
            let serv_env_prefix = service.to_uppercase().replace('-', "_");
            let llvm_profile_env_name = format!("{serv_env_prefix}_LLVM_PROFILE_FILE");
            envs.insert(
                llvm_profile_env_name,
                format!("{container_code_coverage_dir}/{service}-{now_timestamp}-%p-%m.profraw"),
            );

        }
    }

    let option: ComposeRunOption = ComposeRunOption::builder()
        // Wait interval for service health check
        .with_wait_interval(Duration::from_secs(1))
        // Wait interval for service to exist
        .with_wait_services_interval(Duration::from_secs(2))
        .with_env(envs)
        .build();

    let runner = rustainers::runner::Runner::docker() ?;

    info!("### Attempt to run docker compose for [${cur_sub_prj_name}]", );

    // to make sure - clean up previous session
    info!("### Clean previous docker compose session");
    let _ = docker_compose_down(&temp_docker_compose_dir);

    let cfg = Cfg::try_from(temp_docker_compose_dir.to_path_buf()) ?;
    let compose_containers_fut = runner.compose_start_with_options(cfg, option);

    let (docker_compose_dir, compose_containers) = wait_rustainers(
        temp_docker_compose_dir, &cur_sub_prj_name, compose_containers_fut, Duration::from_secs(15))
        .await ?;

    Ok(ComposeContainersState {
        docker_compose_dir,
        compose_containers,
        is_code_coverage_enabled,
        container_name_label: cur_sub_prj_name,
        __non_exhaustive: (),
    })
}
