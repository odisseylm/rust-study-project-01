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


use core::fmt::{Debug, Display};


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



pub mod __private {
    use crate::util::backtrace::{ BacktraceInfo, BacktraceCopyProvider };

    pub fn error_debug_fmt_impl<
        Err          : BacktraceCopyProvider,
        ErrKind      : core::fmt::Debug,
        ErrSource    : core::fmt::Debug + BacktraceCopyProvider,
        ErrKindFn    : FnOnce(&Err)->&ErrKind,
        BtFn         : FnOnce(&Err)->&BacktraceInfo,
        ErrSourceFn  : FnOnce(&Err)->&ErrSource,
    > (f              : & mut core::fmt::Formatter<'_>,
       error          : & Err,
       this_class_name: & 'static str,
       err_kind_fn    : ErrKindFn,
       err_src_fn     : ErrSourceFn,
       btf            : BtFn,
    ) -> core::fmt::Result {

        let err_self_backtrace = btf(error);
        let err_kind   = err_kind_fn(error);
        let err_source = err_src_fn(error);

        if err_self_backtrace.is_captured() {
            let src_contains_captured_backtrace: bool = BacktraceCopyProvider::contains_self_or_child_captured_backtrace(err_source);

            if src_contains_captured_backtrace {
                // We hope there that 'Debug' of error source prints stacktrace (to avoid printing backtrace several times).
                write!(f, "{} {{ kind: {:?}, source: {:?} }}", this_class_name, err_kind, err_source)
            } else {
                write!(f, "{} {{ kind: {:?}, source: {:?}, backtrace: {} }}", this_class_name, err_kind, err_source, err_self_backtrace)
            }
        } else {
            write!(f, "{} {{ kind: {:?}, source: {:?} }}", this_class_name, err_kind, err_source)
        }
    }

}


#[cfg(test)]
mod tests {
}
