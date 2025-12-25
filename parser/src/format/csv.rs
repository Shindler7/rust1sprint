//! Запись и чтение файлов формата *.csv.

use crate::errors::ParseError;
use crate::format::tools::LineUtils;
use crate::models::{YPBankCsvFormat, YPBankTextFormat};
use crate::traits::YPBankIO;
use std::collections::HashMap;
use std::io::{Error, Read, Write};
use std::iter::zip;

impl YPBankIO for YPBankCsvFormat {
    type DataFormat = YPBankCsvFormat;

    fn read_executor(buffer: String) -> Result<Vec<Self::DataFormat>, ParseError> {
        // Проверим заголовок.
        let mut lines = buffer.lines();
        let title_line = lines
            .next()
            .ok_or_else(|| ParseError::parse_error("Ошибка парсинга заголовка csv", 0, 0))?;

        if !title_line.is_eq(Self::make_title().as_str()) {
            return Err(ParseError::parse_error(
                format!("Некорректный заголовок csv: {}", title_line),
                0,
                0,
            ));
        }

        let title_data = title_line
            .split_csv_line()
            .ok_or_else(|| ParseError::parse_error("Ошибка разбора csv-заголовка", 0, 0))?;

        lines
            .enumerate()
            .map(|(i, line)| Self::parse_data_line(&title_data, line, i + 1))
            .collect()
    }

    /// Добавить запись на основе предоставленного экземпляра `YPBankCsvFormat`.
    fn write_to<W: Write>(mut writer: W, records: &[Self::DataFormat]) -> Result<(), ParseError> {
        writeln!(writer, "{}", Self::make_title())?;
        for record in records {
            writeln!(writer, "{}", Self::makeup_records(record))?;
        }

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
    fn parse_data_line(
        title_data: &Vec<String>,
        line: &str,
        count_line: usize,
    ) -> Result<YPBankCsvFormat, ParseError> {
        let data = match line.split_csv_line() {
            Some(data) => {
                if data.len() != title_data.len() {
                    return Err(ParseError::parse_error(
                        format!("Заголовок не совпадает со строкой: {}", line),
                        count_line,
                        0,
                    ));
                }
                data
            }
            None => {
                return Err(ParseError::parse_error(
                    "Ошибка чтения строки csv",
                    count_line,
                    0,
                ));
            }
        };

        let csv_parse: HashMap<_, _> = title_data
            .iter()
            .zip(data)
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        Ok(YPBankCsvFormat::new_from_map(&csv_parse)?)
    }
}
