use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::anyhow;
use log::warn;
use yaml_rust2::Yaml;
use crate::test::{
    is_CI_build, is_manually_launched_task,
    integration::load_yaml,
};
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


pub fn load_images(docker_compose_file: &Path) -> anyhow::Result<()> {
    let images = get_images(docker_compose_file) ?;

    for image in images {
        if !is_image_present(&image) ? {
            pull_image(&image) ?;
        }
    }

    Ok(())
}

fn is_image_present(image: &str) -> anyhow::Result<bool> {
    let mut cmd = Command::new("docker");
    cmd.args(["image", "inspect", image].into_iter())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let status = cmd.status() ?;
    Ok(status.success())
}

fn pull_image(image: &str) -> anyhow::Result<()> {
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

fn get_images(docker_compose_file: &Path) -> anyhow::Result<Vec<String>> {
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
