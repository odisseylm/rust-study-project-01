use mvv_common::backtrace::BacktraceCell;
use mvv_common::entity::{Amount, AmountFormatError};
use mvv_common::test::{TestDisplayStringOps, TestOptionUnwrap};
//--------------------------------------------------------------------------------------------------


#[derive(Debug, thiserror::Error)]
enum ThisError0 {
    #[error("ThisError0::ErrorFromString0 ( {0} )")]
    FromString0(String),
    #[error("ThisError0::FromString0WirthBt ( {error} )")]
    #[allow(dead_code)]
    FromString0WithBt { error: String, backtrace: BacktraceCell, },
    #[error("ThisError0::FromFloatWithBt ( {0} )")]
    #[allow(dead_code)]
    FromFloatWithBt(f64, BacktraceCell),
}

impl mvv_common::backtrace::BacktraceSource for ThisError0 {
    #[allow(unused_imports)]
    fn backtrace_ref(&self) -> Option<&BacktraceCell> {
        use mvv_common::backtrace::BacktraceSource;
        match self {
            ThisError0::FromString0(_) =>
                None,
            ThisError0::FromString0WithBt { ref backtrace, .. } =>
                backtrace.backtrace_ref(),
            ThisError0::FromFloatWithBt(_, ref backtrace) =>
                backtrace.backtrace_ref(),
        }
    }

    #[allow(unused_imports)]
    fn is_taking_backtrace_supported(&self) -> bool {
        use mvv_common::backtrace::BacktraceSource;
        match self {
            ThisError0::FromString0(_) =>
                false,
            ThisError0::FromString0WithBt { ref backtrace, .. } =>
                backtrace.is_taking_backtrace_supported(),
            ThisError0::FromFloatWithBt(_, ref backtrace) =>
                backtrace.is_taking_backtrace_supported(),
        }
    }
}
type StdBacktrace = std::backtrace::Backtrace;

#[derive(
    Debug,
    thiserror::Error,
    mvv_error_macro::ThisErrorFromWithBacktrace,
    mvv_error_macro::ThisErrorBacktraceSource,
)]
enum ThisError1 {
    #[error("ThisError1::ErrorFromString ( {0} )")]
    // ErrorFromString( #[from] String),
    ErrorFromString(String),
    // #[error("ThisError1::ErrorFromStringWithBt")]
    // ErrorFromString { #[from] error: String, backtrace: BacktraceCell, },

    // #[error("ThisError1::ErrorFromThisError0 ( {0} )")]
    // ErrorFromThisError0( #[from] #[source] ThisError0),
    #[error("ThisError1::ErrorFromThisError0 ( {error} )")]
    #[inherit_or_capture]
    // ErrorFromThisError0 { #[from] #[source] error: ThisError0, #[backtrace] backtrace: BacktraceCell },
    // ErrorFromThisError0 { #[source] #[from_bt] error: ThisError0, backtrace: BacktraceCell },
    ErrorFromThisError0 {
        backtrace: BacktraceCell,
        #[source] #[from_bt] error: ThisError0,
    },
    // ErrorFromThisError0 { #[source] #[from_with_bt] error: ThisError0, backtrace: StdBacktrace },

    #[error("ThisError1::ErrorFromEnvVarError ( {0} )")]
    // #[skip_bt_source]
    #[std_error]
    ErrorFromEnvVarError( #[from] #[source] std::env::VarError),

    #[error("ThisError1::ErrorFromEnvVarError ( {0} )")]
    // #[inherit_or_capture]
    // #[std_error]
    ErrorFromStdIoError( #[from_bt] #[source] std::io::Error, StdBacktrace),

    #[error("ThisError1::ErrorFromInt")]
    // ErrorFromInt( #[from] i32),
    #[allow(dead_code)]
    ErrorFromInt(i32),
    // #[error("ThisError1::ErrorFromIntWithBt")]
    // ErrorFromInt { error: i32, backtrace: BacktraceCell, },

    #[error("ThisError1::ErrorFromAnyhowError")]
    ErrorFromAnyhowError( #[from] anyhow::Error),
    #[error("ThisError1::ErrorFromAmountFormatError")]
    ErrorFromAmountFormatError( #[from] AmountFormatError),

    #[error("ThisError1::SqlxError")]
    SqlxError { #[source] #[from_bt] error: sqlx::Error, backtrace: BacktraceCell, },
}


#[test]
fn test_1() {
    println!("\n------------------------------ 01 -------------------------------------");
    let err = ThisError1::ErrorFromString("String123".to_test_string());
    println!("### err 01: {err}");
    println!("### err 01: {err:?}");

    println!("\n------------------------------ 02 -------------------------------------");
    let err0 = ThisError0::FromString0("String345".to_test_string());
    let err = ThisError1::ErrorFromThisError0 {
        error: err0,
        backtrace: BacktraceCell::capture_backtrace().into(),
    };
    println!("### err 01: {err}");
    println!("### err 01: {err:?}");

    println!("\n------------------------------ 03 -------------------------------------");
    let err0 = ThisError0::FromString0("String345".to_test_string());
    let err: ThisError1 = err0.into();
    println!("### err 01: {err}");
    println!("### err 01: {err:?}");

    println!("\n------------------------------ 04 -------------------------------------");
    let err = ThisError1::ErrorFromAmountFormatError(
        Amount::from_str("1234.5 US").err().test_unwrap()
    );
    println!("### err 01: {err}");
    println!("### err 01: {err:?}");
    println!("\n------------------------------ END ------------------------------------");

    assert!(false, "Test error to see console output.");
}
