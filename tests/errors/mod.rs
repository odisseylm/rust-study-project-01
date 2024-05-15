use anyhow::{anyhow, Context};
use project01::util::{BacktraceInfo, ToAnyHowErrorFn};



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
*/



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



fn fn_serde_json_wrapped_by_anyhow_using_question_op() -> Result<Entity1, anyhow::Error> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    let entity: Entity1 = serde_json::from_str(str) ?;
    Ok(entity)
}
pub fn fn_serde_json_wrapped_by_anyhow_using_question_op_01() -> Result<Entity1, anyhow::Error> { fn_serde_json_wrapped_by_anyhow_using_question_op() }
pub fn fn_serde_json_wrapped_by_anyhow_using_question_op_02() -> Result<Entity1, anyhow::Error> { fn_serde_json_wrapped_by_anyhow_using_question_op_01() }
pub fn fn_serde_json_wrapped_by_anyhow_using_question_op_03() -> Result<Entity1, anyhow::Error> { fn_serde_json_wrapped_by_anyhow_using_question_op_02() }
pub fn fn_serde_json_wrapped_by_anyhow_using_question_op_04() -> Result<Entity1, anyhow::Error> { fn_serde_json_wrapped_by_anyhow_using_question_op_03() }
pub fn fn_serde_json_wrapped_by_anyhow_using_question_op_05() -> Result<Entity1, anyhow::Error> { fn_serde_json_wrapped_by_anyhow_using_question_op_04() }



fn fn_wrap_by_my_error_using_map_err_and_question_op() -> Result<Entity1, anyhow::Error> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    let r: serde_json::error::Result<Entity1> = serde_json::from_str(str);
    // !!! WORKING conversion to anyhow::Result !!!
    let ok = r.map_err(|json_err|{ MyError334::JsonError2 { source: json_err} }) ?;
    Ok(ok)
}
pub fn fn_wrap_by_my_error_using_map_err_and_question_op_01() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_question_op() }
pub fn fn_wrap_by_my_error_using_map_err_and_question_op_02() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_question_op_01() }
pub fn fn_wrap_by_my_error_using_map_err_and_question_op_03() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_question_op_02() }
pub fn fn_wrap_by_my_error_using_map_err_and_question_op_04() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_question_op_03() }
pub fn fn_wrap_by_my_error_using_map_err_and_question_op_05() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_question_op_04() }



fn fn_wrap_by_my_error_using_map_err_and_with_context() -> Result<Entity1, anyhow::Error> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    let r: serde_json::error::Result<Entity1> = serde_json::from_str(str);

    // !!! WORKING conversion to anyhow::Result !!!
    r.map_err(|json_err|{ MyError334::JsonError2 { source: json_err } })
        .with_context(|| "Failed to read/parse json from web.")
}
pub fn fn_wrap_by_my_error_using_map_err_and_with_context_01() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_with_context() }
pub fn fn_wrap_by_my_error_using_map_err_and_with_context_02() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_with_context_01() }
pub fn fn_wrap_by_my_error_using_map_err_and_with_context_03() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_with_context_02() }
pub fn fn_wrap_by_my_error_using_map_err_and_with_context_04() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_with_context_03() }
pub fn fn_wrap_by_my_error_using_map_err_and_with_context_05() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_with_context_04() }



fn fn_wrap_by_my_error_using_map_err() -> Result<Entity1, anyhow::Error> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    let r: serde_json::error::Result<Entity1> = serde_json::from_str(str);

    // !!! half-working conversion to anyhow::Result !!!
    // !!! It does not create anyhow errors chain !!!
    r.map_err(|json_err|{ MyError334::JsonError2 { source: json_err} })
        .map_err(anyhow::Error::msg)
}
pub fn fn_wrap_by_my_error_using_map_err_01() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err() }
pub fn fn_wrap_by_my_error_using_map_err_02() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_01() }
pub fn fn_wrap_by_my_error_using_map_err_03() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_02() }
pub fn fn_wrap_by_my_error_using_map_err_04() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_03() }
pub fn fn_wrap_by_my_error_using_map_err_05() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_04() }



fn fn_wrap_by_my_error_using_map_err_and_anyhow_macro() -> Result<Entity1, anyhow::Error> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    let r: serde_json::error::Result<Entity1> = serde_json::from_str(str);

    r.map_err(|json_err|{ anyhow!(MyError334::JsonError2 { source: json_err}) })
}
pub fn fn_wrap_by_my_error_using_map_err_and_anyhow_macro_01() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_anyhow_macro() }
pub fn fn_wrap_by_my_error_using_map_err_and_anyhow_macro_02() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_anyhow_macro_01() }
pub fn fn_wrap_by_my_error_using_map_err_and_anyhow_macro_03() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_anyhow_macro_02() }
pub fn fn_wrap_by_my_error_using_map_err_and_anyhow_macro_04() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_anyhow_macro_03() }
pub fn fn_wrap_by_my_error_using_map_err_and_anyhow_macro_05() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_map_err_and_anyhow_macro_04() }



fn fn_wrap_by_anyhow_error_using_map_err_and_anyhow_macro() -> Result<Entity1, anyhow::Error> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    let r: serde_json::error::Result<Entity1> = serde_json::from_str(str);
    r.map_err(|json_err|{ anyhow!(json_err) })
}
pub fn fn_wrap_by_anyhow_error_using_map_err_and_anyhow_macro_01() -> Result<Entity1, anyhow::Error> { fn_wrap_by_anyhow_error_using_map_err_and_anyhow_macro() }
pub fn fn_wrap_by_anyhow_error_using_map_err_and_anyhow_macro_02() -> Result<Entity1, anyhow::Error> { fn_wrap_by_anyhow_error_using_map_err_and_anyhow_macro_01() }
pub fn fn_wrap_by_anyhow_error_using_map_err_and_anyhow_macro_03() -> Result<Entity1, anyhow::Error> { fn_wrap_by_anyhow_error_using_map_err_and_anyhow_macro_02() }
pub fn fn_wrap_by_anyhow_error_using_map_err_and_anyhow_macro_04() -> Result<Entity1, anyhow::Error> { fn_wrap_by_anyhow_error_using_map_err_and_anyhow_macro_03() }
pub fn fn_wrap_by_anyhow_error_using_map_err_and_anyhow_macro_05() -> Result<Entity1, anyhow::Error> { fn_wrap_by_anyhow_error_using_map_err_and_anyhow_macro_04() }



fn fn_wrap_by_my_error_using_my_fn_to_anyhow_error_fn() -> Result<Entity1, anyhow::Error> {
    let str = r#"{ "intField_666": 123, "stringField": "str123" }"#;
    // let r: serde_json::error::Result<Entity1> = serde_json::from_str(str);
    let r: Result<Entity1, serde_json::Error> = serde_json::from_str(str);
    r.to_anyhow_error_fn(|json_err|{ MyError334::JsonError2 { source: json_err} })
}
pub fn fn_wrap_by_my_error_using_my_fn_to_anyhow_error_fn_01() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_my_fn_to_anyhow_error_fn() }
pub fn fn_wrap_by_my_error_using_my_fn_to_anyhow_error_fn_02() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_my_fn_to_anyhow_error_fn_01() }
pub fn fn_wrap_by_my_error_using_my_fn_to_anyhow_error_fn_03() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_my_fn_to_anyhow_error_fn_02() }
pub fn fn_wrap_by_my_error_using_my_fn_to_anyhow_error_fn_04() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_my_fn_to_anyhow_error_fn_03() }
pub fn fn_wrap_by_my_error_using_my_fn_to_anyhow_error_fn_05() -> Result<Entity1, anyhow::Error> { fn_wrap_by_my_error_using_my_fn_to_anyhow_error_fn_04() }
