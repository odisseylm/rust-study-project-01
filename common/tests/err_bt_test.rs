use mvv_common::backtrace::BacktraceCell;
use mvv_common::entity::{Amount, AmountFormatError};
use mvv_common::test::{TestDisplayStringOps, TestOptionUnwrap};
//--------------------------------------------------------------------------------------------------


#[derive(Debug, thiserror::Error)]
enum ThisError0 {
    #[error("ThisError0::ErrorFromString0 ( {0} )")]
    FromString0(String),
    #[error("ThisError0::FromString0WirthBt ( {error} )")]
    FromString0WirthBt { error: String, backtrace: BacktraceCell, },
}


#[derive(Debug, thiserror::Error)]
enum ThisError1 {
    #[error("ThisError1::ErrorFromString ( {0} )")]
    // ErrorFromString( #[from] String),
    ErrorFromString(String),
    // #[error("ThisError1::ErrorFromStringWithBt")]
    // ErrorFromString { #[from] error: String, backtrace: BacktraceCell, },

    // #[error("ThisError1::ErrorFromThisError0 ( {0} )")]
    // ErrorFromThisError0( #[from] #[source] ThisError0),
    #[error("ThisError1::ErrorFromThisError0 ( {error} )")]
    // ErrorFromThisError0 { #[from] #[source] error: ThisError0, #[backtrace] backtrace: BacktraceCell },
    ErrorFromThisError0 { #[source] error: ThisError0, backtrace: BacktraceCell },

    #[error("ThisError1::ErrorFromEnvVarError ( {0} )")]
    ErrorFromEnvVarError( #[from] #[source] std::env::VarError),

    #[error("ThisError1::ErrorFromInt")]
    // ErrorFromInt( #[from] i32),
    ErrorFromInt(i32),
    // #[error("ThisError1::ErrorFromIntWithBt")]
    // ErrorFromInt { error: i32, backtrace: BacktraceCell, },

    #[error("ThisError1::ErrorFromAnyhowError")]
    ErrorFromAnyhowError(anyhow::Error),
    #[error("ThisError1::ErrorFromAmountFormatError")]
    ErrorFromAmountFormatError(AmountFormatError),

    #[error("ThisError1::SqlxError")]
    SqlxError { error: sqlx::Error, backtrace: BacktraceCell, },
}


#[test]
fn test_1() {
    println!("\n-----------------------------------------------------------------------");
    let err = ThisError1::ErrorFromString("String123".to_test_string());
    // let err: ThisError1 = ("String123".to_test_string()).into();
    println!("### err 01: {err}");
    println!("### err 01: {err:?}");

    println!("\n-----------------------------------------------------------------------");
    let err0 = ThisError0::FromString0("String345".to_test_string());
    // let err = ThisError1::ErrorFromThisError0(err0);
    let err = ThisError1::ErrorFromThisError0 { error: err0, backtrace: BacktraceCell::capture_backtrace() };
    // let err: ThisError1 = err0.into();
    // let err: ThisError1 = ("String123".to_test_string()).into();
    println!("### err 01: {err}");
    println!("### err 01: {err:?}");

    println!("\n-----------------------------------------------------------------------");
    let err = ThisError1::ErrorFromAmountFormatError(
        Amount::from_str("1234.5 US").err().test_unwrap()
    );
    println!("### err 01: {err}");
    println!("### err 01: {err:?}");
    println!("\n-----------------------------------------------------------------------");

    // assert!(false, "Test error to see console output.");
}
