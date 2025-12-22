#[macro_use]
pub mod convert;
pub mod errors;
pub mod format;
pub mod models;
pub mod traits;

use crate::format::bin::BinFormatIO;
use crate::format::csv::CsvFormatIO;
use crate::format::text::TxtFormatIO;
use crate::models::YPBankTransaction;
use crate::traits::YPBankIO;
use errors::ParseError;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};

#[derive(Debug, Copy, Clone)]
pub enum SupportedFormat {
    Csv,
    Text,
    Binary,
}

impl Display for SupportedFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            SupportedFormat::Csv => write!(f, "*.csv (ключ: \"csv\")"),
            SupportedFormat::Text => write!(f, "*.txt (ключи: \"txt\", \"text\")"),
            SupportedFormat::Binary => write!(f, "*.bin (ключи: \"bin\", \"binary\")"),
        }
    }
}

impl SupportedFormat {
    /// Возвращает элемент перечисления `SupportedFormat`, которому соответствует переданное
    /// текстовое значение.
    ///
    /// ## Пример:
    ///
    /// ```
    /// use parser::SupportedFormat;
    ///
    /// let fmt = SupportedFormat::from_str("text");
    /// ```
    ///
    /// При несоответствии, возвращается ошибка `ParseError::UnsupportedFormat`.
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        match s.to_lowercase().as_str() {
            "csv" => Ok(Self::Csv),
            "txt" | "text" => Ok(Self::Text),
            "bin" | "binary" => Ok(Self::Binary),
            _ => Err(ParseError::UnsupportedFormat {
                invalid_format: s.to_string(),
            }),
        }
    }

    /// Прочитать данные в установленном доступном формате.
    ///
    /// ## Args:
    ///
    /// - reader — Экземпляр объекта, поддерживающего трейт Read. Парсер самостоятельно не работает
    ///   с файловой системой.
    ///
    /// ## Returns:
    ///
    /// Вектор экземпляров `YPBankTransaction`, с данными парсинга. При ошибках возвращается
    /// экземпляр соответствующего `ParseError`.
    ///
    /// ## Пример
    ///
    /// ```no_run
    /// use parser::SupportedFormat;
    /// use std::fs::File;
    ///
    /// let mut file = std::fs::File::open("bank.txt").unwrap();
    ///
    /// let s = SupportedFormat::from_str("txt").unwrap();
    /// let result = s.read(&mut file);
    /// ```
    pub fn read<R: Read>(&self, readers: &mut R) -> Result<Vec<YPBankTransaction>, ParseError> {
        match self {
            SupportedFormat::Text => TxtFormatIO::read(readers),
            SupportedFormat::Csv => CsvFormatIO::read(readers),
            SupportedFormat::Binary => BinFormatIO::read(readers),
        }
    }

    /// Быстрый доступ к парсингу данных в формате `csv`.
    ///
    /// ## Применение
    ///
    /// ```no_run
    /// use parser::SupportedFormat;
    ///
    /// let mut file = std::fs::File::open("bank.csv").unwrap();
    /// let data = SupportedFormat::read_csv(&mut file);
    /// ```
    pub fn read_csv<R: Read>(readers: &mut R) -> Result<Vec<YPBankTransaction>, ParseError> {
        let format = SupportedFormat::Csv;
        format.read(readers)
    }

    /// Быстрый доступ к парсингу данных в формате `bin`.
    ///
    /// ## Применение
    ///
    /// ```no_run
    /// use parser::SupportedFormat;
    ///
    /// let mut file = std::fs::File::open("bank.bin").unwrap();
    /// let data = SupportedFormat::read_csv(&mut file);
    /// ```
    pub fn read_bin<R: Read>(readers: &mut R) -> Result<Vec<YPBankTransaction>, ParseError> {
        let format = SupportedFormat::Binary;
        format.read(readers)
    }

    /// Быстрый доступ к парсингу данных в формате `txt`.
    ///
    /// ## Применение
    ///
    /// ```no_run
    /// use parser::SupportedFormat;
    ///
    /// let mut file = std::fs::File::open("bank.txt").unwrap();
    /// let data = SupportedFormat::read_csv(&mut file);
    /// ```
    pub fn read_text<R: Read>(readers: &mut R) -> Result<Vec<YPBankTransaction>, ParseError> {
        let format = SupportedFormat::Text;
        format.read(readers)
    }

    pub fn write<W: Write>(&self, writer: W, records: YPBankTransaction) -> Result<(), ParseError> {
        match self {
            SupportedFormat::Text => TxtFormatIO::write(writer, records),
            SupportedFormat::Csv => CsvFormatIO::write(writer, records),
            SupportedFormat::Binary => BinFormatIO::write(writer, records),
        }
    }
}
