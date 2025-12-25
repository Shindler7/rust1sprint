//! Запись и чтение файлов бинарного формата.

use crate::errors::ParseError;
use crate::models::YPBankBinFormat;
use crate::traits::YPBankIO;
use std::io::{Read, Write};

impl YPBankIO for YPBankBinFormat {
    type DataFormat = YPBankBinFormat;

    fn read_from<R: Read>(reader: &mut R) -> Result<Vec<Self::DataFormat>, ParseError> {
        todo!()
    }

    fn read_executor(buffer: String) -> Result<Vec<Self::DataFormat>, ParseError> {
        todo!()
    }

    fn write_to<W: Write>(writer: W, records: &[Self::DataFormat]) -> Result<(), ParseError> {
        todo!()
    }
}
