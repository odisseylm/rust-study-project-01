use core::fmt;
use bigdecimal::BigDecimal;



// Default BigDecimal Debug impl shows very unfriendly info
pub fn bd_dbg_fmt(bd: &BigDecimal, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{bd} ({bd:?})")
}
