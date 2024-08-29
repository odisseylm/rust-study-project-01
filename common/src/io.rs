use std::path::{Path, PathBuf};
use anyhow::anyhow;



pub fn find_existent<const N: usize>(files: [&Path;N]) -> Option<&Path> {
    files.into_iter()
        .find(|f|f.exists())
}

pub fn find_existent_buf<const N: usize>(files: [PathBuf;N]) -> Option<PathBuf> {
    files.into_iter()
        .find(|f|f.exists())
}

pub fn get_existent<const N: usize>(files: [&Path;N]) -> anyhow::Result<&Path> {
    files.into_iter()
        .find(|f|f.exists())
        .ok_or_else(||anyhow!("None of files [{files:?}] exists"))
}


// ???
// TODO: add EASY error mapping functions
//  * for not found
//  * others