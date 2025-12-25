#[macro_use]
pub mod convert;
pub mod errors;
pub mod format;
pub mod models;
pub mod traits;

use crate::models::{YPBankBinFormat, YPBankCsvFormat, YPBankTextFormat, YPBankTransaction};
use crate::traits::YPBankIO;
use errors::ParseError;
use std::io::{Read, Write};

pub fn read_csv<R: Read>(readers: &mut R) -> Result<Vec<YPBankCsvFormat>, ParseError> {
    YPBankCsvFormat::read_from(readers)
}

pub fn write_csv<W: Write>(writer: &mut W, records: &[YPBankCsvFormat]) -> Result<(), ParseError> {
    YPBankCsvFormat::write_to(writer, records)
}

pub fn read_bin<R: Read>(readers: &mut R) -> Result<Vec<YPBankBinFormat>, ParseError> {
    YPBankBinFormat::read_from(readers)
}

pub fn write_bin<W: Write>(writer: &mut W, records: &[YPBankBinFormat]) -> Result<(), ParseError> {
    YPBankBinFormat::write_to(writer, records)
}

pub fn read_text<R: Read>(readers: &mut R) -> Result<Vec<YPBankTextFormat>, ParseError> {
    YPBankTextFormat::read_from(readers)
}

pub fn write_text<R: Write>(
    writer: &mut R,
    records: &[YPBankTextFormat],
) -> Result<(), ParseError> {
    YPBankTextFormat::write_to(writer, records)
}
