use core::fmt::Debug;
use std::{
    path::PathBuf,
};
use anyhow::anyhow;
use assert_json_diff::assert_json_eq;
use assertables::{assert_ge, assert_ge_as_result};
use log::{debug, info};
use reqwest::{Certificate, Response};
use mvv_common_it_test::thirdparty::rustainers::{
    compose::{
        ComposeError,
        RunnableComposeContainers, RunnableComposeContainersBuilder,
        ToRunnableComposeContainers,
    },
    ExposedPort, Port, WaitStrategy,
};
use serde_json::json;
use mvv_common::{
    fn_name,
    test::{BuildEnv, current_project_target_dir, TestOptionUnwrap},
};
use mvv_common_it_test::{
    is_integration_tests_enabled,
    coverage::{copy_code_coverage_files_for_all_containers},
    docker_compose::AutoDockerComposeDown,
    integration::{ComposeContainersState, launch_soa_docker_compose, LaunchSoaDockerComposeParams},
};
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


// Not the best solution, it would be more properly to use some new trait-factory...,
// but let's live with this :-)
//
impl TryFrom<PathBuf> for AccountSoaTestContainers {
    type Error = ComposeError;
    fn try_from(dir: PathBuf) -> Result<Self, Self::Error> {
        Ok(Self {
            dir,
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

    let build_env = BuildEnv::try_new() ?;
    let exe_file = build_env.target_profile_dir.join("mvv_account_soa");

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
    } = launch_soa_docker_compose::<AccountSoaTestContainers>(&params).await ?;
    let docker_compose_dir = docker_compose_dir.as_path();

    let auto_docker_compose_down = AutoDockerComposeDown {
        docker_compose_file_dir: docker_compose_dir.to_path_buf(),
        log_message: Some("### Cleaning..."),
    };

    let port = compose_containers.account_soa_http_port.host_port().await;
    let port: u16 = port ?.into();

    //----------------------------------------------------------------------------------------------
    //                             Real tests
    //
    test_get_all_client_accounts(port).await ?;
    //----------------------------------------------------------------------------------------------

    if is_code_coverage_enabled {
        copy_code_coverage_files_for_all_containers(docker_compose_dir, &container_name_label, &[]) ?;
    }

    // Uncomment it for docker state analysis.
    //
    // let pause_timeout = Duration::from_secs(5);
    // info!("### Pause for {}s...", pause_timeout.as_secs());
    // tokio::time::sleep(pause_timeout).await;

    let _ = compose_containers; // do it gracefully before 'down'
    info!("### Stopping containers...");

    // to make sure to 'remove containers, networks'
    info!("### Cleaning...");
    let _ = auto_docker_compose_down; // optional, just to have nice predictable output

    info!("### Test [integration_test_run_account_soa_docker_compose] is completed SUCCESSFULLY");

    Ok(())
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

