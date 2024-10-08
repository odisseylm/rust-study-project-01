use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use anyhow::anyhow;
use log::info;
use mvv_common::test::BuildEnv;
use crate::docker_compose::{docker_compose_ps, docker_compose_start_except, docker_compose_stop_except, wait_for_healthy_except};
//--------------------------------------------------------------------------------------------------



pub fn copy_code_coverage_files_for_all_containers(
    docker_compose_dir: &Path, container_name_label: &str)
    -> anyhow::Result<()> {

    let build_env = BuildEnv::try_new() ?;
    let coverage_raw_dir = build_env.target_dir.join("coverage-raw");

    let containers = docker_compose_ps(docker_compose_dir) ?;

    let to_ignore_services = ["database", "postgres"];
    restart_containers(docker_compose_dir, &to_ignore_services) ?;

    for ref container in containers {

        if to_ignore_services.contains(&container.service.as_str()) {
            continue;
        }

        let copy_coverage_res = copy_code_coverage_files(
            &container.id, container_name_label, docker_compose_dir, &coverage_raw_dir);
        if copy_coverage_res.is_err() {
            info!("Error of copy code coverage data.");
        }
    }

    Ok(())
}



fn copy_code_coverage_files(container_id: &str, container_name_label: &str,
                            docker_compose_dir: &Path, coverage_dir: &Path) -> anyhow::Result<()> {

    let container_code_coverage_raw_dir = format!("{container_id}:/appuser/code-coverage/");
    let project_code_coverage_raw_dir = coverage_dir.join(container_name_label);
    let project_code_coverage_raw_dir_str = project_code_coverage_raw_dir.to_string_lossy();

    let cmd = Command::new("docker")
        .current_dir(docker_compose_dir.to_path_buf())
        .args(["cp", &container_code_coverage_raw_dir, &project_code_coverage_raw_dir_str].into_iter())
        .status() ?;

    if cmd.success() {
        // This sub-dir is created by 'docker cp' if base target dir already exists.
        let possible_coverage_sub_dir = project_code_coverage_raw_dir.join("code-coverage");
        if possible_coverage_sub_dir.exists() {
            use fs_extra::dir;
            use fs_extra::dir::DirOptions;

            let files = dir::get_dir_content2(
                &possible_coverage_sub_dir,
                &DirOptions {
                    depth: 1,
                    .. Default::default()
                }) ?.files;

            let files = files.iter()
                .map(|f|PathBuf::from(f))
                .collect::<Vec<_>>();

            fs_extra::move_items(
                &files,
                &project_code_coverage_raw_dir,
                &dir::CopyOptions {
                    copy_inside: true,
                    ..Default::default()
                }) ?;
        }
        Ok(())
    } else {
        Err(anyhow!("Copying code coverage files failed for container."))
    }
}


fn restart_containers(docker_compose_dir: &Path, ignore_services: &[&str])
                      -> anyhow::Result<()> {

    // Restart to surely flush code-coverage.
    info!("### Stopping docker compose services (except {ignore_services:?})");
    docker_compose_stop_except(docker_compose_dir, ignore_services) ?;

    info!("### Starting again docker compose services");
    docker_compose_start_except(docker_compose_dir, ignore_services) ?;

    wait_for_healthy_except(docker_compose_dir, ignore_services, Duration::from_secs(15)) ?;
    Ok(())
}


/// Param 'exe_path' is used to verify presence of LLVM section or LLVM symbols.
pub fn is_code_coverage_enabled_for(exe_path: &Path) -> anyhow::Result<bool> {

    use elf::ElfBytes;
    use elf::endian::AnyEndian;

    // I didn't find (working) ELF lib, which does not load the whole file to memory :-(
    let file_data = std::fs::read(exe_path) ?;
    let file_data = file_data.as_slice();
    let elf_bytes = ElfBytes::<AnyEndian>::minimal_parse(file_data) ?;

    let some_llvm_sections = ["__llvm_prf_names", "__llvm_prf_cnts", "__llvm_prf_data",];
    let has_llvm_section =some_llvm_sections.iter()
        .any(|some_llvm_section|{
            let section = elf_bytes
                .section_header_by_name(some_llvm_section);
            if let Ok(Some(ref _section)) = section {
                true
            } else {
                false
            }
        });

    Ok(has_llvm_section)
}
