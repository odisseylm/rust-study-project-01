// use serde::Deserialize;
// use serde_json::error::Result;
// use serde_with_macros::serde_as;
//use thiserror::Error;

use anyhow::{anyhow, Context};
use crate::util::{BacktraceInfo, ToAnyHowErrorFn };


// #[serde_as]
#[derive(Debug)]
#[derive(serde::Deserialize)]
pub struct Entity1 {
    // #[serde_as(as = "intField")]
    // #[serde(rename(serialize = "intField", deserialize = "intField"), alias="int_field")]
    #[serde(alias="intField")]
    pub int_field: i32,
    // #[serde_as(as = "stringField")]
    #[serde(alias="stringField")]
    pub string_field: String,
}


pub fn extract_json() -> Result<Entity1, serde_json::error::Error> {
    // let correct_json = "{ \"int_field\": 123, \"string_field\": \"str123\" }";
    let incorrect_json = "{ \"intField_666\": 123, \"stringField\": \"str123\" }";
    let r: Result<Entity1, serde_json::Error> = serde_json::from_str(incorrect_json);
    return r;
}


pub fn extract_json_1() -> Result<Entity1, serde_json::error::Error> {
    extract_json()
}
pub fn extract_json_2() -> Result<Entity1, serde_json::error::Error> { extract_json_1() }
pub fn extract_json_3() -> Result<Entity1, serde_json::error::Error> { extract_json_2() }
pub fn extract_json_4() -> Result<Entity1, serde_json::error::Error> { extract_json_3() }
pub fn extract_json_5() -> Result<Entity1, serde_json::error::Error> { extract_json_4() }




#[allow(dead_code)]
// #[derive(thiserror::Error, Debug)]
pub enum MyError {
    Io {
        // #[from]
        source: std::io::Error,
        backtrace: std::backtrace::Backtrace,
    },
    Io2 {
        val1: & 'static str,
        val2: i32,
    },
}


#[allow(dead_code)]
#[derive(Debug)]
pub enum DataStoreError1054554 {
    MyError3331 {
        backtrace: std::backtrace::Backtrace,
        // backtrace: i32,
    },
}


#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub enum DataStoreError102 {
    #[error("my error 3331")]
    MyError3331 {
        // backtrace: std::backtrace::Backtrace,
        // backtrace: i32,
    },
    #[error("data store disconnected")]
    Disconnect(#[from] std::io::Error),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    #[error("unknown data store error")]
    Unknown,
}


/*
// #[derive(thiserror::Error, Debug)]
#[derive(Debug)]
pub enum MyError333 {
    // #[error("my error 3331")]
    MyError3331 {
        backtrace: std::backtrace::Backtrace,
        // backtrace: i32,
    },
    // MyError3331 {
    //     backtrace: std::backtrace::Backtrace,
    // },
    // #[error("data store disconnected")]
    // Disconnect(#[from] std::io::Error),
    Disconnect(std::io::Error),
    // #[error("the data for key `{0}` is not available")]
    Redaction(String),
    // #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    // #[error("unknown data store error")]
    Unknown,
}

fn dsdsdsd() {
    let aa1: MyError = MyError::Io2 { val1: "fdfd", val2: 434 };
    let aa2: MyError333 = MyError333::MyError3331 { backtrace: std::backtrace::Backtrace::capture() };
}

*/

// struct MyError333 {
// }


#[derive(Debug)]
pub enum MyError333 {
    JsonError {
        serde_error: serde_json::Error,
        backtrace: BacktraceInfo,
    },
    JsonError2 {
        serde_error: serde_json::Error,
    },
}



#[derive(thiserror::Error, Debug)]
pub enum MyError334 {
    // #[error("Json error 2")]
    // JsonError2(serde_json::Error),
    #[error("Json error 2")]
    JsonError2{
        #[source]
        source: serde_json::Error
    },
}



impl core::fmt::Display for MyError333 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn error_fn() -> Result<Entity1, MyError333> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    let r: serde_json::error::Result<Entity1> = serde_json::from_str(str);

    r.map_err(|json_err|{ MyError333::JsonError {
            serde_error: json_err,
            backtrace: BacktraceInfo::new(),
        } })
}

pub fn error_fn_1() -> Result<Entity1, MyError333> { error_fn() }
pub fn error_fn_2() -> Result<Entity1, MyError333> { error_fn_1() }
pub fn error_fn_3() -> Result<Entity1, MyError333> { error_fn_2() }
pub fn error_fn_4() -> Result<Entity1, MyError333> { error_fn_3() }
pub fn error_fn_5() -> Result<Entity1, MyError333> { error_fn_4() }



fn error_fn_100() -> Result<Entity1, anyhow::Error> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    let entity: Entity1 = serde_json::from_str(str) ?;
    Ok(entity)
}
pub fn error_fn_101() -> Result<Entity1, anyhow::Error> { error_fn_100() }
pub fn error_fn_102() -> Result<Entity1, anyhow::Error> { error_fn_101() }
pub fn error_fn_103() -> Result<Entity1, anyhow::Error> { error_fn_102() }
pub fn error_fn_104() -> Result<Entity1, anyhow::Error> { error_fn_103() }
pub fn error_fn_105() -> Result<Entity1, anyhow::Error> { error_fn_104() }


fn error_fn_200() -> Result<Entity1, anyhow::Error> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    let r: serde_json::error::Result<Entity1> = serde_json::from_str(str);
    // !!! WORKING conversion to anyhow::Result !!!
    let ok = r.map_err(|json_err|{ MyError334::JsonError2 { source: json_err} }) ?;
    Ok(ok)
}
pub fn error_fn_201() -> Result<Entity1, anyhow::Error> { error_fn_200() }
pub fn error_fn_202() -> Result<Entity1, anyhow::Error> { error_fn_201() }
pub fn error_fn_203() -> Result<Entity1, anyhow::Error> { error_fn_202() }
pub fn error_fn_204() -> Result<Entity1, anyhow::Error> { error_fn_203() }
pub fn error_fn_205() -> Result<Entity1, anyhow::Error> { error_fn_204() }



fn error_fn_300() -> Result<Entity1, anyhow::Error> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    let r: serde_json::error::Result<Entity1> = serde_json::from_str(str);

    // !!! WORKING conversion to anyhow::Result !!!
    r.map_err(|json_err|{ MyError334::JsonError2 { source: json_err } })
        .with_context(|| "Failed to read/parse json from web.")
}
pub fn error_fn_301() -> Result<Entity1, anyhow::Error> { error_fn_300() }
pub fn error_fn_302() -> Result<Entity1, anyhow::Error> { error_fn_301() }
pub fn error_fn_303() -> Result<Entity1, anyhow::Error> { error_fn_302() }
pub fn error_fn_304() -> Result<Entity1, anyhow::Error> { error_fn_303() }
pub fn error_fn_305() -> Result<Entity1, anyhow::Error> { error_fn_304() }



fn error_fn_400() -> Result<Entity1, anyhow::Error> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    let r: serde_json::error::Result<Entity1> = serde_json::from_str(str);

    // !!! half-working conversion to anyhow::Result !!!
    // !!! It does not create anyhow errors chain !!!
    r.map_err(|json_err|{ MyError334::JsonError2 { source: json_err} })
        .map_err(anyhow::Error::msg)
}
pub fn error_fn_401() -> Result<Entity1, anyhow::Error> { error_fn_400() }
pub fn error_fn_402() -> Result<Entity1, anyhow::Error> { error_fn_401() }
pub fn error_fn_403() -> Result<Entity1, anyhow::Error> { error_fn_402() }
pub fn error_fn_404() -> Result<Entity1, anyhow::Error> { error_fn_403() }
pub fn error_fn_405() -> Result<Entity1, anyhow::Error> { error_fn_404() }



fn error_fn_500() -> Result<Entity1, anyhow::Error> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    let r: serde_json::error::Result<Entity1> = serde_json::from_str(str);

    r.map_err(|json_err|{ anyhow!(MyError334::JsonError2 { source: json_err}) })
}
pub fn error_fn_501() -> Result<Entity1, anyhow::Error> { error_fn_500() }
pub fn error_fn_502() -> Result<Entity1, anyhow::Error> { error_fn_501() }
pub fn error_fn_503() -> Result<Entity1, anyhow::Error> { error_fn_502() }
pub fn error_fn_504() -> Result<Entity1, anyhow::Error> { error_fn_503() }
pub fn error_fn_505() -> Result<Entity1, anyhow::Error> { error_fn_504() }



fn error_fn_600() -> Result<Entity1, anyhow::Error> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    let r: serde_json::error::Result<Entity1> = serde_json::from_str(str);
    r.map_err(|json_err|{ anyhow!(json_err) })
}
pub fn error_fn_601() -> Result<Entity1, anyhow::Error> { error_fn_600() }
pub fn error_fn_602() -> Result<Entity1, anyhow::Error> { error_fn_601() }
pub fn error_fn_603() -> Result<Entity1, anyhow::Error> { error_fn_602() }
pub fn error_fn_604() -> Result<Entity1, anyhow::Error> { error_fn_603() }
pub fn error_fn_605() -> Result<Entity1, anyhow::Error> { error_fn_604() }







fn error_fn_700() -> Result<Entity1, anyhow::Error> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    // let r: serde_json::error::Result<Entity1> = serde_json::from_str(str);
    let r: Result<Entity1, serde_json::Error> = serde_json::from_str(str);
    r.to_anyhow_error_fn(|json_err|{ MyError334::JsonError2 { source: json_err} })
}

pub fn error_fn_701() -> Result<Entity1, anyhow::Error> { error_fn_700() }
pub fn error_fn_702() -> Result<Entity1, anyhow::Error> { error_fn_701() }
pub fn error_fn_703() -> Result<Entity1, anyhow::Error> { error_fn_702() }
pub fn error_fn_704() -> Result<Entity1, anyhow::Error> { error_fn_703() }
pub fn error_fn_705() -> Result<Entity1, anyhow::Error> { error_fn_704() }
