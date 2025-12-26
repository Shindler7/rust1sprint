//! Запись и чтение файлов формата *.csv.

use crate::errors::ParseError;
use crate::format::tools::LineUtils;
use crate::models::YPBankCsvFormat;
use crate::traits::YPBankIO;
use std::collections::HashMap;
use std::io::Write;

impl YPBankIO for YPBankCsvFormat {
    type DataFormat = YPBankCsvFormat;

    fn read_executor(buffer: String) -> Result<Vec<Self::DataFormat>, ParseError> {
        // Проверим заголовок.
        let mut lines = buffer.lines();
        let title_line = lines
            .next()
            .ok_or_else(|| ParseError::parse_err("Ошибка парсинга заголовка csv", 0, 0))?;

        if !title_line.is_eq(Self::make_title().as_str()) {
            return Err(ParseError::parse_err(
                format!("Некорректный заголовок csv: {}", title_line),
                0,
                0,
            ));
        }

        let title_data = title_line
            .split_csv_line()
            .ok_or_else(|| ParseError::parse_err("Ошибка разбора csv-заголовка", 0, 0))?;

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
        title_data: &[String],
        line: &str,
        count_line: usize,
    ) -> Result<YPBankCsvFormat, ParseError> {
        let data = match line.split_csv_line() {
            Some(data) => {
                if data.len() != title_data.len() {
                    return Err(ParseError::parse_err(
                        format!("Заголовок не совпадает со строкой: {}", line),
                        count_line,
                        0,
                    ));
                }
                data
            }
            None => {
                return Err(ParseError::parse_err(
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

        YPBankCsvFormat::new_from_map(&csv_parse)
    }
}

#[cfg(test)]
mod csv_tests {
    use crate::errors::ParseError;
    use crate::models::{TxStatus, TxType, YPBankCsvFormat};
    use crate::traits::YPBankIO;

    fn create_test_csv_record() -> YPBankCsvFormat {
        YPBankCsvFormat {
            tx_id: 123456789,
            tx_type: TxType::Transfer,
            from_user_id: 1001,
            to_user_id: 1002,
            amount: 50000,
            timestamp: 1633046400,
            status: TxStatus::Success,
            description: "Test transaction".to_string(),
        }
    }

    fn create_deposit_csv_record() -> YPBankCsvFormat {
        YPBankCsvFormat {
            tx_id: 987654321,
            tx_type: TxType::Deposit,
            from_user_id: 0,
            to_user_id: 1003,
            amount: 100000,
            timestamp: 1633046401,
            status: TxStatus::Pending,
            description: String::new(),
        }
    }

    fn create_withdrawal_csv_record() -> YPBankCsvFormat {
        YPBankCsvFormat {
            tx_id: 555555555,
            tx_type: TxType::Withdrawal,
            from_user_id: 1004,
            to_user_id: 0,
            amount: 25000,
            timestamp: 1633046402,
            status: TxStatus::Failure,
            description: "Withdrawal".to_string(),
        }
    }

    #[test]
    fn test_make_title() {
        // Act
        let title = YPBankCsvFormat::make_title();

        // Assert
        assert_eq!(
            title,
            "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION"
        );
    }

    #[test]
    fn test_makeup_records_with_description() {
        // Arrange
        let record = create_test_csv_record();

        // Act
        let csv_line = YPBankCsvFormat::makeup_records(&record);

        // Assert
        let expected = "123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test transaction\"";
        assert_eq!(csv_line, expected);
    }

    #[test]
    fn test_makeup_records_empty_description() {
        // Arrange
        let record = create_deposit_csv_record();

        // Act
        let csv_line = YPBankCsvFormat::makeup_records(&record);

        // Assert
        let expected = "987654321,DEPOSIT,0,1003,100000,1633046401,PENDING,\"\"";
        assert_eq!(csv_line, expected);
    }

    #[test]
    fn test_makeup_records_description_with_quotes() {
        // Arrange
        let mut record = create_test_csv_record();
        record.description = "Test \"quoted\" transaction".to_string();

        // Act
        let csv_line = YPBankCsvFormat::makeup_records(&record);

        // Assert
        let expected = "123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test \"\"quoted\"\" transaction\"";
        assert_eq!(csv_line, expected);
    }

    #[test]
    fn test_read_executor_single_record() {
        // Arrange
        let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\
                       123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test transaction\"";

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        let record = &result[0];
        assert_eq!(record.tx_id, 123456789);
        assert_eq!(record.tx_type, TxType::Transfer);
        assert_eq!(record.from_user_id, 1001);
        assert_eq!(record.to_user_id, 1002);
        assert_eq!(record.amount, 50000);
        assert_eq!(record.timestamp, 1633046400);
        assert_eq!(record.status, TxStatus::Success);
        assert_eq!(record.description, "Test transaction");
    }

    #[test]
    fn test_read_executor_multiple_records() {
        // Arrange
        let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\
                       123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test transaction\"\n\
                       987654321,DEPOSIT,0,1003,100000,1633046401,PENDING,\"\"\n\
                       555555555,WITHDRAWAL,1004,0,25000,1633046402,FAILURE,\"Withdrawal\"";

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 3);

        assert_eq!(result[0].tx_type, TxType::Transfer);
        assert_eq!(result[0].status, TxStatus::Success);
        assert_eq!(result[0].description, "Test transaction");

        assert_eq!(result[1].tx_type, TxType::Deposit);
        assert_eq!(result[1].status, TxStatus::Pending);
        assert_eq!(result[1].description, "");

        assert_eq!(result[2].tx_type, TxType::Withdrawal);
        assert_eq!(result[2].status, TxStatus::Failure);
        assert_eq!(result[2].description, "Withdrawal");
    }

    #[test]
    fn test_read_executor_empty_description() {
        // Arrange
        let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\
                       123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"\"";

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, "");
    }

    #[test]
    fn test_read_executor_quoted_description() {
        // Arrange
        let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\
                       123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test, with comma\"";

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, "Test, with comma");
    }

    #[test]
    fn test_read_executor_escaped_quotes_in_description() {
        // Arrange
        let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\
                       123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test \"\"quoted\"\" text\"";

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, "Test \"quoted\" text");
    }

    #[test]
    fn test_read_executor_invalid_header() {
        // Arrange
        let csv_data = "WRONG_HEADER,WRONG_TYPE\n\
                       123456789,TRANSFER";

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string());

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(ParseError::ParseError { .. })));
    }

    #[test]
    fn test_read_executor_missing_header() {
        // Arrange
        let csv_data = "";

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string());

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_read_executor_wrong_column_count() {
        // Arrange
        let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\
                       123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS"; // Missing description

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string());

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(ParseError::ParseError { .. })));
    }

    #[test]
    fn test_read_executor_empty_file() {
        // Arrange
        let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION";
        // Only header, no data

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_read_executor_invalid_tx_type() {
        // Arrange
        let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\
                       123456789,INVALID_TYPE,1001,1002,50000,1633046400,SUCCESS,\"Test\"";

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string());

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_read_executor_invalid_status() {
        // Arrange
        let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\
                       123456789,TRANSFER,1001,1002,50000,1633046400,INVALID_STATUS,\"Test\"";

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string());

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_read_executor_invalid_number_format() {
        // Arrange
        let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\
                       NOT_A_NUMBER,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test\"";

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string());

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_write_to_single_record() {
        // Arrange
        let record = create_test_csv_record();
        let mut buffer = Vec::new();

        // Act
        YPBankCsvFormat::write_to(&mut buffer, &[record]).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Assert
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[0],
            "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION"
        );
        assert_eq!(
            lines[1],
            "123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test transaction\""
        );
    }

    #[test]
    fn test_write_to_multiple_records() {
        // Arrange
        let records = vec![
            create_test_csv_record(),
            create_deposit_csv_record(),
            create_withdrawal_csv_record(),
        ];
        let mut buffer = Vec::new();

        // Act
        YPBankCsvFormat::write_to(&mut buffer, &records).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Assert
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(
            lines[0],
            "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION"
        );
        assert_eq!(
            lines[1],
            "123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test transaction\""
        );
        assert_eq!(
            lines[2],
            "987654321,DEPOSIT,0,1003,100000,1633046401,PENDING,\"\""
        );
        assert_eq!(
            lines[3],
            "555555555,WITHDRAWAL,1004,0,25000,1633046402,FAILURE,\"Withdrawal\""
        );
    }

    #[test]
    fn test_write_to_empty_records() {
        // Arrange
        let records: Vec<YPBankCsvFormat> = Vec::new();
        let mut buffer = Vec::new();

        // Act
        YPBankCsvFormat::write_to(&mut buffer, &records).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Assert
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0],
            "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION"
        );
    }

    #[test]
    fn test_write_to_quotes_in_description() {
        // Arrange
        let mut record = create_test_csv_record();
        record.description = "Test \"quoted\" description".to_string();
        let mut buffer = Vec::new();

        // Act
        YPBankCsvFormat::write_to(&mut buffer, &[record]).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Assert
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(
            lines[1],
            "123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test \"\"quoted\"\" description\""
        );
    }

    #[test]
    fn test_write_to_comma_in_description() {
        // Arrange
        let mut record = create_test_csv_record();
        record.description = "Test, with, commas".to_string();
        let mut buffer = Vec::new();

        // Act
        YPBankCsvFormat::write_to(&mut buffer, &[record]).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Assert
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(
            lines[1],
            "123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test, with, commas\""
        );
    }

    #[test]
    fn test_write_read_roundtrip() {
        // Arrange
        let records = vec![
            create_test_csv_record(),
            create_deposit_csv_record(),
            create_withdrawal_csv_record(),
        ];

        // Act: write
        let mut buffer = Vec::new();
        YPBankCsvFormat::write_to(&mut buffer, &records).unwrap();

        // Act: read
        let csv_string = String::from_utf8(buffer).unwrap();
        let read_records = YPBankCsvFormat::read_executor(csv_string).unwrap();

        // Assert
        assert_eq!(read_records.len(), 3);

        // Проверяем, что все поля совпадают
        for (original, read) in records.iter().zip(read_records.iter()) {
            assert_eq!(original.tx_id, read.tx_id);
            assert_eq!(original.tx_type, read.tx_type);
            assert_eq!(original.from_user_id, read.from_user_id);
            assert_eq!(original.to_user_id, read.to_user_id);
            assert_eq!(original.amount, read.amount);
            assert_eq!(original.timestamp, read.timestamp);
            assert_eq!(original.status, read.status);
            assert_eq!(original.description, read.description);
        }
    }

    #[test]
    fn test_write_read_roundtrip_special_characters() {
        // Arrange
        let mut record = create_test_csv_record();
        record.description = "Test \"quoted\", with comma\nand newline".to_string();

        // Act: write
        let mut buffer = Vec::new();
        YPBankCsvFormat::write_to(&mut buffer, &[record.clone()]).unwrap();

        // Act: read
        let csv_string = String::from_utf8(buffer).unwrap();
        let read_records = YPBankCsvFormat::read_executor(csv_string);

        // Assert
        assert_eq!(read_records.is_err(), true);
    }

    #[test]
    fn test_all_tx_types_enum_strings() {
        // Проверяем строковые представления enum
        assert_eq!(TxType::Deposit.to_string(), "DEPOSIT");
        assert_eq!(TxType::Transfer.to_string(), "TRANSFER");
        assert_eq!(TxType::Withdrawal.to_string(), "WITHDRAWAL");
    }

    #[test]
    fn test_all_status_enum_strings() {
        // Проверяем строковые представления enum
        assert_eq!(TxStatus::Success.to_string(), "SUCCESS");
        assert_eq!(TxStatus::Failure.to_string(), "FAILURE");
        assert_eq!(TxStatus::Pending.to_string(), "PENDING");
    }

    #[test]
    fn test_large_numbers() {
        // Arrange
        let csv_data = format!(
            "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\
                               {},TRANSFER,{},{},{},{},SUCCESS,\"Large numbers\"",
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX
        );

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tx_id, u64::MAX);
        assert_eq!(result[0].from_user_id, u64::MAX);
        assert_eq!(result[0].to_user_id, u64::MAX);
        assert_eq!(result[0].amount, u64::MAX);
        assert_eq!(result[0].timestamp, u64::MAX);
        assert_eq!(result[0].description, "Large numbers");
    }

    #[test]
    fn test_zero_amount() {
        // Arrange
        let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\
                       123456789,TRANSFER,1001,1002,0,1633046400,SUCCESS,\"Zero amount\"";

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].amount, 0);
    }

    #[test]
    fn test_trailing_newline() {
        // Arrange
        let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\
                       123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test\"\n"; // Trailing newline

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_carriage_return_newline() {
        // Arrange
        let csv_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\r\n\
                       123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test\"\r\n";

        // Act
        let result = YPBankCsvFormat::read_executor(csv_data.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_parse_data_line_valid() {
        // Arrange
        let title_data: Vec<String> = YPBankCsvFormat::fields()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let line = "123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test transaction\"";

        // Act
        let result = YPBankCsvFormat::parse_data_line(&title_data, line, 1);

        // Assert
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.tx_id, 123456789);
        assert_eq!(record.tx_type, TxType::Transfer);
        assert_eq!(record.description, "Test transaction");
    }

    #[test]
    fn test_parse_data_line_invalid_column_count() {
        // Arrange
        let title_data: Vec<String> = YPBankCsvFormat::fields()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let line = "123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS"; // Missing description

        // Act
        let result = YPBankCsvFormat::parse_data_line(&title_data, line, 1);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_data_line_empty_fields() {
        // Arrange
        let title_data: Vec<String> = YPBankCsvFormat::fields()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let line = ",,,,,,,"; // All empty fields

        // Act
        let result = YPBankCsvFormat::parse_data_line(&title_data, line, 1);

        // Assert
        assert!(result.is_err()); // Должно быть ошибкой парсинга чисел
    }

    #[test]
    fn test_description_always_present() {
        // Arrange
        let record = YPBankCsvFormat {
            tx_id: 1,
            tx_type: TxType::Transfer,
            from_user_id: 1001,
            to_user_id: 1002,
            amount: 100,
            timestamp: 1633046400,
            status: TxStatus::Success,
            description: String::new(), // Пустая строка, но поле присутствует всегда
        };

        // Act & Assert
        // Просто проверяем, что структура создается корректно
        assert_eq!(record.tx_id, 1);
        assert_eq!(record.description, "");
    }

    #[test]
    fn test_makeup_records_with_semicolon_in_description() {
        // Arrange
        let mut record = create_test_csv_record();
        record.description = "Test; with; semicolons".to_string();

        // Act
        let csv_line = YPBankCsvFormat::makeup_records(&record);

        // Assert
        // Точки с запятой не экранируются, так как разделитель - запятая
        let expected =
            "123456789,TRANSFER,1001,1002,50000,1633046400,SUCCESS,\"Test; with; semicolons\"";
        assert_eq!(csv_line, expected);
    }

    #[test]
    fn test_write_read_with_semicolon_in_description() {
        // Arrange
        let mut record = create_test_csv_record();
        record.description = "Test; with; semicolons".to_string();

        // Act: write
        let mut buffer = Vec::new();
        YPBankCsvFormat::write_to(&mut buffer, &[record.clone()]).unwrap();

        // Act: read
        let csv_string = String::from_utf8(buffer).unwrap();
        let read_records = YPBankCsvFormat::read_executor(csv_string).unwrap();

        // Assert
        assert_eq!(read_records.len(), 1);
        assert_eq!(read_records[0].description, "Test; with; semicolons");
    }
}
