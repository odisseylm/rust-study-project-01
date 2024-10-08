use core::fmt::Debug;
use std::{
    path::PathBuf,
    time::Duration,
};
use anyhow::anyhow;
use assert_json_diff::assert_json_eq;
use assertables::{assert_contains, assert_contains_as_result};
use log::{debug, info};
use reqwest::{Certificate, Response};
use mvv_common_it_test::thirdparty::rustainers::{
    ExposedPort, Port, WaitStrategy,
    compose::{
        ComposeError,
        RunnableComposeContainers, RunnableComposeContainersBuilder,
        ToRunnableComposeContainers,
    },
};
use serde_json::json;
use mvv_common::{
    fn_name, string::remove_repeated_spaces,
    test::{
        BuildEnv,TestDisplayStringOps, TestOptionUnwrap,
        current_project_target_dir,
    },
};
use mvv_common_it_test::{
    is_integration_tests_enabled,
    docker_compose::AutoDockerComposeDown,
    coverage::copy_code_coverage_files_for_all_containers,
    integration::{ComposeContainersState, launch_soa_docker_compose, LaunchSoaDockerComposeParams},
};
//--------------------------------------------------------------------------------------------------



const ACCOUNT_SOA_SERVICE: &'static str = "rust-account-soa";
const ACCOUNT_SOA_HTTP_PORT: Port = Port::new(8443);

const CLIENT_SEARCH_SOA_SERVICE: &'static str = "rust-client-search-soa";
const CLIENT_SEARCH_SOA_HTTP_PORT: Port = Port::new(8443);

const ACCOUNT_WEB_SERVICE: &'static str = "rust-account-web";
const ACCOUNT_WEB_HTTP_PORT: Port = Port::new(8443);

const POSTGRES_SERVICE: &'static str = "database";
const POSTGRES_PORT: Port = Port::new(5432);
// Other ports


#[derive(Debug)]
struct AccountWebTestContainers {
    dir: PathBuf,
    account_soa_http_port: ExposedPort,
    client_search_soa_http_port: ExposedPort,
    account_web_http_port: ExposedPort,
    // debug_port: ExposedPort,
    postgres_port: ExposedPort,
}


// Not the best solution, it would be more properly to use some new trait-factory...,
// but let's live with this :-)
//
impl TryFrom<PathBuf> for AccountWebTestContainers {
    type Error = ComposeError;
    fn try_from(dir: PathBuf) -> Result<Self, Self::Error> {
        Ok(Self {
            dir,
            account_soa_http_port: ExposedPort::new(ACCOUNT_SOA_HTTP_PORT.clone()),
            client_search_soa_http_port: ExposedPort::new(CLIENT_SEARCH_SOA_HTTP_PORT.clone()),
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
                (CLIENT_SEARCH_SOA_SERVICE, self.client_search_soa_http_port.clone()),
                (ACCOUNT_WEB_SERVICE, self.account_web_http_port.clone()),
                (POSTGRES_SERVICE, self.postgres_port.clone()),
            ])
            .with_wait_strategies([
                (ACCOUNT_SOA_SERVICE, WaitStrategy::HealthCheck),
                (CLIENT_SEARCH_SOA_SERVICE, WaitStrategy::HealthCheck),
                (ACCOUNT_WEB_SERVICE, WaitStrategy::HealthCheck),
                (POSTGRES_SERVICE, WaitStrategy::stdout_match(
                    regex::Regex::new("PostgreSQL init process complete; ready for start up")
                        .expect("Incorrect RegEx for PostgreSQL."))),
            ])
            .build()
    }
}


/*
async fn launch_account_web_docker_compose() -> anyhow::Result<(PathBuf, ComposeContainers<AccountWebTestContainers>)> {

    // let tests_session_id = chrono::Local::now().timestamp();
    let tests_session_id = 0; // mvv_common::test::build_id() ?;
    let sub_project_dir = current_sub_project_dir().test_unwrap();

    let cfg = PrepareDockerComposeCfg {
        tests_session_id,
        ..Default::default()
    };

    let temp_docker_compose_dir = prepare_docker_compose(&sub_project_dir, &cfg) ?;

    preload_docker_compose_images(&get_docker_compose_file(&temp_docker_compose_dir) ?) ?;

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

    wait_rustainers(temp_docker_compose_dir, "Account WEB", compose_containers_fut, Duration::from_secs(15)).await
}
*/


#[tokio::test]
async fn integration_test_run_account_web_docker_compose() -> anyhow::Result<()> {

    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    if !is_integration_tests_enabled() {
        info!("Integration test [{}] is SKIPPED/IGNORED", fn_name!());
        return Ok(());
    }

    let build_env = BuildEnv::try_new() ?;
    let exe_file = build_env.target_profile_dir.join("mvv_account_web");

    let params = LaunchSoaDockerComposeParams {
        exe_file: Some(exe_file),
        .. Default::default()
    };

    let ComposeContainersState {
        docker_compose_dir,
        compose_containers,
        is_code_coverage_enabled,
        container_name_label,
        ..
        // } = launch_account_soa_docker_compose().await ?;
    } = launch_soa_docker_compose::<AccountWebTestContainers>(&params).await ?;
    let docker_compose_dir = docker_compose_dir.as_path();

    let auto_docker_compose_down = AutoDockerComposeDown {
        docker_compose_file_dir: docker_compose_dir.to_path_buf(),
        log_message: Some("### Cleaning..."),
    };

    // THERE should be real REST tests.
    tokio::time::sleep(Duration::from_secs(2)).await; // just in case

    let soa_port = compose_containers.account_soa_http_port.host_port().await;
    let soa_port: u16 = soa_port ?.into();
    test_soa_get_all_client_accounts(soa_port).await ?;

    let web_port = compose_containers.account_web_http_port.host_port().await;
    let web_port: u16 = web_port ?.into();
    test_web_get_all_client_accounts(web_port).await ?;

    if is_code_coverage_enabled {
        copy_code_coverage_files_for_all_containers(docker_compose_dir, &container_name_label, &[]) ?;
    }

    // let pause_timeout = Duration::from_secs(5);
    // info!("### Pause for {}s...", pause_timeout.as_secs());
    // tokio::time::sleep(pause_timeout).await;

    let _ = compose_containers; // do it gracefully before 'down'
    info!("### Stopping containers...");

    // to make sure to 'remove containers, networks'
    info!("### Cleaning...");
    let _ = auto_docker_compose_down; // optional, just to have nice predictable output

    info!("### Test [integration_test_run_account_web_docker_compose] is completed SUCCESSFULLY");

    Ok(())
}


// Duplicate from account SOA test
async fn test_web_get_all_client_accounts(port: u16) -> anyhow::Result<()> {
    let base_url = format!("https://localhost:{port}");
    let url = format!("{base_url}/ui/current_client_accounts");

    let build_target_dir = current_project_target_dir() ?;
    let cert_path = build_target_dir.join("generated-test-resources/ssl/ca.crt.pem");

    let pem: String = std::fs::read_to_string(&cert_path)
        .map_err(|err| anyhow!("Error of reading from [{cert_path:?}] ({err:?})")) ?;

    let client = reqwest::Client::builder()
        // .danger_accept_invalid_certs(true)
        .add_root_certificate(Certificate::from_pem(pem.as_bytes()) ?)
        .build() ?;

    let resp: Response = client.get(url)
        .basic_auth("cheburan@ukr.net", Some("qwerty"))
        .send()
        .await ?;

    assert_eq!(resp.status().as_u16(), 200);

    let content_type = resp.headers().get("Content-Type")
        .ok_or_else(|| anyhow!("No header [Content-Type]")) ?;
    assert_eq!(content_type, "text/html; charset=utf-8");

    let body_as_str: String = resp.text().await ?;

    assert_contains!(body_as_str, "<title>Client accounts</title>");
    assert_contains!(body_as_str, "<p>Client 00000000-0000-0000-0000-000000000001</p>");

    // let as_text = html2text::parse(&body_as_str).test_unwrap();
    //
    // Other approach regex  replace(/<[^>]*>/g, ‘’)
    // See https://dev.to/sanchithasr/3-ways-to-convert-html-text-to-plain-text-52l8

    let as_text = nanohtml2text::html2text(&body_as_str);

    let as_text: String = remove_repeated_spaces(&as_text.trim());

    let expected_top = "Client 00000000-0000-0000-0000-000000000001";

    // Temp easy solution! Probably makes sense to use more complicated solution.
    assert_contains!(&as_text, &expected_top);

    let expected_accounts_part = "Accounts \
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
    let expected: String = normalize_test_substr(expected_accounts_part);

    // Temp easy solution! Probably makes sense to use more complicated solution.
    assert_contains!(&as_text, &expected);

    let expected_client_info_part =
        "Active 	true \
        First name 	Cheburan \
        Last name 	Vovan \
        Email 	cheburan@ukr.net \
        Phones 	+380671234567 (Mobile) \
        Birthday 	2000-02-28 \
        Client type 	General";
    let expected_client_info_part: String = normalize_test_substr(expected_client_info_part);

    let start_index = as_text.find("Active true");
    if let Some(start_index) = start_index {
        let client_info_part_start = as_text.to_test_string().split_off(start_index);
        assert_text::assert_text_starts_with!(client_info_part_start, expected_client_info_part.as_str());
        // pretty_assertions::assert_eq!(client_info_part_start, expected_client_info_part);
    }

    // Temp easy solution! Probably makes sense to use more complicated solution.
    // pretty_assertions::assert_matches!()
    assert_contains!(&as_text, &expected_client_info_part);
    assert_contains!(&as_text, &expected_client_info_part);

    info!("test_web_get_all_client_accounts SUCCEEDED");

    Ok(())
}

fn normalize_test_substr(str: &str) -> String {
    remove_repeated_spaces(&str.trim().replace('\t', " "))
}


// Duplicate from account SOA test
async fn test_soa_get_all_client_accounts(account_soa_port: u16) -> anyhow::Result<()> {

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

    Ok(())
}
