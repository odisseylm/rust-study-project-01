use std::ffi::OsString;
use std::fmt::Debug;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::time::Duration;
use anyhow::anyhow;
use itertools::Itertools;
use log::{error, info, warn};
use yaml_rust2::{Yaml, YamlEmitter, YamlLoader};
use crate::io::find_existent_buf;
use crate::string::str_vec;
use crate::test::docker_compose::{
    docker_compose_down, docker_compose_down_silent, get_docker_compose,
};
use crate::test::docker_compose_util::{
    change_network, gather_host_volumes_src, remove_host_ports, set_docker_image_profile_suffix_var,
};
use crate::test::files::{do_copy, do_replacements, Copy, CopyCfg, Replace};
use super::{ change_name_by_policy, current_sub_project_dir, find_target_dir};
//--------------------------------------------------------------------------------------------------



pub async fn wait_containers<C>(
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


#[derive(Clone, Debug)]
pub enum NamePolicy {
    Original,
    Custom(String),
    WithSuffix(String),
    WithRandomSuffix,
    WithTestSessionIdSuffix,
    WithBuildIdSuffix,
    WithStringAndBuildIdSuffix(String),
}

#[derive(Debug)]
pub struct PrepareDockerComposeCfg {
    pub tests_session_id: i64,

    pub copy: CopyCfg,
    pub replace: Vec<Replace>,

    /// 'docker' by default.
    pub docker_files_dir: PathBuf,

    /// Docker (possible) files to automatically copy and process/prepare them for using with
    /// integration tests.
    ///
    /// By default, it is docker-compose.yaml/docker-compose.yml, docker-compose.env.
    /// Yiy can customize it.
    pub docker_files: Vec<String>,

    /// It is 'base' docker-compose project directory.
    /// Docker-compose uses directory of docker-compose.yml file as base/project directory,
    /// or it can be specified by '--project-directory' param (preferable for me way).
    ///
    /// I prefer to use 'docker' sub-dir for docker files
    /// and project (subproject) dir as project/base directory.
    pub docker_compose_project_dir: PathBuf,

    /// Used as last/leaf sub-dir name to have good logging in rustainers
    /// (rustainers crate does not support docker-compose '--project-name' now).
    /// If it is missed project directory name will be used (directory with Cargo.toml).
    pub docker_compose_project_name: Option<String>,

    /// Changed/custom/unique docker-compose network/project names are used
    /// to avoid conflicts between integration test docker-compose session
    /// and debug/dev docker-compose session (or another integration test docker-compose session on
    /// CI server)
    pub docker_compose_project_name_policy: NamePolicy,

    /// Changed/custom/unique docker-compose network/project names are used
    /// to avoid conflicts between integration test docker-compose session
    /// and debug/dev docker-compose session (or another integration test docker-compose session on
    /// CI server)
    pub docker_compose_network_name: NamePolicy,

    /// I use env var ${DOCKER_IMAGE_PROFILE_SUFFIX} inside docker-compose.yml file
    /// to distinguish
    /// * 'prod' image (no suffix)
    /// * debug (suffix '-debug') (now not used to save time and disk space)
    /// * debug local (suffix '-debug-local'), cheap build just with putting debug exe file to image
    pub replace_docker_image_profile_suffix: bool,
}


impl Default for PrepareDockerComposeCfg {
    fn default() -> Self {
        let sub_project_dir_res = current_sub_project_dir();
        // let sub_project_dir = sub_project_dir.unwrap_or_else(|err| {
        //     warn!("current_sub_project_dir is not found. Probably problem with CARGO_MANIFEST_DIR env var: {err:?}");
        //     PathBuf::new()
        // });

        let sub_project_dir: PathBuf;
        let docker_compose_project_dir: PathBuf;
        let docker_files_dir: PathBuf;
        let docker_compose_project_name: Option<String>;

        match sub_project_dir_res {
            Ok(sub_project_d) => {
                sub_project_dir = sub_project_d;
                docker_compose_project_dir = sub_project_dir.to_path_buf();

                // my preferable approach to avoid a lot of docker files in project root dir
                let docker_dir = sub_project_dir.join("docker");

                docker_files_dir =
                    if get_docker_compose(&docker_dir).is_ok() {
                        docker_dir
                    } else if get_docker_compose(&sub_project_dir).is_ok() {
                        sub_project_dir.to_path_buf()
                    } else {
                        warn!("docker compose directory is not found");
                        "docker".into()
                    };

                docker_compose_project_name =
                    match sub_project_dir.file_name() {
                        None =>
                            None,
                        Some(filename) =>
                            Some(filename.to_string_lossy().to_lowercase()),
                    };
            }
            Err(err) => {
                warn!("current_sub_project_dir is not found. Probably problem with CARGO_MANIFEST_DIR env var: {err:?}");

                sub_project_dir = PathBuf::new();
                docker_compose_project_dir = PathBuf::new();
                docker_files_dir = "docker".into();
                docker_compose_project_name = None;
            }
        }

        Self {
            tests_session_id: chrono::Local::now().timestamp(),
            copy: CopyCfg {
                base_from_dir: sub_project_dir,
                copy: Vec::new(),
            },
            replace: Vec::new(),
            docker_files_dir,
            // possible files, if they are not present, they will be silently skipped
            docker_files: str_vec(["docker-compose.env", "docker-compose.yml", "docker-compose.yaml"]),
            docker_compose_project_dir,
            docker_compose_project_name,
            docker_compose_project_name_policy: NamePolicy::WithStringAndBuildIdSuffix("-it_tests".to_owned()),
            docker_compose_network_name: NamePolicy::WithStringAndBuildIdSuffix("-it_tests".to_owned()),
            replace_docker_image_profile_suffix: true,
        }
    }
}


pub fn prepare_docker_compose(sub_project_dir: &Path, cfg: &PrepareDockerComposeCfg)
    -> Result<PathBuf, anyhow::Error> {

    // let vars =  std::env::vars().map(|(k,v)|format!("{k:?} = {v:?}")).sorted().join("\n");
    // println!("vars: {vars}");

    let target_dir_ = find_target_dir(sub_project_dir) ?;
    let target_dir_: &Path = &target_dir_;
    let test_res_dir = target_dir_.join("temp/docker_compose_tests");

    let tests_session_id = cfg.tests_session_id;
    let test_res_dir = test_res_dir.join(&format!("root-{tests_session_id}"));

    let sub_project_dir = current_sub_project_dir() ?;
    let sub_project_dir = sub_project_dir.as_path();

    // Since 'rustainers' does not support setting docker compose 'project_name',
    // we have to use (last) directory name as unique project name
    // See 'Specify a project name' https://docs.docker.com/compose/project-name/
    //

    let base_name = match cfg.docker_compose_project_name {
        None => {
            sub_project_dir.file_name()
                .ok_or_else(||anyhow!("No filename for [{sub_project_dir:?}]")) ?
                .to_string_lossy().to_string()
        }
        Some(ref base_name) =>
            base_name.to_string(),
    };

    // Project names must contain only lowercase letters, decimal digits, dashes,
    // and underscores, and must begin with a lowercase letter or decimal digit.
    // See https://github.com/compose-spec/compose-spec/blob/main/spec.md
    //
    let base_name = base_name.to_lowercase();
    let name = change_name_by_policy(&base_name, &cfg.docker_compose_project_name_policy, tests_session_id) ?;

    let last_leaf_dir_name = format!("{}-{tests_session_id}", name);
    let test_res_dir = test_res_dir.join(&last_leaf_dir_name);

    std::fs::create_dir_all(&test_res_dir) ?;

    let standard_docker_files_to_copy = cfg.docker_files.iter()
        .filter_map(|f|{
            let from = find_existent_buf([
                cfg.docker_files_dir.join(&f), sub_project_dir.join(&f), cfg.copy.base_from_dir.join(&f),
            ]);
            from.map(|from|Copy { from: from.to_path_buf(), to: f.into() })
        })
        .collect::<Vec<Copy>>();

    do_copy(&standard_docker_files_to_copy, &cfg.copy.base_from_dir, &test_res_dir) ?;

    do_copy(&cfg.copy.copy, &cfg.copy.base_from_dir, &test_res_dir) ?;

    do_replacements(&cfg.replace, &test_res_dir) ?;

    let new_docker_compose_file = get_docker_compose(&test_res_dir) ?;

    if cfg.replace_docker_image_profile_suffix {
        set_docker_image_profile_suffix_var(&new_docker_compose_file, &test_res_dir) ?;
    }

    copy_volume_src_data(&new_docker_compose_file, &cfg.docker_compose_project_dir, &test_res_dir) ?;

    remove_host_ports(&new_docker_compose_file) ?;
    change_network(&new_docker_compose_file, &cfg.docker_compose_network_name, tests_session_id) ?;

    Ok(test_res_dir.to_path_buf())
}



fn copy_volume_src_data(docker_compose_file: &Path, docker_compose_project_dir: &Path, test_target_dir: &Path) -> anyhow::Result<()> {
    let host_volumes_src = gather_host_volumes_src(docker_compose_file) ?;

    let as_copy_params = host_volumes_src.iter()
        .map(|src|{
            Copy {
                from: src.into(),
                to: src.into(),
            }
        })
        .collect::<Vec<Copy>>();

    info!("copy_volume_src_data => docker-compose project dir: {docker_compose_project_dir:?}");
    do_copy(&as_copy_params, docker_compose_project_dir, test_target_dir) ?;

    Ok(())
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


pub fn add_yaml_to_string(yaml: &Yaml, out_str: &mut String) -> anyhow::Result<()> {
    {
        let mut emitter = YamlEmitter::new(out_str);
        emitter.dump(yaml) ?; // dump the YAML object to a String
    } // in {} block according to official example
    Ok(())
}


pub fn yaml_to_string(yaml: &Yaml) -> anyhow::Result<String> {
    let mut out_str = String::new();
    add_yaml_to_string(yaml, &mut out_str) ?;
    Ok(out_str)
}


pub fn save_yaml(yaml_docs: &Vec<Yaml>, to_file: &Path) -> anyhow::Result<()> {
    let mut out_str = String::new();
    for ref yaml in yaml_docs {
        add_yaml_to_string(yaml, &mut out_str) ?;
        out_str.push('\n');
    }

    out_str.push_str("\n\n");
    let _ = std::fs::write(to_file, out_str) ?;
    Ok(())
}


pub fn load_yaml(yaml_file: &Path) -> anyhow::Result<Vec<Yaml>> {

    let yaml_str = std::fs::read_to_string(yaml_file)
        .map_err(|err| anyhow!("Error of opening [{yaml_file:?}] ({err:?})")) ?;

    // Multi document support, doc is a yaml::Yaml
    let yaml_docs = YamlLoader::load_from_str(&yaml_str) ?;
    Ok(yaml_docs)
}
