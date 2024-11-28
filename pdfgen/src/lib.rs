#![forbid(unsafe_code)]

//! `pdfgen` is a low-level library that offers fine-grained control over PDF syntax and
//! PDF file generation.

use std::io::{self, Error, Write};

use types::dictionary::{Key, WriteDictValue};

mod types;

/// This represents one cohesive PDF document that can contain multiple pages of content.
#[derive(Default)]
pub struct Document;

impl Document {
    /// The PDF file begins with the 5 characters “%PDF–X.X” and byte offsets shall be calculated
    /// from the PERCENT SIGN.
    const PDF_HEADER: &[u8] = b"%PDF-2.0";
    /// The last line of the file shall contain only the end-of-file marker, %%EOF
    const EOF_MARKER: &[u8] = b"%%EOF";

    /// Write the PDF header and return number of written bytes.
    fn write_header(&self, writer: &mut impl Write) -> Result<usize, io::Error> {
        writer.write(Self::PDF_HEADER)
    }

    /// Write the PDF documents EOF.
    fn write_eof(&self, writer: &mut impl Write) -> Result<(), io::Error> {
        writer.write_all(Self::EOF_MARKER)
    }

    /// Write the PDF contents into the provided writer.
    pub fn write(&self, writer: &mut impl Write) -> Result<(), io::Error> {
        let _bytes_written = self.write_header(writer)?;
        self.write_eof(writer)?;

        Ok(())
    }
}

trait WriteExt {
    fn write_newline(&mut self) -> Result<usize, std::io::Error>;
    fn write_dict_entry(&mut self, key: Key, value: &impl WriteDictValue) -> Result<usize, Error>;
}

impl<T> WriteExt for T
where
    T: Write,
{
    fn write_newline(&mut self) -> Result<usize, std::io::Error> {
        self.write(b"\n")
    }

    fn write_dict_entry(
        &mut self,
        key: Key,
        value: &impl WriteDictValue,
    ) -> Result<usize, std::io::Error> {
        let mut written = self.write(b"/")?;
        written += key.write(self)?;
        written += self.write(b" ")?;
        written += value.write(self)?;

        Ok(written)
    }
}
