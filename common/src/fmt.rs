use core::fmt::{self, Display, Debug};


pub struct OptionDisplayRefWrapper<'a, T: Display> (pub &'a Option<T>);

// Mainly for askama, but can be used in other scenarios too.
//
pub fn display_some<'a,T>(value: &'a Option<T>) -> Result<OptionDisplayRefWrapper<'a, T>, fmt::Error>
    where T: Display {
    Ok(OptionDisplayRefWrapper::<'a>(value))
}

impl<'a, T: Display> Display for OptionDisplayRefWrapper<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            None => <str as Display>::fmt("", f),
            Some(ref value) => <T as Display>::fmt(value, f),
        }
    }
}
impl<'a, T: Debug + Display> Debug for OptionDisplayRefWrapper<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}


// TODO: add tests

#[readonly::make]
pub struct BytesAsBuf<'a> {
    pub array: &'a mut [u8],
    pub pos: usize,
}

impl<'a> BytesAsBuf<'a>  {
    pub fn new(array: &'a mut [u8]) -> Self {
        BytesAsBuf::<'a> { array, pos: 0 }
    }
    pub fn pos(&self) -> usize { self.pos }
}


impl<'a> fmt::Write for BytesAsBuf<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let s_bytes = s.as_bytes();
        if self.pos + s_bytes.len() >= self.array.len() { return Err(fmt::Error) }

        for i in 0..s.len() {
            self.array[self.pos] = s_bytes[i];
            self.pos += 1;
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        if self.pos + 1 >= self.array.len() { return Err(fmt::Error) }

        self.array[self.pos] = c as u8;
        self.pos += 1;
        Ok(())
    }
}
