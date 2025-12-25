//! Запись и чтение файлов формата *.csv.

use crate::errors::ParseError;
use crate::format::tools::LineUtils;
use crate::models::{YPBankCsvFormat, YPBankTextFormat};
use crate::traits::YPBankIO;
use std::io::{Error, Read, Write};

impl YPBankIO for YPBankCsvFormat {
    type DataFormat = YPBankCsvFormat;

    fn read_executor(buffer: String) -> Result<Vec<Self::DataFormat>, ParseError> {
        // Проверим заголовок.
        let mut lines = buffer.lines();
        let title_line = match lines.next() {
            Some(line) => line,
            None => {
                return Err(ParseError::parse_error(
                    "Ошибка парсинга заголовка csv",
                    0,
                    0,
                ));
            }
        };
        if !title_line.is_eq(Self::make_title().as_str()) {
            return Err(ParseError::parse_error(
                format!("Некорректный заголовок csv: {}", title_line),
                0,
                0,
            ));
        }

        let mut transaction: Vec<YPBankCsvFormat> = Vec::new();
        for (count, line) in (1..).zip(lines) {
            let line_data = Self::parse_data_line(line, count)?;
            transaction.push(line_data);
        }

        Ok(transaction)
    }

    /// Добавить запись на основе предоставленного экземпляра `YPBankCsvFormat`.
    fn write_to<W: Write>(mut writer: W, records: Self::DataFormat) -> Result<(), ParseError> {
        writer.write(Self::make_title().as_bytes())?;
        Ok(())
    }
}

impl YPBankCsvFormat {
    /// Формирует строку заголовка. Может быть использована при формировании файла, либо при
    /// парсинге, для сопоставления корректности заголовка.
    ///
    /// ## Образец заголовка
    ///
    /// ```plain
    /// TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
    /// ```
    fn make_title() -> String {
        Self::fields().join(",")
    }

    /// Формирует строку записи.
    ///
    /// ## Пример записи
    ///
    /// ```plain
    /// 1000000000000009,DEPOSIT,0,9223372036854775807,1000,1633037400000,FAILURE,"Record number 10"
    /// ```
    fn makeup_records(records: &YPBankCsvFormat) -> String {
        let description = format!(
            "\"{}\"",
            records.description.replace('"', "\"\"") // CSV-экранирование
        );

        [
            records.tx_id.to_string(),
            records.tx_type.to_string(),
            records.from_user_id.to_string(),
            records.to_user_id.to_string(),
            records.amount.to_string(),
            records.timestamp.to_string(),
            records.status.to_string(),
            description,
        ]
        .join(",")
    }

    /// Разбор отдельной строки в CSV.
    fn parse_data_line(line: &str, count_line: usize) -> Result<YPBankCsvFormat, ParseError> {
        let data = match line.split_csv_line() {
            Some(data) => data,
            None => {
                return Err(ParseError::parse_error(
                    "Ошибка чтения заголовка csv",
                    count_line,
                    0,
                ));
            }
        };

        Ok(YPBankCsvFormat)
    }
}
