use std::ffi::OsString;
use std::path::Path;
use anyhow::anyhow;
use itertools::Itertools;
use log::info;
use crate::gen_src::UpdateFile;
//--------------------------------------------------------------------------------------------------


pub fn to_extract_proto_files() -> bool {
    std::env::args_os().contains(&OsString::from("--extract-proto-files"))
        || std::env::args_os().contains(&OsString::from("--extract-proto"))
}

pub fn extract_proto_files<PF: rust_embed::RustEmbed>(
    module_name: &str, update_api_file: UpdateFile, dir: Option<&str>) -> anyhow::Result<()> {

    let alt_dir = format!("proto/{module_name}/proto");
    let dir = dir.map(Path::new)
        .unwrap_or(Path::new(&alt_dir));

    for f in PF::iter() {
        let f_path = f.as_ref();

        let embedded_file = PF::get(f_path)
            .ok_or_else(||anyhow!("No embedded file [{f_path}]")) ?;

        let file = dir.join(f_path);
        let file = file.as_path();

        let parent = Path::new(file).parent();
        if let Some(parent) = parent {
            std::fs::create_dir_all(parent)
                .map_err(|err|anyhow!("Create dir error [{parent:?}] => {err:?}")) ?;
        }

        let to_update_file: bool = match update_api_file {
            UpdateFile::Always => true,
            UpdateFile::IfModelChanged => {
                if !file.exists() { true }
                else {
                    let file_content = std::fs::read(&file) ?;
                    file_content.as_slice() != embedded_file.data.as_ref()
                }
            }
        };

        if to_update_file {
            std::fs::write(&file, embedded_file.data.as_ref())
                .map_err(|err|anyhow!("File error for [{file:?}] => {err:?}")) ?;
            info!("Proto file [{file:?}] is extracted.");
        } else {
            info!("Open API file [{file:?}] is not extracted.");
        }
    }

    Ok(())
}
