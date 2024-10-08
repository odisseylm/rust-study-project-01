mod build_env;

use core::fmt::Debug;
use std::ffi::OsString;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use anyhow::anyhow;
use itertools::Itertools;
use crate::string::is_os_str_true;
//--------------------------------------------------------------------------------------------------


pub use build_env::BuildEnv;
// Actually this code is designed for unit test only,
// but in that case due to strange rust project tests build approach
// it causes showing 'unused code'.
// For that reason I've decided NOW to put it in prod code
// (probably later I'll move them back to 'tests' source directory and suppress
// and will add #[allow(dead_code)])


/// This trait and its impl was added to minimize uncontrolled usage of panic-risky unwrap.
/// Please
///  * use test_unwrap() in tests.
///  * use unchecked_unwrap() in 'xxx_unchecked' methods.
///
/// Try not use pure unwrap() at all production code (to avoid unpredictable panic).
///
pub trait TestResultUnwrap <Ok, Err: Debug> {
    fn test_unwrap(self) -> Ok;
}
pub trait TestOptionUnwrap <Ok> {
    fn test_unwrap(self) -> Ok;
}

//noinspection DuplicatedCode
impl<Ok,Err: Debug> TestResultUnwrap<Ok,Err> for Result<Ok,Err> {
    #[inline]
    #[track_caller]
    fn test_unwrap(self) -> Ok {
        self.unwrap() // allowed
    }
}

impl<Ok> TestOptionUnwrap<Ok> for Option<Ok> {
    #[inline]
    #[track_caller]
    fn test_unwrap(self) -> Ok {
        self.unwrap() // allowed
    }
}


pub trait TestDisplayStringOps {
    #[track_caller]
    fn to_test_display_string(&self) -> String;
    #[track_caller]
    fn to_test_string(&self) -> String;
}

pub trait TestOptionDisplayStringOps {
    #[track_caller]
    fn to_test_display_string(&self) -> String;
    #[track_caller]
    fn to_test_string(&self) -> String;
}

pub trait TestDebugStringOps {
    #[track_caller]
    fn to_test_debug_string(&self) -> String;
}

impl<T> TestDisplayStringOps for T where T: Display {
    #[track_caller]
    fn to_test_display_string(&self) -> String {
        let mut str_buf = String::new();
        use core::fmt::Write;
        write!(str_buf, "{}", self).test_unwrap();
        str_buf
    }
    #[track_caller]
    fn to_test_string(&self) -> String {
        self.to_string()
    }
}

impl<T> TestOptionDisplayStringOps for Option<T> where T: Display {
    #[track_caller]
    fn to_test_display_string(&self) -> String {
        match self {
            None => "None".to_owned(),
            Some(ref v) => {
                let mut str_buf = String::new();
                use core::fmt::Write;
                write!(str_buf, "{}", v).test_unwrap();
                str_buf
            }
        }
    }
    #[track_caller]
    fn to_test_string(&self) -> String {
        match self {
            None => "None".to_owned(),
            Some(ref v) => v.to_string()
        }
    }
}

/*
impl<T> TestOptionDisplayStringOps for Option<&T> where T: Display {
    #[track_caller]
    fn to_test_display_string(&self) -> String {
        match self {
            None => "None".to_owned(),
            Some(ref v) => {
                let mut str_buf = String::new();
                use core::fmt::Write;
                write!(str_buf, "{}", v).test_unwrap();
                str_buf
            }
        }
    }
    #[track_caller]
    fn to_test_string(&self) -> String {
        self.to_string()
    }
}
*/

impl<T> TestDebugStringOps for T where T: Debug {
    #[track_caller]
    fn to_test_debug_string(&self) -> String {
        let mut str_buf = String::new();
        use core::fmt::Write;
        write!(str_buf, "{:?}", self).test_unwrap();
        str_buf
    }
}


#[extension_trait::extension_trait]
pub impl<T> TestOps for T where T: Clone {
    fn test_clone(&self) -> Self {
        self.clone()
    }
}

#[extension_trait::extension_trait]
//noinspection DuplicatedCode
pub impl<V,E> TestResultDebugErrOps for Result<V,E> where E: Debug {
    // #[inline] // warning: `#[inline]` is ignored on function prototypes
    #[track_caller]
    fn err_to_test_debug_string(self) -> String {
        self.err().test_unwrap().to_test_debug_string()
    }
}

#[extension_trait::extension_trait]
pub impl<V,E> TestResultDisplayErrOps for Result<V,E> where E: Display {
    // #[inline] // warning: `#[inline]` is ignored on function prototypes
    #[track_caller]
    fn err_to_test_display_string(self) -> String {
        // self.err().test_unwrap().to_test_display_string()

        let err = self.err().test_unwrap();
        let mut str_buf = String::new();
        use core::fmt::Write;
        write!(str_buf, "{}", err).test_unwrap();
        str_buf
    }
}



pub fn is_manually_launched_task() -> bool {
    let is_exact = std::env::args_os().contains(&OsString::from("--exact"));
    is_exact
}


#[allow(non_snake_case)]
pub fn is_CI_build() -> bool {
    let ci_env_var = std::env::var_os(&OsString::from("CARGO_MAKE_CI"));
    match ci_env_var {
        None => false,
        Some(ref v) => is_os_str_true(v),
    }
}


pub fn current_sub_project_dir() -> anyhow::Result<PathBuf> {
    let sub_project_dir = crate::env::required_env_var_static("CARGO_MANIFEST_DIR") ?;
    let sub_project_dir: PathBuf = sub_project_dir.into();
    Ok(sub_project_dir)
}


pub fn current_project_target_dir() -> anyhow::Result<PathBuf> {
    let out_dir_str = crate::env::required_env_var_static("OUT_DIR") ?;
    let out_dir: PathBuf = out_dir_str.into();

    let target_dir = find_target_dir(&out_dir) ?;
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


pub fn find_target_dir(dir: &Path) -> anyhow::Result<PathBuf> {

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

