use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use anyhow::anyhow;
use itertools::{Either, Itertools};
use log::{debug, info, warn};
use x509_parser::nom::AsBytes;
use yaml_rust2::Yaml;
use crate::test::{
    is_CI_build, is_manually_launched_task,
    integration::load_yaml,
};
//--------------------------------------------------------------------------------------------------



fn call_docker_compose_cmd(docker_compose_file_dir: &Path, cmd: &str) -> anyhow::Result<()> {
    call_docker_compose_cmd_2(docker_compose_file_dir, cmd, &[])
}

fn call_docker_compose_cmd_2(docker_compose_file_dir: &Path, cmd: &str, args: &[&str]) -> anyhow::Result<()> {

    let docker_compose_file = get_docker_compose_file(docker_compose_file_dir) ?;
    if !docker_compose_file.exists() {
        anyhow::bail!("No file [{docker_compose_file:?}].")
    }

    Command::new("docker")
        .current_dir(docker_compose_file_dir.to_path_buf())
        .arg("compose")
        .arg(cmd)
        .args(args)
        .status() ?;
    Ok(())
}


pub fn call_docker_compose_cmd_silent(docker_compose_file_dir: &Path, cmd: &str) {
    let res = call_docker_compose_cmd(docker_compose_file_dir, cmd);
    if let Err(err) = res {
        warn!("Docker compose down failed => {err:?}")
    }
}


pub fn docker_compose_down(docker_compose_file_dir: &Path) -> anyhow::Result<()> {
    call_docker_compose_cmd(docker_compose_file_dir, "down")
}
pub fn docker_compose_down_silent(docker_compose_file_dir: &Path) {
    call_docker_compose_cmd_silent(docker_compose_file_dir, "down")
}


pub fn docker_compose_start(docker_compose_file_dir: &Path) -> anyhow::Result<()> {
    call_docker_compose_cmd(docker_compose_file_dir, "start")
}
pub fn docker_compose_start_silent(docker_compose_file_dir: &Path) {
    call_docker_compose_cmd_silent(docker_compose_file_dir, "start")
}


pub fn docker_compose_stop(docker_compose_file_dir: &Path) -> anyhow::Result<()> {
    call_docker_compose_cmd(docker_compose_file_dir, "stop")
}
pub fn docker_compose_stop_silent(docker_compose_file_dir: &Path) {
    call_docker_compose_cmd_silent(docker_compose_file_dir, "stop")
}


pub fn docker_compose_stop_except(docker_compose_file_dir: &Path, ignore_services: &[&str])
    -> anyhow::Result<()> {

    let containers = docker_compose_ps(docker_compose_file_dir) ?;
    let services = containers.into_iter().map(|c|c.service).collect::<Vec<_>>();

    let services_to_stop = services.into_iter()
        .filter(|s| !ignore_services.contains(&s.as_str()))
        .collect::<Vec<_>>();
    let services_to_stop = services_to_stop.iter().map(|s|s.as_str()).collect::<Vec<_>>();

    call_docker_compose_cmd_2(docker_compose_file_dir, "stop", services_to_stop.as_slice())
}


pub fn docker_compose_start_except(docker_compose_file_dir: &Path, ignore_services: &[&str]) -> anyhow::Result<()> {

    let containers = docker_compose_ps(docker_compose_file_dir) ?;
    let services = containers.into_iter().map(|c|c.service).collect::<Vec<_>>();

    let services_to_stop = services.into_iter()
        .filter(|s| !ignore_services.contains(&s.as_str()))
        .collect::<Vec<_>>();
    let services_to_stop = services_to_stop.iter().map(|s|s.as_str()).collect::<Vec<_>>();

    call_docker_compose_cmd_2(docker_compose_file_dir, "start", services_to_stop.as_slice())
}



pub fn get_docker_compose_file(docker_compose_file_dir: &Path) -> anyhow::Result<PathBuf> {
    let new_docker_compose_file_1 = docker_compose_file_dir.join("docker-compose.yml");
    let new_docker_compose_file_2 = docker_compose_file_dir.join("docker-compose.yaml");
    let docker_compose_file = [new_docker_compose_file_1, new_docker_compose_file_2]
        .into_iter().find(|f|f.exists())
        .ok_or_else(||anyhow!("No compose-file in [{docker_compose_file_dir:?}]")) ?;
    Ok(docker_compose_file)
}


#[derive(Debug, Copy, Clone)]
#[derive(strum::Display, strum::IntoStaticStr, strum::EnumIter)]
pub enum DockerImageProfile {
    Release,
    Debug,
    DebugLocal,
}
impl DockerImageProfile {
    pub fn as_docker_tag_suffix(&self) -> &'static str {
        match self {
            DockerImageProfile::Release => "",
            DockerImageProfile::Debug => "-debug",
            DockerImageProfile::DebugLocal => "-debug-local",
        }
    }
}

pub fn get_docker_image_profile() -> DockerImageProfile {

    let profile_suffix_from_env = crate::env::env_var_static("DOCKER_IMAGE_PROFILE_SUFFIX")
        .unwrap_or_else(|_|Some(String::new())).unwrap_or_else(||String::new());

    // If it is already directly set in makefile.toml, we just inherit it.
    if !profile_suffix_from_env.is_empty() {
        use strum::IntoEnumIterator;

        let image_profile_from_env = DockerImageProfile::iter()
            .find(|el|{
                let s: &'static str = el.as_docker_tag_suffix();
                s == profile_suffix_from_env
            });

        if let Some(image_profile) = image_profile_from_env {
            return image_profile;
        }
    }

    let docker_image_profile: DockerImageProfile =
        if cfg!(debug_assertions) {
            if is_CI_build() {
                // Is it possible, CI 'debug' build?
                DockerImageProfile::Debug
            } else if is_manually_launched_task() {
                DockerImageProfile::DebugLocal
            } else {
                // At that moment I do not see sense to use 'pure' 2-phase rust docker build with 'debug'
                // * Such build is time expensive/consuming (similar to 'release')
                // * Requires additional disk usage
                //
                // DockerImageProfile::Debug

                DockerImageProfile::DebugLocal
            }
        } else {
            DockerImageProfile::Release // prod/release mode
        };

    docker_image_profile
}


pub fn preload_docker_compose_images(docker_compose_file: &Path) -> anyhow::Result<()> {
    let images = get_docker_compose_file_images(docker_compose_file) ?;

    for image in images {
        if !is_docker_image_present(&image) ? {
            pull_docker_image(&image) ?;
        }
    }

    Ok(())
}

fn is_docker_image_present(image: &str) -> anyhow::Result<bool> {
    let mut cmd = Command::new("docker");
    cmd.args(["image", "inspect", image].into_iter())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let status = cmd.status() ?;
    Ok(status.success())
}

fn pull_docker_image(image: &str) -> anyhow::Result<()> {
    let mut cmd = Command::new("docker");
    cmd.args(["image", "pull", image].into_iter())
        // I think it makes sense to show that image was loaded
        // .stdout(Stdio::null())
        // .stderr(Stdio::null())
        ;

    let status = cmd.status() ?;

    if status.success() { Ok(()) }
    else { anyhow::bail!("'docker pull {image}' failed (exit code: {status:?})") }
}

fn get_docker_compose_file_images(docker_compose_file: &Path) -> anyhow::Result<Vec<String>> {
    let yaml_docs = load_yaml(docker_compose_file) ?;

    let images = yaml_docs.into_iter()
        .flat_map(|yaml| get_yaml_images(&yaml))
        .collect::<Vec<String>>();

    Ok(images)
}

fn get_yaml_images(yaml: &Yaml) -> Vec<String> {

    let services = &yaml["services"];
    let mut images = Vec::<String>::new();

    match services {
        Yaml::Hash(ref services) => {
            for (ref _serv_name, ref serv_doc) in services {
                match &serv_doc["image"] {
                    Yaml::String(image) => {
                        images.push(image.to_string());
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    };

    images
}


pub fn docker_compose_ps_ids(docker_compose_file_dir: &Path) -> anyhow::Result<Vec<String>> {

    let docker_compose_file = get_docker_compose_file(docker_compose_file_dir) ?;
    if !docker_compose_file.exists() {
        anyhow::bail!("No file [{docker_compose_file:?}].")
    }

    let out = Command::new("docker")
        .current_dir(docker_compose_file_dir.to_path_buf())
        // '--quiet' is used to get only IDs. Strange param name...
        .args(["compose", "ps", "--quiet"].into_iter())
        .output() ?;

    if !out.status.success() {
        anyhow::bail!("'docker compose ps --quiet' failed with exit status {:?}.", out.status);
    }

    let out_str = String::from_utf8_lossy(out.stdout.as_bytes()).to_string();

    let ids = out_str.split('\n')
        .map(|id|id.trim().to_owned())
        .collect::<Vec<_>>();

    Ok(ids)
}


/*
[
    {
        "ID":"3f94b934006aa0a10f955fecbae5189ca80dd15d212cced91582723fe8cc87a2",
        "Name":"rust-account-soa-project-database-1",
        "Command":"docker-entrypoint.sh sh -c ' cp /certs/database.key.pem /usr/lib/ssl/database-test.key.pem && chown postgres /usr/lib/ssl/database-test.key.pem && chmod go-r /usr/lib/ssl/database-test.key.pem && docker-entrypoint.sh postgres -l -c ssl=on -c ssl_cert_file=/certs/database.crt.pem -c ssl_key_file=/usr/lib/ssl/database-test.key.pem '",
        "Project":"rust-account-soa-project",
        "Service":"database",
        "State":"running",
        "Health":"healthy",
        "ExitCode":0,
        "Publishers":[{"URL":"0.0.0.0","TargetPort":5432,"PublishedPort":5432,"Protocol":"tcp"},{"URL":"::","TargetPort":5432,"PublishedPort":5432,"Protocol":"tcp"}]
    },
    {
        "ID":"0f7408d20e9bd6bce0087b080bf827ea806ef44b823fe843b056aa315044eddc",
        "Name":"rust-account-soa-project-rust-account-soa-1",
        "Command":"/bin/mvv_account_soa arg1 arg2","Project":"rust-account-soa-project",
        "Service":"rust-account-soa",
        "State":"running",
        "Health":"healthy",
        "ExitCode":0,
        "Publishers":[{"URL":"","TargetPort":80,"PublishedPort":0,"Protocol":"tcp"},{"URL":"","TargetPort":443,"PublishedPort":0,"Protocol":"tcp"},{"URL":"","TargetPort":2735,"PublishedPort":0,"Protocol":"tcp"},{"URL":"","TargetPort":8000,"PublishedPort":0,"Protocol":"tcp"},{"URL":"","TargetPort":8080,"PublishedPort":0,"Protocol":"tcp"},{"URL":"0.0.0.0","TargetPort":8443,"PublishedPort":8101,"Protocol":"tcp"},{"URL":"::","TargetPort":8443,"PublishedPort":8101,"Protocol":"tcp"}]
    }
]
 */
#[allow(dead_code)] // for fields
#[derive(Debug, serde::Deserialize)]
pub struct ContainerInfo {
    #[serde(alias="ID")]
    pub id: String,
    #[serde(alias="Name")]
    pub name: String,
    #[serde(alias="Command")]
    pub command: String,
    #[serde(alias="Service")]
    pub service: String,
    #[serde(alias="State")]
    pub state: String,
    #[serde(alias="Health")]
    pub health: String,
    // ... others
}


pub fn docker_compose_ps(docker_compose_file_dir: &Path) -> anyhow::Result<Vec<ContainerInfo>> {

    let docker_compose_file = get_docker_compose_file(docker_compose_file_dir) ?;
    if !docker_compose_file.exists() {
        anyhow::bail!("No file [{docker_compose_file:?}].")
    }

    // !!! Very important to use docker-compose instead of 'docker compose'. !!!
    // docker-compose returns correct JSON array, but 'docker compose' returns
    // separate lines with JSON in every line, and seems with some (probably) truncated fields.
    // let out = Command::new("docker-compose")
    //     .current_dir(docker_compose_file_dir.to_path_buf())
    //     .args(["ps", "--all", "--format", "json"].into_iter())
    //     .output() ?;

    let out = Command::new("docker")
        .current_dir(docker_compose_file_dir.to_path_buf())
        .args(["compose", "ps", "--all", "--format", "json"].into_iter())
        .output() ?;

    if !out.status.success() {
        anyhow::bail!("'docker compose ps --all --format json' failed with exit status {:?}.", out.status);
    }

    let out_str = String::from_utf8_lossy(out.stdout.as_bytes()).to_string();

    debug!("### out_str: {out_str}");

    // in case of 'docker-compose'
    // let details: Vec<ContainerInfo> = serde_json::from_str(&out_str) ?;

    // in case of 'docker compose'
    let details: Vec<ContainerInfo> = out_str.split('\n')//.into_iter()
        .filter(|s|!s.is_empty())
        .map(|s| serde_json::from_str(s))
        .collect::<Result<Vec<_>, _>>() ?;

    Ok(details)
}

// I do not see sense to do it async (now)... but probably will change it later (just for fun)
pub fn wait_for_healthy(docker_compose_file_dir: &Path, timeout: Duration) -> anyhow::Result<()> {
    wait_for_healthy_except(docker_compose_file_dir, &[], timeout)
}

// I do not see sense to do it async (now)... but probably will change it later (just for fun)
// (just replace sync sleep with async sleep)
pub fn wait_for_healthy_except(docker_compose_file_dir: &Path, ignore_services: &[&str], timeout: Duration) -> anyhow::Result<()> {
    let started = Instant::now();
    loop {
        let details = docker_compose_ps(docker_compose_file_dir);
        let details = details.map(|details| details.into_iter()
            .filter(|el| !ignore_services.contains(&el.service.as_str()))
            .collect::<Vec<_>>());

        match details {
            Ok(details) => {
                let (_healthy, unhealthy): (Vec<ContainerInfo>, Vec<ContainerInfo>) =
                    details.into_iter()
                        .partition_map(|c|
                            if c.health == "healthy" || c.health == "started" {
                                Either::Left(c)
                            } else {
                                Either::Right(c)
                            }
                        );

                let elapsed = Instant::now() - started;

                if unhealthy.is_empty() {
                    info!("Wait for healthy took {elapsed:?}.");
                    return Ok(());
                } else {
                    let unhealthy = unhealthy.iter()
                        .map(|c| format!("{}: {}", c.service, c.health))
                        .join(", ");
                    debug!("### unhealthy: {unhealthy:?}");

                    if elapsed > timeout {
                        anyhow::bail!("Some containers are still unhealthy [{unhealthy}]");
                    }
                }
            }
            Err(err) => {
                debug!("wait_for_healthy error: {err:?}");

                let elapsed = Instant::now() - started;
                if elapsed > timeout {
                    anyhow::bail!("Wait for healthy error ({err:?})");
                }
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}
