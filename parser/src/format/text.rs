//! Запись и чтение файлов формата *.txt.

use crate::errors::ParseError;
use crate::models::YPBankTransaction;
use crate::traits::YPBankIO;
use std::io::{Error, Read, Write};

pub struct TxtFormatIO {}

impl YPBankIO for TxtFormatIO {
    fn read<R: Read>(reader: &mut R) -> Result<Vec<YPBankTransaction>, ParseError> {
        let mut buffer = String::new();
        reader.read_to_string(&mut buffer);

        for line in buffer.lines() {
            println!("{}", line);
        }

        Ok(Vec::new())
    }

    fn write<W: Write>(writer: W, records: YPBankTransaction) -> Result<(), ParseError> {
        todo!()
    }
}
