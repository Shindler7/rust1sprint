//! Единые трейты библиотеки для поддержки универсальности методов.

use crate::errors::ParseError;
use crate::models::YPBankTransaction;
use std::io::{Read, Write};

pub trait YPBankIO {
    fn read<R: Read>(reader: &mut R) -> Result<Vec<YPBankTransaction>, ParseError>;
    fn write<W: Write>(writer: W, records: YPBankTransaction) -> Result<(), ParseError>;
}
