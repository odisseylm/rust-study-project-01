// #![feature(error_generic_member_access)]



// Compilation error:
//   unresolved import `thiserror`
//   use of unstable library feature 'error_generic_member_access'
//
// #[derive(thiserror::Error, Debug)]
// pub enum MyError4567 {
//     Io {
//         #[from]
//         source: std::io::Error,
//         backtrace: std::backtrace::Backtrace,
//     },
// }


use std::fmt::{Debug, Display};


pub trait ToAnyHowError <
    Ok,
    Err: Debug,
    Err2: Debug + Display + std::error::Error + Send + Sync + 'static,
    > {
    fn to_anyhow_error(self, err2: Err2) -> Result<Ok, anyhow::Error>;
}

pub trait ToAnyHowErrorFn <
    Ok,
    Err: Debug,
    Err2: Debug + Display + std::error::Error + Send + Sync + 'static,
    F: Fn(Err)->Err2,
    > {
    // TODO: try to use the same name 'to_anyhow_error'
    fn to_anyhow_error_fn(self, f: F) -> Result<Ok, anyhow::Error>;
}


impl<
    Ok,
    Err: Debug,
    Err2: Debug + Display + std::error::Error + Send + Sync + 'static,
    > ToAnyHowError<Ok,Err,Err2> for Result<Ok,Err> {
    fn to_anyhow_error(self, err2: Err2) -> Result<Ok, anyhow::Error> {
        let ok = self.map_err(|_cur_err| err2 ) ?;
        Ok(ok)
        // self.map_err(|json_err| err2 ).map_err(anyhow::Error::msg)
    }
}

impl<
    Ok,
    Err: Debug,
    Err2: Debug + Display + std::error::Error + Send + Sync + 'static,
    F: Fn(Err)->Err2,
    > ToAnyHowErrorFn<Ok,Err,Err2,F> for Result<Ok,Err> {
    fn to_anyhow_error_fn(self, f: F) -> Result<Ok, anyhow::Error> {
        let ok = self.map_err(|err0| f(err0)) ?;
        Ok(ok)
        // self.map_err(|json_err| err2 ).map_err(anyhow::Error::msg)
    }
}

