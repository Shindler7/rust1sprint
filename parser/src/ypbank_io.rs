//! Методы ввода-вывода для данных о банковских операциях.

use crate::ypbank_base::{YPBankBinFormat, YPBankCsvFormat, YPBankTextFormat};
use std::io::Read;

pub trait YPBankIO: Sized {
    type Error;
    fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl YPBankIO for YPBankTextFormat {}

impl YPBankIO for YPBankCsvFormat {}

impl YPBankIO for YPBankBinFormat {}
