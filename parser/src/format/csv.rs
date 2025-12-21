//! Запись и чтение файлов формата *.csv.

use crate::errors::ParseError;
use crate::models::YPBankTransaction;
use crate::traits::YPBankIO;
use std::io::{Error, Read, Write};

pub struct CsvFormatIO {}

impl YPBankIO for CsvFormatIO {
    fn read<R: Read>(reader: &mut R) -> Result<Vec<YPBankTransaction>, ParseError> {
        todo!()
    }

    fn write<W: Write>(writer: W, records: YPBankTransaction) -> Result<(), ParseError> {
        todo!()
    }
}
