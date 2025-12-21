//! Запись и чтение файлов бинарного формата.

use crate::errors::ParseError;
use crate::models::YPBankTransaction;
use crate::traits::YPBankIO;
use std::io::{Read, Write};

pub struct BinFormatIO {}

impl YPBankIO for BinFormatIO {
    fn read<R: Read>(reader: &mut R) -> Result<Vec<YPBankTransaction>, ParseError> {
        todo!()
    }

    fn write<W: Write>(writer: W, records: YPBankTransaction) -> Result<(), ParseError> {
        todo!()
    }
}
