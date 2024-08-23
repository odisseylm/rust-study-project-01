use std::path::{Path, PathBuf};
use std::time::Duration;
use anyhow::anyhow;
use assert_json_diff::{assert_json_eq};
use assertables::{assert_ge, assert_ge_as_result};
use itertools::Itertools;
use log::{debug, error, info};
use reqwest::{Certificate, Response};
use rustainers::compose::{
    ComposeContainers, ComposeError, ComposeRunOption,
    RunnableComposeContainers, RunnableComposeContainersBuilder,
    ToRunnableComposeContainers,
};
use rustainers::{ExposedPort, Port, WaitStrategy};
use mvv_common::test::integration::{
    AutoDockerComposeDown, docker_compose_down_silent as docker_compose_down,
    is_integration_tests_enabled, prepare_docker_compose,
    PrepareDockerComposeCfg, Replace, get_docker_image_profile,
};
use mvv_common::test::{TestOptionUnwrap, TestResultUnwrap};
use serde_json::json;
use mvv_common::fn_name;
//--------------------------------------------------------------------------------------------------



const ACCOUNT_SOA_SERVICE: &'static str = "rust-account-soa";
const ACCOUNT_SOA_HTTP_PORT: Port = Port::new(8080);

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


async fn launch_account_soa_docker_compose() -> anyhow::Result<(PathBuf, ComposeContainers<AccountSoaTestContainers>)> {

    fn p(path: &str) -> PathBuf { PathBuf::from(path) }

    // CARGO_MANIFEST_DIR = /home/.../rust-study-project-01/account_soa
    // CARGO_PKG_NAME = mvv_account_soa
    // OUT_DIR = /home/.../target/debug/build/mvv_account_soa-a02bfbee150dfc8f/out

    let tests_session_id = chrono::Local::now().timestamp();

    let docker_image_profile = get_docker_image_profile();
    let docker_image_profile_suffix: &str = docker_image_profile.as_docker_tag_suffix();

    let sub_project_dir = std::env::var("CARGO_MANIFEST_DIR").test_unwrap();
    let sub_project_dir: PathBuf = sub_project_dir.into();
    let project_dir = sub_project_dir.join("..").canonicalize().test_unwrap();
    let new_network = format!("it-tests-rust-account-soa-{tests_session_id}");

    use mvv_common::test::integration::Copy;

    let cfg = PrepareDockerComposeCfg {
        tests_session_id,
        // Project names must contain only lowercase letters, decimal digits, dashes,
        // and underscores, and must begin with a lowercase letter or decimal digit.
        // See https://github.com/compose-spec/compose-spec/blob/main/spec.md
        //
        name: "rust_account_soa".to_owned(),
        base_from_dir: sub_project_dir,
        copy: vec!(
            Copy { from: p("docker/docker-compose.env"), to: p("docker-compose.env") },
            Copy { from: p("docker/docker-compose.yml"), to: p("docker-compose.yml") },
            Copy { from: p("test_resources/postgres"), to: p("test_resources/postgres") },
            Copy { from: p("../target/generated-test-resources/ssl"), to: p("generated-test-resources/ssl") },
        ),
        replace_file_content: vec!(
            /*
            Replace {
                file: p("docker-compose.yml"),
                from: vec!("./test_resources/postgres/init/".to_owned()),
                to: vec!("SomeMyCustomPath".to_owned()),
            },
            */
            Replace::by_str(
                p("docker-compose.yml"),
                [
                    "name: account-soa-it-tests",
                    "name:account-soa-it-tests",
                    "name:\taccount-soa-it-tests",
                ],
                [&format!("name: {new_network}")],
            ),
            Replace::by_str(
                p("docker-compose.yml"),
                ["../target/generated-test-resources/ssl/"],
                ["./generated-test-resources/ssl/"],
            ),
            Replace::by_str(
                p("docker-compose.yml"),
                ["${DOCKER_IMAGE_PROFILE_SUFFIX}"],
                [docker_image_profile_suffix],
            ),
            //
            // Disable port's publishing to specific/hardcoded ports because they can be used already
            // and test will fail.
            // It is needed
            // * to have possibility to run integration tests without stopping
            //   already launched testing docker-compose environment
            // * to safe launch integration tests on build server
            //
            Replace::by_str(
                p("docker-compose.yml"),
                [ "- 5432:5432", "-5432:5432", "-\t5432:5432", ],
                ["- 5432"],
            ),
            Replace::by_str(
                p("docker-compose.yml"),
                [ "- 8101:8080", "-8101:8080", "-\t8101:8080", ],
                ["- 8080"],
            ),
        ),
    };

    let temp_docker_compose_dir = prepare_docker_compose(&project_dir, &cfg) ?;

    // let docker_image_profile = get_docker_image_profile();
    // let docker_image_profile_suffix: &str = docker_image_profile.as_docker_tag_suffix();
    std::env::set_var("DOCKER_IMAGE_PROFILE_SUFFIX", docker_image_profile_suffix);

    let option: ComposeRunOption = ComposeRunOption::builder()
        // Wait interval for service health check
        .with_wait_interval(Duration::from_secs(1))
        // Wait interval for service to exist
        .with_wait_services_interval(Duration::from_secs(2))
        .build();

    let runner = rustainers::runner::Runner::docker() ?;

    info!("### Attempt to run docker compose for [account_soa]", );

    // to make sure - clean up previous session
    info!("#### Clean previous docker compose session");
    docker_compose_down(&temp_docker_compose_dir);

    let compose_containers_fut = runner.compose_start_with_options(
        AccountSoaTestContainers::new(&temp_docker_compose_dir).await ?,
        option,
    );

    let pause_duration = Duration::from_secs(15); // 120
    let compose_containers = tokio::time::timeout(pause_duration, compose_containers_fut).await;

    match compose_containers {
        Ok(res) => {
            match res {
                Ok(res) => {
                    info!("SUCCESS of Account SOA launch => {res:?}");
                    Ok( (temp_docker_compose_dir, res) )
                },
                Err(err) => {
                    error!("FAILURE of Account SOA launch => Error {{ {err:?} }}");
                    docker_compose_down(&temp_docker_compose_dir);
                    Err(anyhow!(err))
                },
            }
        },
        Err(err) => {
            error!("FAILURE of Account SOA launch => Error {{ {err:?} }}");
            info!("Shut down docker compose manually...");
            docker_compose_down(&temp_docker_compose_dir);
            Err(anyhow!(err))
        },
    }
}



#[tokio::test]
async fn integration_test_run_account_soa_docker_compose() {

    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    // let vars =  std::env::vars().map(|(k,v)|format!("{k:?} = {v:?}")).sorted().join("\n");
    // println!("vars: {vars}");

    if !is_integration_tests_enabled() {
        info!("Integration test [{}] is SKIPPED/IGNORED", fn_name!());
        return;
    }

    let (temp_docker_compose_dir, compose_containers) = launch_account_soa_docker_compose().await.test_unwrap();
    let auto_docker_compose_down = AutoDockerComposeDown {
        docker_compose_file_dir: temp_docker_compose_dir.to_path_buf(),
        log_message: Some("### Cleaning..."),
    };

    // THERE should be real REST tests.
    tokio::time::sleep(Duration::from_secs(2)).await; // just in case

    let port = compose_containers.account_soa_http_port.host_port().await;

    let port: u16 = port.test_unwrap().into();

    test_get_all_client_accounts(port).await;

    // let pause_timeout = Duration::from_secs(5);
    // info!("### Pause for {}s...", pause_timeout.as_secs());
    // tokio::time::sleep(pause_timeout).await;

    let _ = compose_containers;
    info!("### Stopping containers...");

    // to make sure to 'remove containers, networks'
    info!("### Cleaning...");
    let _ = auto_docker_compose_down;

    info!("### Test is completed");
}


async fn test_get_all_client_accounts(account_soa_port: u16) {

    let base_url = format!("https://localhost:{account_soa_port}");
    let url = format!("{base_url}/api/client/00000000-0000-0000-0000-000000000001/account/all");

    // "CARGO_MANIFEST_DIR" = "/home/vmelnykov/projects/rust/rust-study-project-01/account_soa"
    let project_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR").test_unwrap().into();
    let root_dir: &Path = project_dir.parent().test_unwrap();
    let cert_path = root_dir.join("target/generated-test-resources/ssl/ca.crt.pem");

    let pem: String = std::fs::read_to_string(&cert_path)
        .map_err(|err| anyhow!("Error of reading from [{cert_path:?}] ({err:?})")).test_unwrap();

    let client = reqwest::Client::builder()
        // .danger_accept_invalid_certs(true)
        .add_root_certificate(Certificate::from_pem(pem.as_bytes()).test_unwrap())
        .build().test_unwrap();

    let resp: Response = client.get(url)
        .basic_auth("vovan-read", Some("qwerty"))
        .send()
        .await
        .test_unwrap();

    assert_eq!(resp.status().as_u16(), 200);

    use core::str::FromStr;
    let body_as_str: String = resp.text().await.test_unwrap();

    let actual = serde_json::Value::from_str(&body_as_str).test_unwrap();
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
}


/*
/// We use this special function wrapper to avoid missed clean up
/// due to panic by assert (or unwrap in test code)
/// We also could use `AssertUnwindSafe(our_test_method_future).catch_unwind().await`
/// but this approach is easier and works enough properly (at least in our tests).
async fn run_test<>(f: impl Future<Output=()> + Send + Sync + 'static) -> anyhow::Result<()> {
    let res = tokio::spawn(f).await;
    match res {
        Ok(res) => Ok(res),
        Err(err) => Err(anyhow!(err)),
    }
}
*/
