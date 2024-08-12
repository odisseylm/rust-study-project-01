use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::time::Duration;
use anyhow::anyhow;
use assert_json_diff::{assert_json_eq, assert_json_include};
use assertables::{assert_ge, assert_ge_as_result};
use log::{error, info, warn};
use reqwest::Response;
use rustainers::compose::{
    ComposeContainers, ComposeError, ComposeRunOption,
    RunnableComposeContainers, RunnableComposeContainersBuilder,
    ToRunnableComposeContainers,
};
use rustainers::{ExposedPort, Port, WaitStrategy};
use mvv_common::test::integration::{ docker_compose_down_silent as docker_compose_down, prepare_docker_compose, PrepareDockerComposeCfg};
use mvv_common::test::{TestOptionUnwrap, TestResultUnwrap};
use serde_json::json;
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
                (ACCOUNT_SOA_SERVICE, WaitStrategy::HttpSuccess {
                    https: false,
                    path: "/health-check".to_owned(),
                    container_port: ACCOUNT_SOA_HTTP_PORT.clone(),
                }),
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

    let sub_project_dir = std::env::var("CARGO_MANIFEST_DIR").test_unwrap();
    let sub_project_dir: PathBuf = sub_project_dir.into();
    let project_dir = sub_project_dir.join("..").canonicalize().test_unwrap();

    use mvv_common::test::integration::Copy;

    let cfg = PrepareDockerComposeCfg {
        name: "rust_account_soa".to_owned(),
        base_from_dir: sub_project_dir,
        copy: vec!(
            Copy { from: p("docker/docker-compose.env"), to: p("docker-compose.env") },
            Copy { from: p("docker/docker-compose.yml"), to: p("docker-compose.yml") },
            Copy { from: p("test_resources/postgres"), to: p("test_resources/postgres") },
        ),
        replace_file_content: vec!(
            /*
            Replace {
                file: p("docker-compose.yml"),
                from: vec!("./test_resources/postgres/init/".to_owned()),
                to: vec!("SomeMyCustomPath".to_owned()),
            },
            */
        ),
    };

    let temp_docker_compose_dir = prepare_docker_compose(&project_dir, &cfg) ?;

    let option: ComposeRunOption = ComposeRunOption::builder()
        // Wait interval for service health check
        .with_wait_interval(Duration::from_secs(1))
        // Wait interval for service to exist
        .with_wait_services_interval(Duration::from_secs(2))
        .build();

    let runner = rustainers::runner::Runner::docker() ?;

    info!("Attempt to run docker compose for [account_soa]", );

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
// #[ignore]
async fn run_account_soa_docker_compose() {

    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    let (temp_docker_compose_dir, compose_containers) = launch_account_soa_docker_compose().await.test_unwrap();

    // THERE should be real REST tests.
    tokio::time::sleep(Duration::from_secs(2)).await; // just in case

    let port = compose_containers.account_soa_http_port.host_port().await;
    if port.is_err() {
        docker_compose_down(&temp_docker_compose_dir);
    }

    let port: u16 = port.test_unwrap().into();

    use futures::future::FutureExt;
    let test_res = AssertUnwindSafe(test_get_all_client_accounts(port)).catch_unwind().await;

    if test_res.is_err() {
        docker_compose_down(&temp_docker_compose_dir);
    }

    let test_res = test_res.test_unwrap();
    if test_res.is_err() {
        docker_compose_down(&temp_docker_compose_dir);
    }
    test_res.test_unwrap();

    let pause_timeout = Duration::from_secs(10);
    info!("### Pause for {}s...", pause_timeout.as_secs());
    tokio::time::sleep(pause_timeout).await;

    let _ = compose_containers;
    info!("### Stopping containers...");

    // to make sure to 'remove containers, networks'
    info!("### Cleaning...");
    docker_compose_down(&temp_docker_compose_dir);

    info!("### Test is completed");
}


async fn test_get_all_client_accounts(account_soa_port: u16) -> anyhow::Result<()> {

    let base_url = format!("http://localhost:{account_soa_port}");
    let url = format!("{base_url}/api/client/00000000-0000-0000-0000-000000000001/account/all");

    let client = reqwest::Client::new();
    let resp: Response = client.get(url)
        .basic_auth("vovan-read", Some("qwerty"))
        .send()
        .await ?;

    assert_eq!(resp.status().as_u16(), 200);

    use core::str::FromStr;
    let body_as_str: String = resp.text().await ?;

    let actual = serde_json::Value::from_str(&body_as_str).test_unwrap();
    println!("### response: {actual}");

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

    Ok(())
}
