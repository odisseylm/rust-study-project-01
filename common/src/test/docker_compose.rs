use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::anyhow;
use log::warn;
use crate::test::{is_CI_build, is_manually_launched_task};
//--------------------------------------------------------------------------------------------------



pub fn docker_compose_down(docker_compose_file_dir: &Path) -> anyhow::Result<()> {

    let docker_compose_file = get_docker_compose(docker_compose_file_dir) ?;
    if !docker_compose_file.exists() {
        anyhow::bail!("No file [{docker_compose_file:?}].")
    }

    Command::new("docker")
        .current_dir(docker_compose_file_dir.to_path_buf())
        .arg("compose")
        .arg("down")
        .status() ?;
    Ok(())
}


pub fn docker_compose_down_silent(docker_compose_file_dir: &Path) {
    let res = docker_compose_down(docker_compose_file_dir);
    if let Err(err) = res {
        warn!("Docker compose down failed => {err:?}")
    }
}


pub fn get_docker_compose(docker_compose_file_dir: &Path) -> anyhow::Result<PathBuf> {
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

    let profile_suffix_from_env = crate::env::env_var("DOCKER_IMAGE_PROFILE_SUFFIX")
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
