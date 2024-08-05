
pub mod parse_currency_another_01 {

    #[derive(Debug, thiserror::Error)]
    #[derive(Copy, Clone)]
    #[derive(PartialOrd, PartialEq)]
    pub enum ErrorKind {
        #[error("no currency")]
        NoCurrency,
        #[error("Incorrect currency format")]
        IncorrectCurrencyFormat,
    }

    #[derive(thiserror::Error)]
    #[derive(mvv_error_macro::MyStaticStructError)]
    // #[static_struct_error_internal_type_path_mode(ExternalCratePath)]
    pub struct CurrencyFormatError {
        pub kind: ErrorKind,
        // #[source]
        // pub source: ErrorSource,
        pub backtrace: mvv_common::backtrace::BacktraceCell,
    }

    // #[derive(thiserror::Error)]
    // pub enum ErrorSource {
    //     #[error("No source")]
    //     NoSource,
    // }
}



pub mod parse_amount_another_01 {
    use bigdecimal::ParseBigDecimalError;
    use crate::parse_currency_another_01::CurrencyFormatError;
    use mvv_common::backtrace::BacktraceCell;

    #[derive(Debug, derive_more::Display)]
    #[display(fmt = "Struct123")]
    pub struct Struct123;

    #[derive(Debug, thiserror::Error)]
    #[derive(Copy, Clone)]
    #[derive(PartialOrd, PartialEq)]
    pub enum ErrorKind {
        #[error("No currency in amount")]
        NoCurrency,
        #[error("Incorrect currency format")]
        IncorrectCurrency,
        #[error("Incorrect amount format")]
        IncorrectAmount,
    }

    #[derive(thiserror::Error)]
    #[derive(mvv_error_macro::MyStaticStructError)]
    #[static_struct_error_internal_type_path_mode(ExternalCratePath)]
    pub struct ParseAmountError {
        pub kind: ErrorKind,
        #[source]
        // #[from]
        pub source: ErrorSource,
        pub backtrace: BacktraceCell,
    }

    // It can be generated by macro
    #[derive(mvv_error_macro::MyStaticStructErrorSource)]
    // Full type or short type can be used: ParseAmountError/crate::entity::amount::parse::ParseAmountError
    #[struct_error_type(ParseAmountError)]
    #[static_struct_error_internal_type_path_mode(ExternalCratePath)]
    // #[do_not_generate_std_error]
    pub enum ErrorSource {
        // #[error("No source")]
        NoSource,
        // #[error("Currency format error")]
        // #[error_macro::StaticStructErrorType(ParseAmountError)]
        // #[from_error_kind(IncorrectCurrency)]
        CurrencyFormatError(CurrencyFormatError),
        // for testing
        // CurrencyFormatError22(crate::entity::currency::parse_currency::CurrencyFormatError),
        // #[error("Decimal format error")]
        #[from_error_kind(IncorrectAmount)]
        #[no_source_backtrace]
        ParseBigDecimalError(ParseBigDecimalError),

        SomeWithoutSource,

        // With duplicated types
        Some1FromString(String),
        Some2FromString(String),

        // #[error("Some1FromInt")]
        Some1FromInt(i32),
        // #[error("Some2FromInt")]
        Some2FromInt(i32),

        #[no_source_backtrace]
        #[no_std_error]
        Some3FromSomeStruct(Struct123),

        // #[error("SomeAnyHowError")]
        SomeAnyHowError(anyhow::Error),

        StdErrorError(Box<dyn std::error::Error>),
    }
}


#[test]
fn test_currency_format_error_other() {
    use parse_currency_another_01::*;
    // use mvv_common::backtrace::NewBacktracePolicy;
    // use mvv_common::backtrace::NewBacktracePolicy;
    use anyhow::__private::kind::TraitKind;
    use thiserror::__private::AsDynError;
    use core::any::Any;
    use std::error::Error;

    let err = CurrencyFormatError::with_backtrace(ErrorKind::NoCurrency); // , NewBacktracePolicy::Default);

    // ??? What is it?
    let anyhow_kind = err.anyhow_kind();
    // anyhow::kind is private
    // let anyhow_kind: anyhow::kind::Trait = err.anyhow_kind();
    // let anyhow_kind: &dyn core::any::Any = &err.anyhow_kind();
    println!("anyhow_kind: {:?}", anyhow_kind.type_id());

    let std_err_src: Option<&dyn Error> = err.source();
    // T O D O: add support of it after appearing std::error::Error.provide() in stable build.
    assert!(std_err_src.is_none());

    let std_err: &dyn Error = err.as_dyn_error();
    assert!(std_err.is::<CurrencyFormatError>());
}
