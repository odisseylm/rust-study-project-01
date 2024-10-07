use core::fmt::Debug;
use std::{
    path::{Path, PathBuf},
    time::Duration,
    process::Command,
};
use anyhow::anyhow;
use assert_json_diff::{assert_json_eq};
use assertables::{assert_ge, assert_ge_as_result};
use log::{debug, info};
use reqwest::{Certificate, Response};
use rustainers::{
    ExposedPort, Port, WaitStrategy,
    compose::{
        ComposeContainers, ComposeError, ComposeRunOption,
        RunnableComposeContainers, RunnableComposeContainersBuilder,
        ToRunnableComposeContainers,
    }
};
use serde_json::json;
use mvv_common::test::{
    {current_project_target_dir, current_sub_project_dir, TestOptionUnwrap},
    BuildEnv,
    docker_compose::{get_docker_compose_file, preload_docker_compose_images},
    docker_compose::{
        docker_compose_down, docker_compose_ps, docker_compose_start_except, docker_compose_stop_except,
    },
    docker_compose::{wait_for_healthy_except,},
    files::CopyCfg,
    integration::{AutoDockerComposeDown, PrepareDockerComposeCfg},
    integration::{is_integration_tests_enabled, prepare_docker_compose, wait_containers},
};
use mvv_common::fn_name;
//--------------------------------------------------------------------------------------------------



const ACCOUNT_SOA_SERVICE: &'static str = "rust-account-soa";
const ACCOUNT_SOA_HTTP_PORT: Port = Port::new(8443);

const POSTGRES_SERVICE: &'static str = "database";
const POSTGRES_PORT: Port = Port::new(5432);
// Other ports


#[derive(Debug)]
struct AccountSoaTestContainers {
    dir: PathBuf,
    account_soa_http_port: ExposedPort,
    // debug_port: ExposedPort,
    postgres_port: ExposedPort,
}


impl AccountSoaTestContainers {
    pub async fn new(dir: &Path) -> Result<Self, ComposeError> {
        Ok(Self {
            dir: dir.to_path_buf(),
            account_soa_http_port: ExposedPort::new(ACCOUNT_SOA_HTTP_PORT.clone()),
            postgres_port: ExposedPort::new(POSTGRES_PORT.clone()),
        })
    }
}

impl ToRunnableComposeContainers for AccountSoaTestContainers {
    type AsPath = PathBuf; // TemporaryDirectory, TemporaryFile

    fn to_runnable(&self, builder: RunnableComposeContainersBuilder<Self::AsPath>) -> RunnableComposeContainers<Self::AsPath> {
        builder
            // Only directory can be passed :-(
            .with_compose_path(self.dir.clone())
            .with_port_mappings([
                (ACCOUNT_SOA_SERVICE, self.account_soa_http_port.clone()),
                (POSTGRES_SERVICE, self.postgres_port.clone()),
            ])
            .with_wait_strategies([
                (ACCOUNT_SOA_SERVICE, WaitStrategy::HealthCheck),
                /*
                (ACCOUNT_SOA_SERVICE, WaitStrategy::HttpSuccess {
                    https: true, // rustainers does not support ignoring certificate
                    path: "/health-check".to_owned(),
                    container_port: ACCOUNT_SOA_HTTP_PORT.clone(),
                }),
                */
                (POSTGRES_SERVICE, WaitStrategy::stdout_match(
                    regex::Regex::new("PostgreSQL init process complete; ready for start up")
                        .expect("Incorrect RegEx for PostgreSQL."))),
            ])
            .build()
    }
}

#[derive(Debug)]
pub struct ComposeContainersState<Cfg: ToRunnableComposeContainers>
    where
        Cfg: Debug,
        <Cfg as ToRunnableComposeContainers>::AsPath: Debug,
{
    docker_compose_dir: PathBuf,
    compose_containers: ComposeContainers<Cfg>, //AccountSoaTestContainers>,
    // It some name similar to subproject or docker-compose service
    // It is used only as comment/label/description.
    container_name_label: String,
    is_code_coverage_enabled: bool,
}


async fn launch_account_soa_docker_compose() -> anyhow::Result<ComposeContainersState<AccountSoaTestContainers>> {

    let build_env = BuildEnv::try_new() ?;

    let exe_file = build_env.target_profile_dir.join("mvv_account_soa");
    let is_code_coverage_enabled = is_code_coverage_enabled_for(&exe_file) ?;

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
    preload_docker_compose_images(&get_docker_compose_file(&temp_docker_compose_dir) ?) ?;

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
        let now_timestamp = chrono::Local::now().timestamp();
        envs.insert(
            "LLVM_PROFILE_FILE".to_owned(),
            format!("/appuser/code-coverage/{cur_sub_prj_name}-{now_timestamp}-%p-%m.profraw"),
        );
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

    let compose_containers_fut = runner.compose_start_with_options(
        AccountSoaTestContainers::new(&temp_docker_compose_dir).await ?,
        option,
    );

    let (docker_compose_dir, compose_containers) = wait_containers(
        temp_docker_compose_dir, &cur_sub_prj_name, compose_containers_fut, Duration::from_secs(15))
        .await ?;

    Ok(ComposeContainersState {
        docker_compose_dir,
        compose_containers,
        is_code_coverage_enabled,
        container_name_label: cur_sub_prj_name,
    })
}


fn is_code_coverage_enabled_for(path: &Path) -> anyhow::Result<bool> {

    use elf::ElfBytes;
    use elf::endian::AnyEndian;

    // I didn't find (working) ELF lib, which does not load the whole file to memory :-(
    let file_data = std::fs::read(path) ?;
    let file_data = file_data.as_slice();
    let elf_bytes = ElfBytes::<AnyEndian>::minimal_parse(file_data) ?;

    let some_llvm_sections = ["__llvm_prf_names", "__llvm_prf_cnts", "__llvm_prf_data",];
    let has_llvm_section =some_llvm_sections.iter()
        .any(|some_llvm_section|{
            let section = elf_bytes
                .section_header_by_name(some_llvm_section);
            if let Ok(Some(ref _section)) = section {
                true
            } else {
                false
            }
        });

    Ok(has_llvm_section)
}


#[tokio::test]
async fn integration_test_run_account_soa_docker_compose() -> anyhow::Result<()> {

    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    // let vars =  std::env::vars().map(|(k,v)|format!("{k:?} = {v:?}")).sorted().join("\n");
    // println!("vars: {vars}");

    if !is_integration_tests_enabled() {
        info!("Integration test [{}] is SKIPPED/IGNORED", fn_name!());
        return Ok(());
    }

    let ComposeContainersState {
        docker_compose_dir,
        compose_containers,
        is_code_coverage_enabled,
        container_name_label,
        ..
    } = launch_account_soa_docker_compose().await ?;
    let docker_compose_dir = docker_compose_dir.as_path();

    let auto_docker_compose_down = AutoDockerComposeDown {
        docker_compose_file_dir: docker_compose_dir.to_path_buf(),
        log_message: Some("### Cleaning..."),
    };

    // THERE should be real REST tests.
    tokio::time::sleep(Duration::from_secs(2)).await; // just in case

    let port = compose_containers.account_soa_http_port.host_port().await;

    let port: u16 = port ?.into();

    // TODO: use unwind
    test_get_all_client_accounts(port).await ?;

    if is_code_coverage_enabled {
        copy_code_coverage_files_for_all_containers(docker_compose_dir, &container_name_label) ?;
    }

    // let pause_timeout = Duration::from_secs(5);
    // info!("### Pause for {}s...", pause_timeout.as_secs());
    // tokio::time::sleep(pause_timeout).await;

    let _ = compose_containers;
    info!("### Stopping containers...");

    // to make sure to 'remove containers, networks'
    info!("### Cleaning...");
    let _ = auto_docker_compose_down;

    info!("### Test is completed");

    Ok(())
}

fn copy_code_coverage_files_for_all_containers(docker_compose_dir: &Path, container_name_label: &str)
    -> anyhow::Result<()> {

    let build_env = BuildEnv::try_new() ?;
    let coverage_raw_dir = build_env.target_dir.join("coverage-raw");

    let containers = docker_compose_ps(docker_compose_dir) ?;

    let to_ignore_services = ["database", "postgres"];
    restart_containers(docker_compose_dir, &to_ignore_services) ?;

    for ref container in containers {

        if to_ignore_services.contains(&container.service.as_str()) {
            continue;
        }

        let copy_coverage_res = copy_code_coverage_files(
            &container.id, container_name_label, docker_compose_dir, &coverage_raw_dir);
        if copy_coverage_res.is_err() {
            info!("Error of copy code coverage data.");
        }
    }

    Ok(())
}

fn restart_containers(docker_compose_dir: &Path, ignore_services: &[&str])
    -> anyhow::Result<()> {

    // Restart to surely flush code-coverage.
    info!("### Stopping docker compose services (except {ignore_services:?})");
    docker_compose_stop_except(docker_compose_dir, ignore_services) ?;

    info!("### Starting again docker compose services");
    docker_compose_start_except(docker_compose_dir, ignore_services) ?;

    wait_for_healthy_except(docker_compose_dir, ignore_services, Duration::from_secs(15)) ?;
    Ok(())
}

fn copy_code_coverage_files(container_id: &str, container_name_label: &str,
                            docker_compose_dir: &Path, coverage_dir: &Path) -> anyhow::Result<()> {

    let container_code_coverage_raw_dir = format!("{container_id}:/appuser/code-coverage/");
    let project_code_coverage_raw_dir = coverage_dir.join(container_name_label);
    let project_code_coverage_raw_dir_str = project_code_coverage_raw_dir.to_string_lossy();

    let cmd = Command::new("docker")
        .current_dir(docker_compose_dir.to_path_buf())
        .args(["cp", &container_code_coverage_raw_dir, &project_code_coverage_raw_dir_str].into_iter())
        .status() ?;

    if cmd.success() {
        // This sub-dir is created by 'docker cp' if base target dir already exists.
        let possible_coverage_sub_dir = project_code_coverage_raw_dir.join("code-coverage");
        if possible_coverage_sub_dir.exists() {
            use fs_extra::dir;
            use fs_extra::dir::DirOptions;

            let files = dir::get_dir_content2(
                &possible_coverage_sub_dir,
                &DirOptions {
                    depth: 1,
                    .. Default::default()
                }) ?.files;

            let files = files.iter().map(|f|PathBuf::from(f)).collect::<Vec<_>>();

            fs_extra::move_items(
                &files,
                &project_code_coverage_raw_dir,
                &dir::CopyOptions {
                    copy_inside: true,
                    ..Default::default()
                }) ?;
        }
        Ok(())
    } else {
        Err(anyhow!("Copying code coverage files failed for container."))
    }
}

async fn test_get_all_client_accounts(account_soa_port: u16) -> anyhow::Result<()> {

    let base_url = format!("https://localhost:{account_soa_port}");
    let url = format!("{base_url}/api/client/00000000-0000-0000-0000-000000000001/account/all");

    let build_target_dir = current_project_target_dir() ?;
    let cert_path = build_target_dir.join("generated-test-resources/ssl/ca.crt.pem");

    let pem: String = std::fs::read_to_string(&cert_path)
        .map_err(|err| anyhow!("Error of reading from [{cert_path:?}] ({err:?})")) ?;

    let client = reqwest::Client::builder()
        // .danger_accept_invalid_certs(true)
        .add_root_certificate(Certificate::from_pem(pem.as_bytes()) ?)
        .build() ?;

    let resp: Response = client.get(url)
        .basic_auth("vovan-read", Some("qwerty"))
        .send()
        .await ?;

    assert_eq!(resp.status().as_u16(), 200);

    use core::str::FromStr;
    let body_as_str: String = resp.text().await ?;

    let actual = serde_json::Value::from_str(&body_as_str) ?;
    debug!("### response: {actual}");

    let accounts: serde_json::Value = actual;
    let first_account: &serde_json::Value = accounts.get(0).test_unwrap();

    assert_ge!(accounts.as_array().test_unwrap().len(), 3);

    assert_json_eq!(
        first_account,
        json!(
            {
                "amount": {
                  "currency": "USD",
                  "value": 150
                },
                "clientId": "00000000-0000-0000-0000-000000000001",
                "createdAt": "2021-11-10T15:14:13Z",
                "iban":  "UA71 3736 5721 7292 6969 8418 3239 3",
                "id": "00000000-0000-0000-0000-000000000101",
                "name":  "USD account 1",
                "updatedAt": "2021-11-10T15:14:15Z",
            }
        )
    );

    /*
    assert_json_include!(
        // actual: accounts, // not supported so crazy include :-)
        actual: first_account,
        expected: json!(
            {
                "amount": {
                  "currency": "USD",
                  "value": 150
                },
                "clientId": "00000000-0000-0000-0000-000000000001",
                "createdAt": "2021-11-10T15:14:13Z",
                "iban":  "UA71 3736 5721 7292 6969 8418 3239 3",
                "id": "00000000-0000-0000-0000-000000000101",
                "name":  "USD account 1",
                "updatedAt": "2021-11-10T15:14:15Z",
            }
        )
    );
    */

    info!("test_get_all_client_accounts SUCCEEDED");

    Ok(())
}

