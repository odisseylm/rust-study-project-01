use std::path::PathBuf;
use anyhow::anyhow;
use crate::string::remove_optional_suffix;
//--------------------------------------------------------------------------------------------------



// Returns without extension
pub fn current_exe_name() -> Result<String, anyhow::Error> {
    let cur_exe_as_os_str = std::env::current_exe()
        .map(|ref p| p.file_name().map(|s|s.to_os_string())) ?
        .ok_or_else(||anyhow!("Cannot find executable name.")) ?;
    let cur_exe = cur_exe_as_os_str.to_string_lossy().to_string();
    let cur_exe = remove_optional_suffix(cur_exe, ".exe");
    Ok(cur_exe)
}

pub fn current_exe_dir() -> Result<PathBuf, anyhow::Error> {
    let exe_path = std::env::current_exe() ?;
    let exe_path_dir = exe_path.parent().ok_or_else(|| anyhow::anyhow!("No parent for exe path [{exe_path:?}].")) ?;
    Ok(exe_path_dir.to_path_buf())
}