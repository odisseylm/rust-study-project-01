use std::path::{Path, PathBuf};
use std::time::SystemTime;



pub(crate) fn touch_files_task() {
    let mut args = std::env::args();

    // T O D O: add support of named arg params
    let first_arg = args.nth(2);
    let second_arg = args.nth(3);

    let dir: PathBuf;
    let mask: String;
    let project_dir = std::env::current_dir().unwrap();

    if let Some(ref mask_p) = second_arg {
        mask = mask_p.clone();
        if let Some(ref sub_dir) = first_arg {
            dir = project_dir.join(sub_dir).to_path_buf();
        } else {
            dir = project_dir.to_path_buf();
        }
    } else if let Some(ref mask_p) = first_arg {
        mask = mask_p.clone();
        dir = project_dir.to_path_buf();
    } else {
        eprintln!(
            "Tasks: \n \
               x-touch-files dir mask \n \
               x-touch-files mask \n \
            ");
        panic!("No mask");
    }

    touch_files(&dir, mask.as_str()).unwrap();
}



/*
fn touch_project_files(mask: &str) -> std::io::Result<()> {
    let BuildEnv { project_dir, .. } = BuildEnv::new();
    touch_files(&project_dir, mask)
}
*/

pub(crate) fn touch_files(dir: &Path, mask: &str) -> std::io::Result<()> {
    println!("Touching files in [{:?}] by mask[{mask}]", dir);

    let files = gather_files(dir, mask);
    for f in files {
        touch_file(&f) ?;
    }
    Ok(())
}

fn touch_file(file: &Path) -> std::io::Result<()> {

    let f = std::fs::File::open(file);
    match f {
        Ok(f) => {
            let res = f.set_modified(SystemTime::now());
            if res.is_err() {
                eprintln!("Error of file [{file:?}] updating timestamp (2).");
            } else {
                println!("File [{file:?}] is touched.");
            }
            res
        }
        Err(err) => {
            eprintln!("Error of file [{file:?}] updating timestamp (1).");
            Err(err)
        }
    }
}

fn gather_files(dir: &Path, mask: &str) -> Vec<PathBuf> {
    use walkdir::WalkDir;

    let files = WalkDir::new(&dir).into_iter().filter_map(|entry|{
        match entry {
            Ok(ref entry) if entry.path().is_file() => {
                let f_name = entry.file_name();
                let f_name = f_name.to_string_lossy();
                let f_name: &str = f_name.as_ref();

                let match_to_mask = matches_to_mask(f_name, mask);
                if match_to_mask {
                    Some(entry.path().to_path_buf())
                } else {
                    None
                }
            }
            _ => None,
        }
    })
        .collect::<Vec<_>>();

    files
}


fn matches_to_mask(filename: &str, mask: &str) -> bool {
    let f_name = filename;
    let f_name: &str = f_name.as_ref();

    let matches_to_mask: bool =
        // T O D O: find mask support third-party
        if mask.starts_with("*") && mask.ends_with("*") {
            let expected_f_name_part = mask.strip_prefix("*").unwrap_or(mask);
            let expected_f_name_part = expected_f_name_part.strip_prefix("*").unwrap_or(expected_f_name_part);

            f_name.contains(expected_f_name_part)
        }
        else if mask.starts_with("*") {
            let expected_f_name_start = mask.strip_prefix("*").unwrap_or(mask);
            f_name.ends_with(expected_f_name_start)
        }
        else if mask.ends_with("*") {
            let expected_f_name_end = mask.strip_suffix("*").unwrap_or(mask);
            if f_name.starts_with(expected_f_name_end) {
                true
            } else {
                let f_name = remove_ext(f_name).unwrap_or_else(|s|s);
                f_name.starts_with(expected_f_name_end)
            }
        } else {
            f_name == mask
        };

    matches_to_mask
}


fn remove_ext(f_name: &str) -> Result<String, String> {
    let f_name_as_path = PathBuf::from(f_name);
    let f_name_as_path = f_name_as_path.as_path();
    let f_ext = f_name_as_path.extension();

    if let Some(f_ext) = f_ext {
        let f_ext = f_ext.to_string_lossy();
        let f_ext = f_ext.as_ref();

        let f_name: &str = f_name.strip_suffix(f_ext).unwrap_or(f_name);
        let f_name: &str = f_name.strip_suffix(".").unwrap_or(f_name);
        Ok(f_name.to_string())
    } else {
        Err(f_name.to_string())
    }
}
