use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::Duration;
use anyhow::anyhow;
use assert_json_diff::{assert_json_eq};
use assertables::{assert_contains, assert_contains_as_result};
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
use mvv_common::string::remove_repeated_spaces;
use mvv_common::test::docker_compose::docker_compose_down;
//--------------------------------------------------------------------------------------------------



const ACCOUNT_SOA_SERVICE: &'static str = "rust-account-soa";
const ACCOUNT_SOA_HTTP_PORT: Port = Port::new(8443);

const ACCOUNT_WEB_SERVICE: &'static str = "rust-account-web";
const ACCOUNT_WEB_HTTP_PORT: Port = Port::new(8443);

const POSTGRES_SERVICE: &'static str = "database";
const POSTGRES_PORT: Port = Port::new(5432);
// Other ports


#[derive(Debug)]
struct AccountWebTestContainers {
    dir: PathBuf,
    account_soa_http_port: ExposedPort,
    account_web_http_port: ExposedPort,
    // debug_port: ExposedPort,
    postgres_port: ExposedPort,
}


impl AccountWebTestContainers {
    pub async fn new(dir: &Path) -> Result<Self, ComposeError> {
        Ok(Self {
            dir: dir.to_path_buf(),
            account_soa_http_port: ExposedPort::new(ACCOUNT_SOA_HTTP_PORT.clone()),
            account_web_http_port: ExposedPort::new(ACCOUNT_WEB_HTTP_PORT.clone()),
            postgres_port: ExposedPort::new(POSTGRES_PORT.clone()),
        })
    }
}

impl ToRunnableComposeContainers for AccountWebTestContainers {
    type AsPath = PathBuf; // TemporaryDirectory, TemporaryFile

    fn to_runnable(&self, builder: RunnableComposeContainersBuilder<Self::AsPath>) -> RunnableComposeContainers<Self::AsPath> {
        builder
            // Only directory can be passed :-(
            .with_compose_path(self.dir.clone())
            .with_port_mappings([
                (ACCOUNT_SOA_SERVICE, self.account_soa_http_port.clone()),
                (ACCOUNT_WEB_SERVICE, self.account_web_http_port.clone()),
                (POSTGRES_SERVICE, self.postgres_port.clone()),
            ])
            .with_wait_strategies([
                (ACCOUNT_SOA_SERVICE, WaitStrategy::HealthCheck),
                (ACCOUNT_WEB_SERVICE, WaitStrategy::HealthCheck),
                (POSTGRES_SERVICE, WaitStrategy::stdout_match(
                    regex::Regex::new("PostgreSQL init process complete; ready for start up")
                        .expect("Incorrect RegEx for PostgreSQL."))),
            ])
            .build()
    }
}


async fn launch_account_web_docker_compose() -> anyhow::Result<(PathBuf, ComposeContainers<AccountWebTestContainers>)> {

    let tests_session_id = chrono::Local::now().timestamp();
    let sub_project_dir = current_sub_project_dir().test_unwrap();

    let cfg = PrepareDockerComposeCfg {
        tests_session_id,
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
        AccountWebTestContainers::new(&temp_docker_compose_dir).await ?,
        option,
    );

    wait_containers(temp_docker_compose_dir, "Account WEB", compose_containers_fut, Duration::from_secs(15)).await
}


#[tokio::test]
async fn integration_test_run_account_web_docker_compose() {

    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    if !is_integration_tests_enabled() {
        info!("Integration test [{}] is SKIPPED/IGNORED", fn_name!());
        return;
    }

    let (temp_docker_compose_dir, compose_containers) = launch_account_web_docker_compose().await.test_unwrap();
    let auto_docker_compose_down = AutoDockerComposeDown {
        docker_compose_file_dir: temp_docker_compose_dir.to_path_buf(),
        log_message: Some("### Cleaning..."),
    };

    // THERE should be real REST tests.
    tokio::time::sleep(Duration::from_secs(2)).await; // just in case

    let soa_port = compose_containers.account_soa_http_port.host_port().await;
    // let soa_port = ExposedPort::new(ACCOUNT_SOA_HTTP_PORT.clone()).host_port().await;
    let soa_port: u16 = soa_port.test_unwrap().into();
    test_soa_get_all_client_accounts(soa_port).await;

    let web_port = compose_containers.account_web_http_port.host_port().await;
    // let web_port = ExposedPort::new(ACCOUNT_WEB_HTTP_PORT.clone()).host_port().await;
    let web_port: u16 = web_port.test_unwrap().into();
    test_web_get_all_client_accounts(web_port).await;

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


// Duplicate from account SOA test
async fn test_web_get_all_client_accounts(port: u16) {
    let base_url = format!("https://localhost:{port}");
    let url = format!("{base_url}/ui/current_client_accounts");

    let build_target_dir = current_project_target_dir().test_unwrap();
    let cert_path = build_target_dir.join("generated-test-resources/ssl/ca.crt.pem");

    let pem: String = std::fs::read_to_string(&cert_path)
        .map_err(|err| anyhow!("Error of reading from [{cert_path:?}] ({err:?})")).test_unwrap();

    let client = reqwest::Client::builder()
        // .danger_accept_invalid_certs(true)
        .add_root_certificate(Certificate::from_pem(pem.as_bytes()).test_unwrap())
        .build().test_unwrap();

    let resp: Response = client.get(url)
        .basic_auth("cheburan@ukr.net", Some("qwerty"))
        .send()
        .await
        .test_unwrap();

    assert_eq!(resp.status().as_u16(), 200);

    let content_type = resp.headers().get("Content-Type").test_unwrap();
    assert_eq!(content_type, "text/html; charset=utf-8");

    let body_as_str: String = resp.text().await.test_unwrap();

    assert_contains!(body_as_str, "<title>Client accounts</title>");
    assert_contains!(body_as_str, "<p>Client 00000000-0000-0000-0000-000000000001</p>");

    // let as_text = html2text::parse(&body_as_str).test_unwrap();
    //
    // Other approach regex  replace(/<[^>]*>/g, ‘’)
    // See https://dev.to/sanchithasr/3-ways-to-convert-html-text-to-plain-text-52l8

    let as_text = nanohtml2text::html2text(&body_as_str);

    let as_text: String = remove_repeated_spaces(&as_text.trim());

    let expected = "Client 00000000-0000-0000-0000-000000000001  Accounts \
      Account info   UA71 3736 5721 7292 6969 8418 3239 3   \
      Name  USD account 1   Amount  150 USD   \
      Updated at  2021-11-10 15:14:15 UTC   Created at  2021-11-10 15:14:13 UTC     \
      Account info   UA94 8614 7668 5733 7364 6254 6466 8   \
      Name  USD account 2   Amount  250 USD   \
      Updated at  2021-11-11 17:00:00 UTC   Created at  2021-11-11 16:00:00 UTC     \
      Account info   UA56 5117 3742 7482 6517 5455 3347 9   \
      Name  UAH account 1   Amount  1000 UAH   \
      Updated at  2021-11-12 18:12:00 UTC   Created at  2021-11-12 17:00:00 UTC     \
      Account info   UA49 6826 1538 4394 4716 5383 8292 8   \
      Name  UAH account 2   Amount  2000 UAH   \
      Updated at  2021-11-15 15:00:00 UTC   Created at  2021-11-13 10:00:00 UTC";
    let expected: String = remove_repeated_spaces(expected.trim());

    // Temp easy solution! Probably makes sense to use more complicated solution.
    assert_contains!(&as_text, &expected);
}


// Duplicate from account SOA test
async fn test_soa_get_all_client_accounts(account_soa_port: u16) {

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
}
