use core::str::FromStr;
use assertables:: { assert_starts_with, assert_starts_with_as_result };
use bigdecimal::BigDecimal;
use mvv_common::entity::amount::{Amount, ops::AmountOpsError, ops::ErrorKind as OpsErrorKind };
// use project01::entity::amount::ops;
use mvv_common::test::{ TestOptionUnwrap, TestResultUnwrap, TestSringOps };


#[test]
fn test_big_decimal_eq() {
    {
        let amount1 = BigDecimal::from_str("111.111").test_unwrap();
        let amount2 = BigDecimal::from_str("111.111").test_unwrap();
        assert_eq!(amount1, amount2);
    }
    {
        assert_eq!(BigDecimal::from_str("111.111").test_unwrap(), BigDecimal::from_str("111.111").test_unwrap());
        assert_eq!(BigDecimal::from_str("111.111000").test_unwrap(), BigDecimal::from_str("111.111").test_unwrap());
        assert_eq!(BigDecimal::from_str("111.111").test_unwrap(), BigDecimal::from_str("111.111000").test_unwrap());
    }
    {
        assert_ne!(BigDecimal::from_str("111.1").test_unwrap(), BigDecimal::from_str("111.2").test_unwrap());
    }
}


#[test]
fn test_add_amounts() {

    let amount1 = Amount::from_str("123.45  BRL").test_unwrap();
    let amount2 = Amount::from_str("111.111 BRL").test_unwrap();
    assert_eq!((&amount1 + &amount2).test_unwrap(), Amount::from_str("234.561 BRL").test_unwrap());

    // to test still valid ref (not moved)
    assert_eq!((amount1 + amount2).test_unwrap(), Amount::from_str("234.561 BRL").test_unwrap());

    // now moved - compilation error
    // assert_eq!((amount1 + amount2).test_unwrap(), Amount::from_str("234.561 BRL").test_unwrap());
}

#[test]
#[should_panic(expected = "AmountOpsError { kind: DifferentCurrencies(Currency(BRL), Currency(EUR)),")]
fn test_add_amounts_with_different_currencies() {

    let amount1 = Amount::from_str("123.45  BRL").test_unwrap();
    let amount2 = Amount::from_str("111.111 EUR").test_unwrap();
    assert_eq!((&amount1 + &amount2).test_unwrap(), Amount::from_str("234.561 BRL").test_unwrap());
}

#[test]
fn test_add_amounts_with_different_currencies_to_see_currencies() {
    use core::fmt::Write;

    let amount1 = Amount::from_str("123.45  BRL").test_unwrap();
    let amount2 = Amount::from_str("111.111 EUR").test_unwrap();
    let res = amount1 + amount2;

    let err: AmountOpsError = res.err().test_unwrap();

    let err_as_str = err.to_test_string();
    // println!("{}", err_as_str);
    assert_eq!(err_as_str, "AmountOpsError { Different currencies (BRL,EUR) }");

    let mut str_buf = String::new();
    write!(str_buf, "{}", err).test_unwrap();
    assert_eq!(str_buf, "AmountOpsError { Different currencies (BRL,EUR) }");

    let mut str_buf = String::new();
    write!(str_buf, "{:?}", err).test_unwrap();
    assert_starts_with!(str_buf, "AmountOpsError { kind: DifferentCurrencies(Currency(BRL), Currency(EUR)),");
}


#[test]
fn test_sub_amounts() {

    let amount1 = Amount::from_str("123.456  BRL").test_unwrap();
    let amount2 = Amount::from_str("111.11   BRL").test_unwrap();
    assert_eq!((&amount1 - &amount2).test_unwrap(), Amount::from_str("12.346 BRL").test_unwrap());

    // to test still valid ref (not moved)
    assert_eq!((amount1 - amount2).test_unwrap(), Amount::from_str(" 12.346 BRL ").test_unwrap());

    // now moved - compilation error
    // assert_eq!((amount1 - amount2).test_unwrap(), Amount::from_str("234.561 BRL").test_unwrap());
}


#[test]
#[should_panic(expected = "AmountOpsError { kind: DifferentCurrencies(Currency(BRL), Currency(EUR)),")]
fn test_sub_amounts_with_different_currencies() {

    let amount1 = Amount::from_str("123.45  BRL").test_unwrap();
    let amount2 = Amount::from_str("111.111 EUR").test_unwrap();
    assert_eq!((&amount1 - &amount2).test_unwrap(), Amount::from_str("234.561 BRL").test_unwrap());
}


#[test]
fn test_mul_amount_by_bd() {

    let amount = Amount::from_str("111.11   BRL").test_unwrap();
    let k = BigDecimal::from_str("2").test_unwrap();
    assert_eq!(&amount * &k, Amount::from_str("222.22 BRL").test_unwrap());

    // to test still valid ref (not moved)
    assert_eq!(amount * k, Amount::from_str("222.22 BRL").test_unwrap());

    // now moved - compilation error
    // assert_eq!(amount * k, Amount::from_str("222.22 BRL").test_unwrap());
}

#[test]
fn test_mul_bd_by_amount() {

    let amount = Amount::from_str("111.11   BRL").test_unwrap();
    let k = BigDecimal::from_str("2").test_unwrap();
    assert_eq!(&k * &amount, Amount::from_str("222.22 BRL").test_unwrap());

    // to test still valid ref (not moved)
    assert_eq!(k * amount, Amount::from_str("222.22 BRL").test_unwrap());

    // now moved - compilation error
    // assert_eq!(k * amount, Amount::from_str("222.22 BRL").test_unwrap());
}


#[test]
fn test_div_amount_by_bd() {

    let amount = Amount::from_str("222.22   BRL").test_unwrap();
    let k = BigDecimal::from_str("2").test_unwrap();
    assert_eq!((&amount / &k).test_unwrap(), Amount::from_str("111.11 BRL").test_unwrap());

    // to test still valid ref (not moved)
    assert_eq!((amount / k).test_unwrap(), Amount::from_str("111.11 BRL").test_unwrap());

    // now moved - compilation error
    // assert_eq!((amount / k).test_unwrap(), Amount::from_str("111.11 BRL").test_unwrap());
}


#[test]
#[should_panic(expected = "AmountOpsError { kind: DivideByZero")]
fn test_div_amount_by_zero() {
    let amount = Amount::from_str("222.22   BRL").test_unwrap();
    let k = BigDecimal::from_str("0").test_unwrap();
    assert_eq!((&amount / &k).err().test_unwrap().kind, OpsErrorKind::DivideByZero);

    // to test still valid ref (not moved)
    assert_eq!((amount / k).test_unwrap(), Amount::from_str("111.11 BRL").test_unwrap());

    // now moved - compilation error
    // assert_eq!((amount / k).test_unwrap(), Amount::from_str("111.11 BRL").test_unwrap());
}


#[test]
fn test_div_expects_endless_fraction() { // BUT no endless fraction (comparing with java).

    let amount = Amount::from_str("1 BRL").test_unwrap();
    let k = BigDecimal::from_str("3").test_unwrap();

    // Very strange?!
    // It does NOT throw error in such case

    assert_eq!((&amount / &k).test_unwrap(), Amount::from_str("0.3333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333 BRL").test_unwrap());

    // to test still valid ref (not moved)
    assert_eq!((amount / k).test_unwrap(), Amount::from_str("0.3333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333 BRL").test_unwrap());

    // now moved - compilation error
    // assert_eq!((amount / k).test_unwrap(), Amount::from_str(""0.3333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333 BRL"").test_unwrap());
}
