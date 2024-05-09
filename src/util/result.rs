
use core::fmt;


// !!! Impossible in rust !!!
// Error: only traits defined in the current crate can be implemented for types defined out-side of the crate
//
// impl fmt::Display for Result<Currency, Fuck> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}{}{}", self.0[0] as char, self.0[1] as char, self.0[2] as char)
//     }
// }


pub struct PrintableResult<'a, T: fmt::Display, E: fmt::Display>(pub &'a Result<T, E>);


#[inline]
// fn as_printable<'a, T: fmt::Display, E: fmt::Display>(r: &'a Result<T, E>) -> PrintableResult<'a, T, E> {
pub fn as_printable<T: fmt::Display, E: fmt::Display>(r: &Result<T, E>) -> PrintableResult<T, E> {
    return PrintableResult(r);
}


impl<'a, T: fmt::Display, E: fmt::Display> fmt::Display for PrintableResult<'a, T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Ok(ok)   => { write!(f, "{}", ok)  }
            Err(err) => { write!(f, "{}", err) }
        }
    }
}


#[inline]
pub fn as_printable_ptr<'a, T: fmt::Display, E: fmt::Display>(r: &'a Result<T, E>) -> & 'a dyn fmt::Display {
    return match r {
        Ok(ok)   => { ok  }
        Err(err) => { err }
    }
}
