//! Единые трейты библиотеки для поддержки универсальности методов.

use crate::MAX_SIZE_CSV_TXT_BYTES;
use crate::errors::ParseError;
use std::io::{BufReader, Read, Write};

/// Читает и записывает данные банковских операций в различных форматах.
///
/// Этот типаж определяет общий интерфейс для работы с различными форматами
/// банковских данных (CSV, JSON, XML и т.д.). Каждый формат должен реализовать
/// логику парсинга в [`YPBankIO::read_executor`].
///
/// ## Методы по умолчанию
///
/// Типаж предоставляет реализацию [`YPBankIO::read_from`] по умолчанию, которая:
/// 1. Читает все данные из `reader` в строку
/// 2. Передаёт строку в [`YPBankIO::read_executor`]
/// 3. Проверяет, что результат не пустой
///
/// Другие методы декларативны, и должны быть определены.
pub trait YPBankIO {
    /// Тип данных, представляющий одну запись в формате.
    type DataFormat;

    /// Читает данные из reader и парсит их в вектор записей.
    ///
    /// Этот метод по умолчанию буферизует ввод и делегирует парсинг
    /// методу [`read_executor`]. Переопределите его, если нужна
    /// специальная логика чтения.
    fn read_from<R: Read>(reader: &mut R) -> Result<Vec<Self::DataFormat>, ParseError> {
        let mut buffer = String::new();
        let mut buf_reader = BufReader::new(reader);
        buf_reader
            .read_to_string(&mut buffer)
            .map_err(|e| ParseError::io_error(e, "Ошибка парсинга данных"))?;

        if buffer.len() > MAX_SIZE_CSV_TXT_BYTES {
            return Err(ParseError::lim_exceed(buffer.len(), MAX_SIZE_CSV_TXT_BYTES));
        }

        let transaction = Self::read_executor(buffer)?;
        if transaction.is_empty() {
            return Err(ParseError::EmptyData);
        }

        Ok(transaction)
    }

    /// Парсит строку с данными в вектор записей.
    ///
    /// Этот метод должен быть реализован для каждого формата.
    /// Он содержит специфичную для формата логику парсинга.
    fn read_executor(buffer: String) -> Result<Vec<Self::DataFormat>, ParseError>;

    /// Записывает вектор записей в writer.
    fn write_to<W: Write>(writer: W, records: &[Self::DataFormat]) -> Result<(), ParseError>;
}
