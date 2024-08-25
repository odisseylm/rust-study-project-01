use std::collections::HashSet;
use std::ffi::OsString;
use std::fmt::Debug;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::time::Duration;
use anyhow::anyhow;
use itertools::Itertools;
use log::{error, info, warn};
use yaml_rust2::{Yaml, YamlEmitter, YamlLoader};
use crate::string::str_vec;
use super::{is_CI_build, is_manually_launched_task, TestResultUnwrap };
//--------------------------------------------------------------------------------------------------



pub async fn wait_containers<C>(
    res_docker_compose_dir: PathBuf,
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
                    info!("SUCCESS of Account SOA launch => {res:?}");
                    Ok( (res_docker_compose_dir, res) )
                },
                Err(err) => {
                    error!("FAILURE of Account SOA launch => Error {{ {err:?} }}");
                    let _ = docker_compose_down(&res_docker_compose_dir);
                    Err(anyhow!(err))
                },
            }
        },
        Err(err) => {
            error!("FAILURE of Account SOA launch => Error {{ {err:?} }}");
            info!("Shut down docker compose manually...");
            let _ = docker_compose_down(&res_docker_compose_dir);
            Err(anyhow!(err))
        },
    }
}


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


#[derive(Clone, Debug)]
pub enum NamePolicy {
    Original,
    Custom(String),
    WithSuffix(String),
    WithRandomSuffix,
    WithTestSessionIdSuffix,
    WithBuildIdSuffix,
}

#[derive(Debug)]
pub struct PrepareDockerComposeCfg {
    pub tests_session_id: i64,

    pub base_from_dir: PathBuf, // TODO: redesign/rename/doc about it
    pub copy: Vec<Copy>,
    pub replace_file_content: Vec<Replace>,

    pub docker_files_dir: PathBuf,
    pub docker_files: Vec<String>,
    // It can be directory of docker-compose.yml or your subproject dir, or root workspace dir.
    // I prefer to use subproject dir.
    pub docker_compose_project_dir: PathBuf,

    // Used last sub-dir name (to have good logging in rustainers).
    // If it is missed project directory name will be used (directory with Cargo.toml).
    pub docker_compose_project_name: Option<String>, // used last sub-dir name (to have good logging in rustainers)
    pub docker_compose_project_name_policy: NamePolicy,
    pub docker_compose_network_name: NamePolicy,
    pub replace_docker_image_profile_suffix: bool,
}


pub fn current_sub_project_dir() -> anyhow::Result<PathBuf> {
    let sub_project_dir = crate::env::required_env_var("CARGO_MANIFEST_DIR") ?;
    let sub_project_dir: PathBuf = sub_project_dir.into();
    Ok(sub_project_dir)
}

pub fn current_project_target_dir() -> anyhow::Result<PathBuf> {
    let out_dir_str = crate::env::required_env_var("OUT_DIR") ?;
    let out_dir: PathBuf = out_dir_str.into();

    let target_dir = find_nearest_target_dir(&out_dir) ?;
    Ok(target_dir)
}

pub fn current_root_project_dir() -> anyhow::Result<PathBuf> {
    let target_dir = current_project_target_dir() ?;
    let root_project_dir = target_dir.parent().ok_or_else(||anyhow!("No parent directory if {target_dir:?}")) ?;
    Ok(root_project_dir.to_path_buf())
}

pub fn build_id() -> anyhow::Result<i64> {
    let target_dir = current_project_target_dir() ?;
    let build_id_file = target_dir.join("buildId");

    let build_id = if build_id_file.exists() {
        let str_build_id = std::fs::read_to_string(&build_id_file) ?;
        let str_build_id = str_build_id.trim();

        let build_id: i64 = core::str::FromStr::from_str(str_build_id) ?;
        build_id
    } else {
        let build_id: i64 = chrono::Local::now().timestamp();
        std::fs::write(&build_id_file, format!("{build_id}")) ?;
        build_id
    };
    Ok(build_id)
}

impl Default for PrepareDockerComposeCfg {
    fn default() -> Self {
        let sub_project_dir = current_sub_project_dir();
        let sub_project_dir = sub_project_dir.unwrap_or_else(|err| {
            warn!("current_sub_project_dir is not found. Probably problem with CARGO_MANIFEST_DIR env var: {err:?}");
            PathBuf::new()
        });

        let docker_compose_project_dir = sub_project_dir.to_path_buf();

        Self {
            tests_session_id: chrono::Local::now().timestamp(),
            base_from_dir: sub_project_dir,
            copy: Vec::new(),
            replace_file_content: Vec::new(),
            docker_files_dir: "docker".into(),
            docker_files: str_vec(["docker-compose.env", "docker-compose.yml", "docker-compose.yaml"]),
            docker_compose_project_dir,
            docker_compose_project_name: None,
            docker_compose_project_name_policy: NamePolicy::WithTestSessionIdSuffix,
            docker_compose_network_name: NamePolicy::WithBuildIdSuffix, // WithSuffix("_it_tests".to_owned()),
            replace_docker_image_profile_suffix: true,
        }
    }
}


pub fn prepare_docker_compose(sub_project_dir: &Path, cfg: &PrepareDockerComposeCfg)
    -> Result<PathBuf, anyhow::Error> {

    // let vars =  std::env::vars().map(|(k,v)|format!("{k:?} = {v:?}")).sorted().join("\n");
    // println!("vars: {vars}");

    let target_dir_ = find_nearest_target_dir(sub_project_dir) ?;
    let target_dir_: &Path = &target_dir_;
    let test_res_dir = target_dir_.join("temp/docker_compose_tests");

    let tests_session_id = cfg.tests_session_id;
    let test_res_dir = test_res_dir.join(&format!("root-{tests_session_id}"));

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
            let from = cfg.docker_files_dir.join(&f);
            let from_full_path = if from.is_absolute() { from.clone() }
                                 else { cfg.base_from_dir.join(&from) };

            if from_full_path.exists() { Some(Copy { from, to: f.into() }) }
            else { None }
        })
        .collect::<Vec<Copy>>();

    copy_to_test_target_dir(&standard_docker_files_to_copy, &cfg.base_from_dir, &test_res_dir) ?;

    copy_to_test_target_dir(&cfg.copy, &cfg.base_from_dir, &test_res_dir) ?;

    do_replace(&cfg.replace_file_content, &test_res_dir) ?;

    let new_docker_compose_file = get_docker_compose(&test_res_dir) ?;

    if cfg.replace_docker_image_profile_suffix {
        set_docker_image_profile_suffix_var(&new_docker_compose_file, &test_res_dir) ?;
    }

    copy_volume_src_data(&new_docker_compose_file, &cfg.docker_compose_project_dir, &test_res_dir) ?;

    fix_ports(&new_docker_compose_file) ?;
    change_network(&new_docker_compose_file, &cfg.docker_compose_network_name, tests_session_id) ?;

    Ok(test_res_dir.to_path_buf())
}


fn copy_to_test_target_dir(copy: &Vec<Copy>, base_from_dir: &Path, root_to_dir: &Path) -> anyhow::Result<()> {
    for copy in copy.iter() {
        let Copy { from, to } = copy;

        let from_orig = from.clone();
        let from: PathBuf =
            if from.is_absolute() && from.exists() { from.clone() }
            else { base_from_dir.join(from) };

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
                    root_to_dir.to_path_buf()
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

    Ok(())
}

fn do_replace(replace_file_content: &Vec<Replace>, test_res_dir: &Path) -> anyhow::Result<()> {
    for replace in replace_file_content.iter() {
        let Replace { file, from, to } = replace;

        let file =
            if file.is_absolute() { file.clone() }
            else { test_res_dir.join(file) };

        let mut text = std::fs::read_to_string(&file) ?;

        for from in from.iter() {
            for to in to {
                text = text.replace(from, &to);
            }
        }

        std::fs::write(file, &text) ?;
    }

    Ok(())
}

fn set_docker_image_profile_suffix_var(new_docker_compose_file: &Path, test_res_dir: &Path) -> anyhow::Result<()> {
    let docker_image_profile = get_docker_image_profile();
    let docker_image_profile_suffix: &str = docker_image_profile.as_docker_tag_suffix();

    let r = vec!(Replace::by_str(
        new_docker_compose_file.to_path_buf(),
        ["${DOCKER_IMAGE_PROFILE_SUFFIX}"], [docker_image_profile_suffix]));
    do_replace(&r, &test_res_dir) ?;

    // just in case lets put it to env vars too
    std::env::set_var("DOCKER_IMAGE_PROFILE_SUFFIX", docker_image_profile_suffix);

    Ok(())
}


fn get_docker_compose(docker_compose_file_dir: &Path) -> anyhow::Result<PathBuf> {
    let new_docker_compose_file_1 = docker_compose_file_dir.join("docker-compose.yml");
    let new_docker_compose_file_2 = docker_compose_file_dir.join("docker-compose.yaml");
    let docker_compose_file = [new_docker_compose_file_1, new_docker_compose_file_2]
        .into_iter().find(|f|f.exists())
        .ok_or_else(||anyhow!("No compose-file in [{docker_compose_file_dir:?}]")) ?;
    Ok(docker_compose_file)
}


fn find_nearest_target_dir(dir: &Path) -> anyhow::Result<PathBuf> {

    let orig_dir = dir;
    let dir = dir.canonicalize().map_err(|_err|anyhow!("Seems no dir [{dir:?}].")) ?;
    let mut dir = dir.as_path();
    let mut iter_count = 0;

    loop {
        let target_dir = dir.join("target");
        if target_dir.exists() {
            return Ok(target_dir);
        }

        let parent_dir = dir.parent();
        match parent_dir {
            None =>
                anyhow::bail!("'target' dir for [{orig_dir:?}] is not found."),
            Some(parent_dir) => {
                dir = parent_dir;
            }
        }

        iter_count += 1;
        if iter_count > 20 {
            anyhow::bail!("Too many recursion in finding 'target' dir for [{orig_dir:?}]")
        }
    }
}

pub fn fix_ports(docker_compose_file: &Path) -> anyhow::Result<()> {

    let yaml_str = std::fs::read_to_string(docker_compose_file)
        .map_err(|err| anyhow!("Error of opening [{docker_compose_file:?}] ({err:?})")) ?;

    // Multi document support, doc is a yaml::Yaml
    let mut yaml_docs = YamlLoader::load_from_str(&yaml_str).test_unwrap();

    let mut changed = false;
    for yaml in &mut yaml_docs {
        changed |= remove_host_ports_in_docker_compose_yaml(yaml) ?;
    }

    if changed {
        save_yaml(&yaml_docs, docker_compose_file) ?;
    }

    Ok(())
}


/// Returns true if fixed (to save file)
pub fn remove_host_ports_in_docker_compose_yaml(yaml: &mut Yaml) -> anyhow::Result<bool> {

    let mut changed = false;
    let services = &mut yaml["services"];

    match services {
        Yaml::Hash(ref mut services) => {
            for (ref _serv_name, ref mut serv_doc) in services {

                let ports = &mut serv_doc["ports"];
                match ports {
                    Yaml::Array(ports) => {
                        for port in ports {
                            match port {
                                Yaml::String(port_pair) => {
                                    let parts = port_pair.rsplit_once(":");
                                    if let Some((_, container_port_str)) = parts {
                                        let as_int_port: i64 = FromStr::from_str(container_port_str)
                                            .map_err(|_|anyhow!("Incorrect port format [{container_port_str} (in ports pair [{port_pair}])]")) ?;
                                        *port = Yaml::Integer(as_int_port);
                                        changed = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    };

    Ok(changed)
}



pub fn change_network(docker_compose_file: &Path, network_name_policy: &NamePolicy, test_session_id: i64) -> anyhow::Result<()> {

    let yaml_str = std::fs::read_to_string(docker_compose_file)
        .map_err(|err| anyhow!("Error of opening [{docker_compose_file:?}] ({err:?})")) ?;

    // Multi document support, doc is a yaml::Yaml
    let mut yaml_docs = YamlLoader::load_from_str(&yaml_str).test_unwrap();

    let mut changed = false;
    for yaml in &mut yaml_docs {
        changed |= change_network_in_docker_compose_yaml(yaml, network_name_policy, test_session_id) ?;
    }

    if changed {
        save_yaml(&yaml_docs, docker_compose_file) ?;
    }

    Ok(())
}


/// Returns true if fixed (to save file)
pub fn change_network_in_docker_compose_yaml(yaml: &mut Yaml, network_name_policy: &NamePolicy, test_session_id: i64) -> anyhow::Result<bool> {

    let networks = &mut yaml["networks"];
    let mut changed = false;

    if let Yaml::Hash(ref mut networks) = networks {
        for (_net_alias_name, net_doc) in networks {
            let net_name = &mut net_doc["name"];
            match net_name {
                Yaml::String(ref mut net_name) => {
                    *net_name = change_name_by_policy(net_name, network_name_policy, test_session_id) ?;
                    changed = true;
                }
                _ => {}
            }
        }
    }

    Ok(changed)
}

fn change_name_by_policy(base_name: &str, network_name_policy: &NamePolicy, test_session_id: i64) -> anyhow::Result<String> {
    match network_name_policy {
        NamePolicy::Original => Ok(base_name.to_owned()),
        NamePolicy::Custom(ref new_network_name) => Ok(new_network_name.to_string()),
        NamePolicy::WithSuffix(ref suffix) => Ok(format!("{base_name}{suffix}")),
        NamePolicy::WithRandomSuffix => {
            let rnd: i64 = chrono::Local::now().timestamp();
            Ok(format!("{base_name}-{rnd}"))
        }
        NamePolicy::WithBuildIdSuffix => {
            let build_id: i64 = build_id() ?;
            Ok(format!("{base_name}-{build_id}"))
        }
        NamePolicy::WithTestSessionIdSuffix =>
            Ok(format!("{base_name}-{test_session_id}")),
    }
}


fn copy_volume_src_data(docker_compose_file: &Path, docker_compose_project_dir: &Path, test_target_dir: &Path) -> anyhow::Result<()> {
    let host_volumes_src = gather_host_volumes_src(docker_compose_file) ?;
    println!("### host_volumes_src: {host_volumes_src:?}");

    let as_copy_params = host_volumes_src.iter()
        .map(|src|{
            Copy {
                from: src.into(),
                to: src.into(),
            }
        })
        .collect::<Vec<Copy>>();

    println!("### docker_compose_project_dir: {docker_compose_project_dir:?}");
    copy_to_test_target_dir(&as_copy_params, docker_compose_project_dir, test_target_dir) ?;

    Ok(())
}


pub fn gather_host_volumes_src(docker_compose_file: &Path) -> anyhow::Result<HashSet<String>> {

    let volume_pairs = gather_volumes(docker_compose_file) ?;
    let volumes_src: HashSet<&str> = volume_pairs.iter()
        .filter_map(|volume_mapping|{
            let sp = volume_mapping.split_once(':');
            match sp {
                None => None,
                Some((src, _)) => Some(src),
            }
        })
        .collect();

    let volumes_src = volumes_src.into_iter().map(|s|s.to_string()).collect::<HashSet<String>>();
    Ok(volumes_src)
}

fn gather_volumes(docker_compose_file: &Path) -> anyhow::Result<Vec<String>> {
    let yaml_str = std::fs::read_to_string(docker_compose_file)
        .map_err(|err| anyhow!("Error of opening [{docker_compose_file:?}] ({err:?})")) ?;

    // Multi document support, doc is a yaml::Yaml
    let mut yaml_docs = YamlLoader::load_from_str(&yaml_str).test_unwrap();

    let mut volumes = Vec::<String>::new();
    for yaml in &mut yaml_docs {
        volumes.extend(gather_volumes_in_docker_compose_yaml(yaml) ?);
    }

    Ok(volumes)
}


// Public only for tests.
pub fn gather_volumes_in_docker_compose_yaml(yaml: &Yaml) -> anyhow::Result<Vec<String>> {

    let services = &yaml["services"];
    let mut all_volumes = Vec::<String>::new();

    /*
    if let Yaml::Hash(ref services) = services {
        services.iter().flat_map(|(ref serv_name, ref serv_doc)|{
            let volumes = &serv_doc["volumes"];
            if let Yaml::Array(volumes) = volumes {
                volumes.iter().flat_map(|volume|{
                    match volume {
                        Yaml::String(volume) => {
                            volume.to_owned()
                        }
                        _ => {}
                    }
                }
            }
        })
    }
    */

    match services {
        Yaml::Hash(ref services) => {
            for (ref _serv_name, ref serv_doc) in services {

                let volumes = &serv_doc["volumes"];
                match volumes {
                    Yaml::Array(volumes) => {
                        for volume in volumes {
                            match volume {
                                Yaml::String(volume) => {
                                    all_volumes.push(volume.to_owned())
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    };

    Ok(all_volumes)
}


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
