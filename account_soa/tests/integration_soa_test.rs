use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::Duration;
use anyhow::anyhow;
use assert_json_diff::{assert_json_eq};
use assertables::{assert_ge, assert_ge_as_result};
use log::{debug, info};
use reqwest::{Certificate, Response};
use rustainers::compose::{
    ComposeContainers, ComposeError, ComposeRunOption,
    RunnableComposeContainers, RunnableComposeContainersBuilder,
    ToRunnableComposeContainers,
};
use rustainers::{ExposedPort, Port, WaitStrategy};
use mvv_common::test::integration::{AutoDockerComposeDown, is_integration_tests_enabled, prepare_docker_compose, PrepareDockerComposeCfg, wait_containers, };
use mvv_common::test::{current_project_target_dir, current_sub_project_dir, TestOptionUnwrap, TestResultUnwrap};
use serde_json::json;
use mvv_common::fn_name;
use mvv_common::test::docker_compose::docker_compose_down;
use mvv_common::test::files::CopyCfg;
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


async fn launch_account_soa_docker_compose() -> anyhow::Result<(PathBuf, ComposeContainers<AccountSoaTestContainers>)> {

    let tests_session_id = chrono::Local::now().timestamp();
    let sub_project_dir = current_sub_project_dir().test_unwrap();

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

    let option: ComposeRunOption = ComposeRunOption::builder()
        // Wait interval for service health check
        .with_wait_interval(Duration::from_secs(1))
        // Wait interval for service to exist
        .with_wait_services_interval(Duration::from_secs(2))
        .build();

    let runner = rustainers::runner::Runner::docker() ?;

    info!("### Attempt to run docker compose for [account_soa]", );

    // to make sure - clean up previous session
    info!("### Clean previous docker compose session");
    let _ = docker_compose_down(&temp_docker_compose_dir);

    let compose_containers_fut = runner.compose_start_with_options(
        AccountSoaTestContainers::new(&temp_docker_compose_dir).await ?,
        option,
    );

    // TODO: use long timeout if third-party docker images are not loaded from net yet
    wait_containers(temp_docker_compose_dir, "Account SOA", compose_containers_fut, Duration::from_secs(15)).await
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

    let build_target_dir = current_project_target_dir().test_unwrap();
    let cert_path = build_target_dir.join("generated-test-resources/ssl/ca.crt.pem");

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

