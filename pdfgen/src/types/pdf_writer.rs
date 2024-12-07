//! Implementation of the [`PdfWriter`] wrapper.

use super::{
    constants,
    hierarchy::{
        cross_reference_table::CrossReferenceTable,
        primitives::{obj_ref::ObjRef, object::Object},
    },
};
use std::io::{self, Write};

/// A wrapper around any type that implements [`Write`], adding pdf specific functionality to keep a
/// clear and consistent CrossReferenceTable state
pub struct PdfWriter<W: Write> {
    /// Inner member, representing a type that implements [`Write`].
    inner: W,
    /// Current byte offset from the top of document, representing the current position of the `cursor`.
    current_offset: usize,
    /// CrossReferenceTable member, representing the current state of the cross_reference_table
    /// for the document
    cross_reference_table: CrossReferenceTable,
}

impl<W: Write> PdfWriter<W> {
    /// The PDF file begins with the 5 characters “%PDF–X.X” and byte offsets shall be calculated
    /// from the PERCENT SIGN.
    const PDF_HEADER: &[u8] = b"%PDF-2.0";
    /// The last line of the file shall contain only the end-of-file marker, %%EOF
    const EOF_MARKER: &[u8] = b"%%EOF";
    /// Marker indicating end of an object section
    const END_OBJ_MARKER: &[u8] = b"endobj";

    /// Creates a new [`PdfWriter`] instance.
    pub fn new(inner: W) -> Self {
        PdfWriter {
            inner,
            current_offset: 0,
            cross_reference_table: CrossReferenceTable::default(),
        }
    }

    /// Write the PDF documents header marker updating the `cursor`s byte offset with the number of
    /// bytes written
    pub fn write_header(&mut self) -> Result<(), io::Error> {
        // Delegate the actual writing to the inner writer incrementing the current_offset to
        // reflect current `cursor` position.
        self.current_offset += self.inner.write(Self::PDF_HEADER)?;
        self.current_offset += self.inner.write(constants::NL_MARKER)?;

        Ok(())
    }

    /// Writes the object start marker(`X X obj`), following with the structured data of the object
    /// itself, finalizing with object end marker(`endobj`), ensuring correct CrossReferenceTable
    /// and cursor update.
    pub fn write_object(&mut self, obj: &impl Object, obj_ref: ObjRef) -> Result<(), io::Error> {
        // Save the objects byte offset in the CrossReferenceTable.
        self.cross_reference_table.add_object(self.current_offset);

        // X Y obj\n
        self.current_offset += obj_ref.write_def(&mut self.inner)?;
        self.current_offset += self.inner.write(constants::NL_MARKER)?;

        // Delegate the actual writing to the inner writer.
        self.current_offset += obj.write(&mut self.inner)?;
        self.current_offset += self.inner.write(constants::NL_MARKER)?;

        // endobj\n
        self.current_offset += self.inner.write(Self::END_OBJ_MARKER)?;
        self.current_offset += self.inner.write(constants::NL_MARKER)?;

        Ok(())
    }

    /// Writes the cross reference table contents
    pub fn write_crt(&mut self) -> Result<(), io::Error> {
        self.cross_reference_table.write(&mut self.inner)?;

        Ok(())
    }

        Ok(())
    }

    /// Write the PDF documents EOF marker.
    pub fn write_eof(&mut self) -> Result<(), io::Error> {
        // Delegate the actual writing to the inner writer.
        self.inner.write_all(Self::EOF_MARKER)
    }
}

#[cfg(test)]
mod tests {
    use crate::{types::hierarchy::primitives::obj_ref::ObjRef, PdfWriter};

    use super::Object;

    struct Dummy;
    impl Object for Dummy {
        fn write(&self, writer: &mut impl std::io::Write) -> Result<usize, std::io::Error> {
            writer.write(b"FirstLine\nSecondLine")
        }
    }

    #[test]
    fn write_header() {
        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer);

        pdf_writer.write_header().unwrap();

        let output = String::from_utf8(writer).unwrap();

        insta::assert_snapshot!(
            output,
            @"%PDF-2.0"
        );
    }

    #[test]
    fn write_eof() {
        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer);

        pdf_writer.write_eof().unwrap();

        let output = String::from_utf8(writer).unwrap();

        insta::assert_snapshot!(
            output,
            @"%%EOF"
        );
    }

    #[test]
    fn write_object() {
        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer);

        pdf_writer.write_object(&Dummy, ObjRef::from(17)).unwrap();

        let output = String::from_utf8(writer).unwrap();

        insta::assert_snapshot!(
            output,
            @r"
        17 0 obj
        FirstLine
        SecondLine
        endobj
        "
        );
    }

    #[test]
    fn write_crt() {
        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer);

        pdf_writer.write_header().unwrap();
        pdf_writer.write_object(&Dummy, ObjRef::from(1)).unwrap();
        pdf_writer.write_object(&Dummy, ObjRef::from(2)).unwrap();
        pdf_writer.write_object(&Dummy, ObjRef::from(3)).unwrap();
        pdf_writer.write_object(&Dummy, ObjRef::from(4)).unwrap();
        pdf_writer.write_crt().unwrap();
        pdf_writer.write_eof().unwrap();

        let output = String::from_utf8(writer).unwrap();

        insta::assert_snapshot!(
            output,
            @r"
        %PDF-2.0
        1 0 obj
        FirstLine
        SecondLine
        endobj
        2 0 obj
        FirstLine
        SecondLine
        endobj
        3 0 obj
        FirstLine
        SecondLine
        endobj
        4 0 obj
        FirstLine
        SecondLine
        endobj
        xref
        0 4
        0000000009 00000 n 
        0000000045 00000 n 
        0000000081 00000 n 
        0000000117 00000 n 
        %%EOF
        "
        );
    }
}
