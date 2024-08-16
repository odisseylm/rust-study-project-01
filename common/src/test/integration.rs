use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::anyhow;
use itertools::Itertools;
use log::{info, warn};
//--------------------------------------------------------------------------------------------------


#[derive(Debug)]
pub struct Copy {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Debug)]
pub struct Replace {
    pub file: PathBuf,
    pub from: Vec<String>,
    pub to: Vec<String>,
}
impl Replace {
    pub fn by_str<const N: usize, const M: usize>(file: PathBuf, from: [&str;N], to: [&str;M]) -> Self {
        Replace {
            file,
            from: from.into_iter().map(|s|s.to_owned()).collect::<Vec<_>>(),
            to: to.into_iter().map(|s|s.to_owned()).collect::<Vec<_>>(),
        }
    }
}

#[derive(Debug)]
pub struct PrepareDockerComposeCfg {
    pub tests_session_id: i64,
    pub name: String, // used last sub-dir name (to have good logging in rustainers)
    pub base_from_dir: PathBuf,
    pub copy: Vec<Copy>,
    pub replace_file_content: Vec<Replace>,
}

/*
fn is_dir_target(path: &Path) -> bool {
    let as_str = path.to_string_lossy();
    match as_str {
        Cow::Borrowed(as_str) => as_str.ends_with("/"),
        Cow::Owned(as_str) => as_str.ends_with("/"),
    }
}
*/

pub fn prepare_docker_compose(project_dir: &Path, cfg: &PrepareDockerComposeCfg)
    -> Result<PathBuf, anyhow::Error> {

    let target_dir = project_dir.join("target/temp/docker_compose_tests");
    let tests_session_id = cfg.tests_session_id;

    // Since 'rustainers' does not support setting docker compose 'project_name',
    // we have to use (last) directory name as unique project name
    // See 'Specify a project name' https://docs.docker.com/compose/project-name/
    //
    let root_to_dir = format!("{}-{tests_session_id}", cfg.name);
    let root_to_dir = target_dir.join(&root_to_dir);

    std::fs::create_dir_all(&root_to_dir) ?;

    for copy in cfg.copy.iter() {
        let Copy { from, to } = copy;

        let from_orig = from.clone();
        let from: PathBuf =
            if from.is_absolute() && from.exists() { from.clone() }
            else { cfg.base_from_dir.join(from) };

        if !from.exists() {
            anyhow::bail!("Path [{from:?}] does not exist.")
        }

        let is_dir_copying = from.is_dir();

        if !is_dir_copying && to.is_absolute() && to.exists() {
            anyhow::bail!("Path [{from:?}] already exists.")
        }

        let is_empty_to = to.as_os_str().is_empty() || (to.as_os_str() == OsString::from("."));

        let to =
            if is_empty_to {
                if is_dir_copying {
                    root_to_dir.clone()
                } else {
                    if from_orig.is_absolute() {
                        root_to_dir.join(
                            from.file_name().ok_or_else(|| anyhow!("Now filename of [{from:?}]")) ?)
                    } else {
                        root_to_dir.join(&from_orig)
                    }
                }
            } else {
                root_to_dir.join(&to)
            };

        std::fs::create_dir_all(
            &to.parent().ok_or_else(||anyhow!("No parent in [{to:?}].")) ?
        ) ?;

        if is_dir_copying {
            fs_extra::copy_items(&[&from], &to, &fs_extra::dir::CopyOptions {
                copy_inside: true,
                depth: 16,
                .. Default::default()
            }) ?;
        } else {
            fs_extra::file::copy(&from, &to, &fs_extra::file::CopyOptions::default()) ?;
        }
    }

    for replace in cfg.replace_file_content.iter() {
        let Replace { file, from, to } = replace;

        let file =
            if file.is_absolute() { file.clone() }
            else { root_to_dir.join(file) };

        let mut text = std::fs::read_to_string(&file) ?;

        for from in from.iter() {
            for to in to {
                text = text.replace(from, &to);
            }
        }

        std::fs::write(file, &text) ?;
    }

    Ok(root_to_dir.to_path_buf())
}


pub fn docker_compose_down(docker_compose_file_dir: &Path) -> anyhow::Result<()> {

    let docker_compose_file = docker_compose_file_dir.join("docker-compose.yml");
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

pub struct AutoDockerComposeDown {
    pub docker_compose_file_dir: PathBuf,
    pub log_message: Option<&'static str>,
}
impl Drop for AutoDockerComposeDown {
    fn drop(&mut self) {
        if let Some(log_message) = self.log_message {
            info!("{}", log_message);
        }
        docker_compose_down_silent(&self.docker_compose_file_dir)
    }
}


pub fn is_integration_tests_enabled() -> bool {

    let is_it_1 = std::env::var("INTEGRATION_TEST").is_ok();
    let is_it_2 = std::env::var("INTEGRATION_TESTS").is_ok();
    let is_exact = std::env::args_os().contains(&OsString::from("--exact"));

    let test_enabled = is_it_1 || is_it_2 || is_exact;

    test_enabled
}
