use std::ffi::OsString;
use std::path::{Path, PathBuf};
use anyhow::anyhow;
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
pub struct CopyCfg {
    /// It is used if 'copy' field contains 'non-absolute' files.
    pub base_from_dir: PathBuf,
    pub copy: Vec<Copy>,
}
impl Default for CopyCfg {
    fn default() -> Self {
        Self {
            base_from_dir: PathBuf::new(),
            copy: Vec::new(),
        }
    }
}


pub fn do_replacements(replace_file_content: &Vec<Replace>, test_res_dir: &Path) -> anyhow::Result<()> {
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


pub fn do_copy(copy: &Vec<Copy>, base_from_dir: &Path, to_dir: &Path) -> anyhow::Result<()> {
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
                    to_dir.to_path_buf()
                } else {
                    if from_orig.is_absolute() {
                        to_dir.join(
                            from.file_name().ok_or_else(|| anyhow!("Now filename of [{from:?}]")) ?)
                    } else {
                        to_dir.join(&from_orig)
                    }
                }
            } else {
                to_dir.join(&to)
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
