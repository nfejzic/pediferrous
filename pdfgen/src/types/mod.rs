use std::io::{Error, Write};

pub mod hierarchy;

pub trait WriteDictValue {
    #[allow(dead_code)]
    fn write(&self, writer: &mut impl Write) -> Result<usize, Error>;
}
