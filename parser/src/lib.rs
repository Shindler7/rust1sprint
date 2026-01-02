//! Библиотека, обеспечивающая парсинг и сериализацию форматов.
//! Реализации парсера используют абстракции, предоставляемые стандартной библиотекой языка, чтобы
//! обеспечить гибкость кода.
//!
//! "Яндекс Практикум", "Rust для действующих разработчиков", 2025.
//!
//! ## Быстрый старт
//!
//! ```
//! use parser::models::{TxStatus, TxType, YPBankTextFormat};
//! use parser::utils::get_timestamp;
//! use std::io;
//! use parser::write_text;
//!
//! let timestamp = get_timestamp();
//!
//! let yp_txt = vec![
//!     YPBankTextFormat {
//!         tx_id: 1000000000000982,
//!         tx_type: TxType::Transfer,
//!         from_user_id: 9223372036854775807,
//!         to_user_id: 29918172560165698,
//!         amount: 98300,
//!         timestamp,
//!         status: TxStatus::Pending,
//!         description: "Record number 982".to_string()
//!     }
//! ];
//!
//! let mut stdout = io::stdout();
//!
//! write_text(&mut stdout, &yp_txt).unwrap();
//! ```
#![warn(missing_docs)]

#[macro_use]
pub mod errors;
pub mod format;
pub mod models;
pub mod traits;
pub mod utils;

use crate::models::{YPBankBinFormat, YPBankCsvFormat, YPBankTextFormat, YPBankTransaction};
use crate::traits::YPBankIO;
use errors::ParseError;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};

/// Значение MiB.
const MI_B: usize = 1_048_576;
/// Максимальный размер входящего потока для бинарного формата.
pub const MAX_SIZE_BIN_BYTES: usize = 8 * MI_B;
/// Максимальный размер входящего потока для CSV и TXT.
pub const MAX_SIZE_CSV_TXT_BYTES: usize = 4 * MI_B;

/// Трейт предоставляющий дополнительные методы для векторов, содержащих структуры данных
/// в форматах обработки файлов.
pub trait Transaction {
    /// Конвертировать набор структур [`YPBankCsvFormat`], [`YPBankBinFormat`],
    /// [`YPBankTextFormat`] в универсальную [`YPBankTransaction`].
    fn convert_to_transaction(self) -> Result<Vec<YPBankTransaction>, ParseError>;
}

impl<T> Transaction for Vec<T>
where
    YPBankTransaction: TryFrom<T, Error = ParseError>,
{
    /// Конвертировать векторы с типами [`YPBankBinFormat`], [`YPBankCsvFormat`],
    /// [`YPBankTextFormat`] в вектор с типом [`YPBankTransaction`].
    ///
    /// ## Пример
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use parser::read_text;
    /// use crate::parser::Transaction;
    ///
    /// let mut file = File::open("data.txt").unwrap();
    /// let data = read_text(&mut file).unwrap().convert_to_transaction();
    /// ```
    fn convert_to_transaction(self) -> Result<Vec<YPBankTransaction>, ParseError> {
        self.into_iter().map(YPBankTransaction::try_from).collect()
    }
}

/// Считывает данные в формате `csv`.
///
/// Обёртка для низкоуровневого метода [`YPBankCsvFormat::read_from`].
///
/// ## Пример
///
/// ```no_run
/// use std::fs::File;
/// use parser::read_csv;
///
/// let mut file = File::open("data.csv").unwrap();
/// let data = read_csv(&mut file);
/// ```
///
/// ## Returns
///
/// Вектор с элементами [`YPBankCsvFormat`] при успешном разборе, либо [`ParseError`] в случае
/// ошибки.
pub fn read_csv<R: Read>(readers: &mut R) -> Result<Vec<YPBankCsvFormat>, ParseError> {
    YPBankCsvFormat::read_from(readers)
}

/// Записывает данные в формате `csv`.
///
/// Обёртка для низкоуровневого метода [`YPBankCsvFormat::write_to`].
///
/// ## Пример
///
/// ```no_run
/// use std::fs::File;
/// use parser::models::{TxStatus, TxType, YPBankCsvFormat};
/// use parser::write_csv;
/// use std::time::SystemTime;
///
/// let timestamp = SystemTime::now()
///     .duration_since(SystemTime::UNIX_EPOCH)
///     .unwrap()
///     .as_secs();
///
/// let data = vec![
///     YPBankCsvFormat {
///         tx_id: 1000000000000863,
///         tx_type: TxType::Withdrawal,
///         from_user_id: 536976054377442000,
///         to_user_id: 0,
///         amount: 86400,
///         timestamp,
///         status: TxStatus::Success,
///         description: "Record number 864".to_string(),
///     },
/// ];
///
/// let mut file = File::create("data.csv").unwrap();
/// let data = write_csv(&mut file, &data);
/// ```
///
/// ## Returns
///
/// При успешной записи пустой `Result`, и [`ParseError`] в случае ошибки.
pub fn write_csv<W: Write>(writer: &mut W, records: &[YPBankCsvFormat]) -> Result<(), ParseError> {
    YPBankCsvFormat::write_to(writer, records)
}

/// Считывает данные в бинарном формате (`bin`).
///
/// Обёртка для низкоуровневого метода [`YPBankBinFormat::read_from`].
///
/// ## Пример
///
/// ```no_run
/// use std::fs::File;
/// use parser::read_bin;
///
/// let mut file = File::open("data.bin").unwrap();
/// let data = read_bin(&mut file);
/// ```
///
/// ## Returns
///
/// Вектор с элементами [`YPBankBinFormat`] при успешном разборе, либо [`ParseError`] в случае
/// ошибки.
pub fn read_bin<R: Read>(readers: &mut R) -> Result<Vec<YPBankBinFormat>, ParseError> {
    YPBankBinFormat::read_from(readers)
}

/// Записывает данные в бинарном формате (`bin`).
///
/// Обёртка для низкоуровневого метода [`YPBankBinFormat::write_to`].
///
/// ## Пример
///
/// ```no_run
/// use std::fs::File;
/// use parser::models::{TxStatus, TxType, YPBankBinFormat};
/// use parser::write_bin;
/// use std::time::SystemTime;
///
/// let timestamp = SystemTime::now()
///     .duration_since(SystemTime::UNIX_EPOCH)
///     .unwrap()
///     .as_secs();
///
/// let data = vec![
///     YPBankBinFormat {
///         tx_id: 1000000000000863,
///         tx_type: TxType::Deposit,
///         from_user_id: 0,
///         to_user_id: 8508422095236124061,
///         amount: 92600,
///         timestamp,
///         status: TxStatus::Success,
///         desc_len: 0,
///         description: None,
///     },
/// ];
///
/// let mut file = File::create("data.bin").unwrap();
/// let data = write_bin(&mut file, &data);
/// ```
///
/// ## Returns
///
/// При успешной записи пустой `Result`, и [`ParseError`] в случае ошибки.
pub fn write_bin<W: Write>(writer: &mut W, records: &[YPBankBinFormat]) -> Result<(), ParseError> {
    YPBankBinFormat::write_to(writer, records)
}

/// Считывает данные в `txt`-формате.
///
/// Обёртка для низкоуровневого метода [`YPBankTextFormat::read_from`].
///
/// ## Пример
///
/// ```no_run
/// use std::fs::File;
/// use parser::read_text;
///
/// let mut file = File::open("data.txt").unwrap();
/// let data = read_text(&mut file);
/// ```
///
/// ## Returns
///
/// Вектор с элементами [`YPBankTextFormat`] при успешном разборе, либо [`ParseError`] в случае
/// ошибки.
pub fn read_text<R: Read>(readers: &mut R) -> Result<Vec<YPBankTextFormat>, ParseError> {
    YPBankTextFormat::read_from(readers)
}

/// Записывает данные в текстовом формате (`txt`).
///
/// Обёртка для низкоуровневого метода [`YPBankTextFormat::write_to`].
///
/// ## Пример
///
/// ```no_run
/// use std::fs::File;
/// use parser::models::{TxStatus, TxType, YPBankTextFormat};
/// use std::time::SystemTime;
/// use parser::write_text;
///
/// let timestamp = SystemTime::now()
///     .duration_since(SystemTime::UNIX_EPOCH)
///     .unwrap()
///     .as_secs();
///
/// let data = vec![
///     YPBankTextFormat {
///         tx_id: 1000000000000863,
///         tx_type: TxType::Deposit,
///         from_user_id: 0,
///         to_user_id: 9223372036854775807,
///         amount: 100000,
///         timestamp,
///         status: TxStatus::Failure,
///         description: "Record number 1000".to_string(),
///     },
/// ];
///
/// let mut file = File::create("data.txt").unwrap();
/// let data = write_text(&mut file, &data);
/// ```
///
/// ## Returns
///
/// При успешной записи пустой `Result`, и [`ParseError`] в случае ошибки.
pub fn write_text<R: Write>(
    writer: &mut R,
    records: &[YPBankTextFormat],
) -> Result<(), ParseError> {
    YPBankTextFormat::write_to(writer, records)
}

/// Поддерживаемые форматы данных, используемые для чтения и записи в случаях, когда возможна
/// работа с двумя разными типами (например, `csv` и `txt`): конвертация, сравнение.
///
/// При работе с одним типом необходимо использовать прямые методы. Например, [`read_text`] для
/// чтения в текстовом формате, [`write_bin`] — для записи в бинарном формате, и так далее.
pub enum YPFormatSupported {
    /// Текстовый формат (`*.txt`): человекочитаемый формат, хранящий данные в виде обычного текста.
    Text,

    /// CSV-формат (`*.csv`): табличный текстовый формат, где каждая строка представляет собой
    /// отдельную запись, а поля разделяются запятыми.
    Csv,

    /// Бинарный формат (`*.bin`): компактный, нечитаемый человеком формат, хранящий данные
    /// в виде байтов.
    Binary,
}

impl Display for YPFormatSupported {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            YPFormatSupported::Text => write!(f, "txt"),
            YPFormatSupported::Csv => write!(f, "csv"),
            YPFormatSupported::Binary => write!(f, "bin"),
        }
    }
}

impl YPFormatSupported {
    /// Преобразование вектора элементов в доступных форматах (например, [`YPBankTextFormat`],
    /// [`YPBankCsvFormat`], [`YPBankBinFormat`], в универсальный тип: [`YPBankTransaction`].
    ///
    /// Использовать универсальный формат целесообразно, если требуется проведение операций между
    /// разными типами. Например, их сравнение, конвертация и так далее.
    ///
    /// ## Пример
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use parser::YPFormatSupported;
    ///
    /// let mut file = File::open("file.txt").unwrap();
    /// let transactions_txt = YPFormatSupported::Text.to_transaction(&mut file);
    /// let mut file2 = File::open("file.bin").unwrap();
    /// let transaction_bin = YPFormatSupported::Binary.to_transaction(&mut file2);
    ///
    /// let text_vec = &transactions_txt.unwrap();
    /// let bin_vec = &transaction_bin.unwrap();
    ///
    /// let first_txt = text_vec.first();
    /// let first_bin = bin_vec.first();
    ///
    /// assert_eq!(first_txt, first_bin)
    /// ```
    ///
    /// ## Returns
    ///
    /// В случае успеха возвращён будет набор элементов [`YPBankTransaction`] в векторе. При ошибке
    /// [`ParseError`].
    pub fn to_transaction<R: Read>(
        &self,
        readers: &mut R,
    ) -> Result<Vec<YPBankTransaction>, ParseError> {
        match self {
            YPFormatSupported::Text => read_text(readers)?.convert_to_transaction(),
            YPFormatSupported::Csv => read_csv(readers)?.convert_to_transaction(),
            YPFormatSupported::Binary => read_bin(readers)?.convert_to_transaction(),
        }
    }

    /// Преобразование вектора с элементами универсального типа [`YPBankTransaction`] в вектор
    /// с типами выбранного формата. Например, [`YPBankTextFormat`], [`YPBankCsvFormat`],
    /// [`YPBankBinFormat`].
    ///
    /// Необходимо переводить из универсального формата в оконечный, если требуется выполнить
    /// операцию ввода-вывода. [`YPBankTransaction`] не умеет читать и сохранять данные в других
    /// форматах, его можно использовать только для хранения, сравнения, конвертации и т.п.
    ///
    /// ## Пример
    ///
    /// ```
    /// use std::io;
    /// use parser::models::{TxStatus, TxType, YPBankTransaction};
    /// use parser::YPFormatSupported;
    /// use std::time::SystemTime;
    ///
    /// let timestamp = SystemTime::now()
    ///     .duration_since(SystemTime::UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_secs();
    ///
    /// let yp_universal = vec![
    ///     YPBankTransaction {
    ///         tx_id: 1000000000000982,
    ///         tx_type: TxType::Transfer,
    ///         from_user_id: 9223372036854775807,
    ///         to_user_id: 29918172560165698,
    ///         amount: 98300,
    ///         timestamp,
    ///         status: TxStatus::Pending,
    ///         description: Some("Record number 982".to_string())
    ///     }
    /// ];
    ///
    /// let mut stdout = io::stdout();
    ///
    /// YPFormatSupported::Text.convert_transactions(&mut stdout, &yp_universal);
    /// YPFormatSupported::Binary.convert_transactions(&mut stdout, &yp_universal);
    /// ```
    pub fn convert_transactions<W: Write>(
        &self,
        writer: &mut W,
        transaction: &[YPBankTransaction],
    ) -> Result<(), ParseError> {
        match self {
            YPFormatSupported::Text => {
                let transformed = transaction
                    .iter()
                    .cloned()
                    .map(|bt| bt.try_into())
                    .collect::<Result<Vec<YPBankTextFormat>, ParseError>>()?;

                write_text(writer, &transformed)?;
                Ok(())
            }
            YPFormatSupported::Binary => {
                let transformed = transaction
                    .iter()
                    .cloned()
                    .map(|bt| bt.try_into())
                    .collect::<Result<Vec<YPBankBinFormat>, ParseError>>()?;

                write_bin(writer, &transformed)?;
                Ok(())
            }

            YPFormatSupported::Csv => {
                let transformed = transaction
                    .iter()
                    .cloned()
                    .map(|bt| bt.try_into())
                    .collect::<Result<Vec<YPBankCsvFormat>, ParseError>>()?;

                write_csv(writer, &transformed)?;
                Ok(())
            }
        }
    }
}
