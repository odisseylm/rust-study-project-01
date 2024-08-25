use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;
use anyhow::anyhow;
use yaml_rust2::{Yaml, YamlLoader};
use crate::test::{
    change_name_by_policy,
    docker_compose::get_docker_image_profile,
    files::{do_replacements, Replace},
    integration::{save_yaml, NamePolicy},
};
//--------------------------------------------------------------------------------------------------



pub fn set_docker_image_profile_suffix_var(new_docker_compose_file: &Path, test_res_dir: &Path) -> anyhow::Result<()> {
    let docker_image_profile = get_docker_image_profile();
    let docker_image_profile_suffix: &str = docker_image_profile.as_docker_tag_suffix();

    let r = vec!(Replace::by_str(
        new_docker_compose_file.to_path_buf(),
        ["${DOCKER_IMAGE_PROFILE_SUFFIX}"], [docker_image_profile_suffix]));
    do_replacements(&r, &test_res_dir) ?;

    // just in case lets put it to env vars too
    std::env::set_var("DOCKER_IMAGE_PROFILE_SUFFIX", docker_image_profile_suffix);

    Ok(())
}


pub fn remove_host_ports(docker_compose_file: &Path) -> anyhow::Result<()> {

    let yaml_str = std::fs::read_to_string(docker_compose_file)
        .map_err(|err| anyhow!("Error of opening [{docker_compose_file:?}] ({err:?})")) ?;

    // Multi document support, doc is a yaml::Yaml
    let mut yaml_docs = YamlLoader::load_from_str(&yaml_str) ?;

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
    let mut yaml_docs = YamlLoader::load_from_str(&yaml_str) ?;

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
    let mut yaml_docs = YamlLoader::load_from_str(&yaml_str) ?;

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
