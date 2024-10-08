use core::{
    fmt::Debug,
    future::Future,
    time::Duration,
};
use std::{
    ffi::OsString,
    path::{PathBuf},
};
use anyhow::anyhow;
use itertools::Itertools;
use log::{error, info};
use crate::{
    docker_compose::{docker_compose_down},
};
//--------------------------------------------------------------------------------------------------



pub async fn wait_rustainers<C>(
    res_docker_compose_dir: PathBuf,
    docker_compose_display_name: &str,
    fut: impl Future<Output=Result<rustainers::compose::ComposeContainers<C>, rustainers::runner::RunnerError>>,
    pause_duration: Duration,
) -> anyhow::Result<(PathBuf, rustainers::compose::ComposeContainers<C>)>
where
    C: rustainers::compose::ToRunnableComposeContainers + Debug,
    rustainers::compose::ComposeContainers<C>: Debug,
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
