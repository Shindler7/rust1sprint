//! Единые трейты библиотеки для поддержки универсальности методов.

use crate::errors::ParseError;
use crate::models::YPBankTransaction;
use std::io::{Read, Write};

pub trait YPBankIO {
    type DataFormat;
    fn read_from<R: Read>(reader: &mut R) -> Result<Vec<Self::DataFormat>, ParseError> {
        let mut buffer = String::new();
        reader
            .read_to_string(&mut buffer)
            .map_err(|e| ParseError::io_error(e, "Ошибка парсинга данных"))?;

        let transaction = Self::read_executor(buffer)?;
        if transaction.is_empty() {
            return Err(ParseError::EmptyData);
        }

        Ok(transaction)
    }

    fn read_executor(buffer: String) -> Result<Vec<Self::DataFormat>, ParseError>;
    fn write_to<W: Write>(writer: W, records: &[Self::DataFormat]) -> Result<(), ParseError>;
}
