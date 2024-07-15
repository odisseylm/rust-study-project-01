

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


impl<'a> core::fmt::Write for BytesAsBuf<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let s_bytes = s.as_bytes();
        if self.pos + s_bytes.len() >= self.array.len() { return Err(core::fmt::Error) }

        for i in 0..s.len() {
            self.array[self.pos] = s_bytes[i];
            self.pos += 1;
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        if self.pos + 1 >= self.array.len() { return Err(core::fmt::Error) }

        self.array[self.pos] = c as u8;
        self.pos += 1;
        Ok(())
    }
}
