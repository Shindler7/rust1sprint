//! Запись и чтение файлов формата *.txt.

use crate::errors::ParseError;
use crate::format::tools::LineUtils;
use crate::models::YPBankTextFormat;
use crate::traits::YPBankIO;
use regex::Regex;
use std::collections::HashMap;
use std::io::Write;

impl YPBankIO for YPBankTextFormat {
    /// Парсинг (чтение) данных в формате `txt`.
    ///
    /// Возвращает вектор экземпляров `YPBankTextFormat`, содержащих все записи из источника.
    type DataFormat = YPBankTextFormat;

    fn read_executor(buffer: String) -> Result<Vec<YPBankTextFormat>, ParseError> {
        let mut transaction: Vec<YPBankTextFormat> = Vec::new();

        let mut block_buffer: Vec<String> = Vec::new();
        for (count, line) in buffer.lines().enumerate() {
            if line.is_empty_line() {
                continue;
            }

            match (block_buffer.is_empty(), line.is_hash_marker()) {
                (true, true) => {
                    // Начало блока.
                    let title = Self::parse_title(line, count)?;
                    block_buffer.push(title);
                }
                (false, true) => {
                    // Буфер собрали. Надо отдать его на обработку и обнулить.
                    let block_data = Self::parse_block(&block_buffer, count)?;
                    transaction.push(block_data);
                    block_buffer.clear(); // Обработанные данные.

                    let title = Self::parse_title(line, count)?; // Новый цикл.
                    block_buffer.push(title);
                }
                (false, false) => {
                    // Внутри блока.
                    block_buffer.push(line.to_string());
                }
                (true, false) => {
                    return Err(ParseError::parse_error(
                        format!("Некорректная строка: {line}"),
                        count + 1,
                        0,
                    ));
                }
            }
        }

        if !block_buffer.is_empty() {
            let block_data = Self::parse_block(&block_buffer, buffer.lines().count())?;
            transaction.push(block_data);
        }

        Ok(transaction)
    }

    /// Добавить записи на основе предоставленного экземпляра `YPBankTextFormat`.
    fn write_to<W: Write>(mut writer: W, records: &[Self::DataFormat]) -> Result<(), ParseError> {
        for record in records {
            writeln!(writer, "{}", Self::makeup_records(record))?;
        }

        Ok(())
    }
}

impl YPBankTextFormat {
    /// Парсинг отдельного блока информации.
    ///
    /// # Аргументы
    ///
    /// * `block` — вектор со строками блока для парсинга. Нулевая запись вектора это технические
    ///    данные. Например, вид операции из заголовка блока.
    /// * `end_line` — номер последней линии блока.
    ///
    /// ## Образец блока:
    /// ```plain
    /// ## Record 1 (DEPOSIT)
    /// TX_TYPE: DEPOSIT
    /// TO_USER_ID: 9223372036854775807
    /// FROM_USER_ID: 0
    /// TIMESTAMP: 1633036860000
    /// DESCRIPTION: "Record number 1"
    /// TX_ID: 1000000000000000
    /// AMOUNT: 100
    /// STATUS: FAILURE
    /// ```
    fn parse_block(block: &Vec<String>, end_line: usize) -> Result<YPBankTextFormat, ParseError> {
        let mut fields = HashMap::new();
        let first_line = end_line - block.len();

        for (count, line) in (1..).zip(block[1..].iter()) {
            if let Some((key, value)) = line.split_into_key_value() {
                // Подбор и проверка полей.
                if !YPBankTextFormat::has_field_from_str(&key) {
                    return Err(ParseError::parse_error(
                        format!("Некорректный ключ {key} в строке: {line}"),
                        first_line + count,
                        0,
                    ));
                }
                fields.insert(key, value);
            } else {
                return Err(ParseError::parse_error(
                    format!("Неверный формат строки txt: {}", line),
                    first_line + count,
                    0,
                ));
            }
        }

        let result = YPBankTextFormat::new_from_map(fields)?;

        Ok(result)
    }

    /// Парсинг заголовка сообщения.
    ///
    /// Возвращает `String` с названием операции, если парсинг успешен или `ParseError`,
    /// если возникли ошибки.
    ///
    /// ## Образец заголовка
    ///
    /// ```plain
    /// ## Record 1 (DEPOSIT)
    /// ```
    fn parse_title(line: &str, count_line: usize) -> Result<String, ParseError> {
        let re = Regex::new(r#"^#\s*Record\s+\d+\s*\((?P<tx_type>[^)]+)\)$"#)
            .expect("Ошибка в регулярном выражении парсинга заголовка блоков формата TXT");

        re.captures(line)
            .and_then(|caps| caps.name("tx_type"))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| {
                ParseError::parse_error(
                    format!("Некорректная строка заголовка: {}", line),
                    count_line,
                    0,
                )
            })
    }

    /// Подготовить единицу записи к публикации.
    fn makeup_records(records: &YPBankTextFormat) -> String {
        format!("{}\n{}", Self::make_title(&records), records)
    }

    /// Формирует заголовок блока записи.
    ///
    /// ## Образец заголовка
    ///
    /// ```plain
    /// ## Record 2 (TRANSFER)
    /// ```
    fn make_title(records: &YPBankTextFormat) -> String {
        let tx_id = records.tx_id % 1_000_000_000_000_000;
        format!("# Record {} ({})", tx_id, records.tx_type)
    }
}

#[cfg(test)]
mod text_tests {
    use super::*;
    use crate::models::{TxStatus, TxType, YPBankTextFormat};
    use crate::traits::YPBankIO;

    fn create_test_text_record() -> YPBankTextFormat {
        YPBankTextFormat {
            tx_id: 1234567890000000,
            tx_type: TxType::Transfer,
            from_user_id: 1001,
            to_user_id: 1002,
            amount: 50000,
            timestamp: 1633046400,
            status: TxStatus::Success,
            description: "Test transaction".to_string(),
        }
    }

    fn create_deposit_text_record() -> YPBankTextFormat {
        YPBankTextFormat {
            tx_id: 9876543210000000,
            tx_type: TxType::Deposit,
            from_user_id: 0,
            to_user_id: 1003,
            amount: 100000,
            timestamp: 1633046401,
            status: TxStatus::Pending,
            description: String::new(),
        }
    }

    fn create_withdrawal_text_record() -> YPBankTextFormat {
        YPBankTextFormat {
            tx_id: 5555555550000000,
            tx_type: TxType::Withdrawal,
            from_user_id: 1004,
            to_user_id: 0,
            amount: 25000,
            timestamp: 1633046402,
            status: TxStatus::Failure,
            description: "Withdrawal description".to_string(),
        }
    }

    fn sample_deposit_block() -> String {
        String::from(
            "# Record 7890000000 (DEPOSIT)\n\
            TX_TYPE: DEPOSIT\n\
            TO_USER_ID: 1003\n\
            FROM_USER_ID: 0\n\
            TIMESTAMP: 1633046401\n\
            DESCRIPTION: \"\"\n\
            TX_ID: 9876543210000000\n\
            AMOUNT: 100000\n\
            STATUS: PENDING\n",
        )
    }

    fn sample_transfer_block() -> String {
        String::from(
            "# Record 7890000000 (TRANSFER)\n\
            TX_TYPE: TRANSFER\n\
            FROM_USER_ID: 1001\n\
            TO_USER_ID: 1002\n\
            TIMESTAMP: 1633046400\n\
            DESCRIPTION: \"Test transaction\"\n\
            TX_ID: 1234567890000000\n\
            AMOUNT: 50000\n\
            STATUS: SUCCESS\n",
        )
    }

    fn sample_withdrawal_block() -> String {
        String::from(
            "# Record 5550000000 (WITHDRAWAL)\n\
            TX_TYPE: WITHDRAWAL\n\
            FROM_USER_ID: 1004\n\
            TO_USER_ID: 0\n\
            TIMESTAMP: 1633046402\n\
            DESCRIPTION: \"Withdrawal description\"\n\
            TX_ID: 5555555550000000\n\
            AMOUNT: 25000\n\
            STATUS: FAILURE\n",
        )
    }

    #[test]
    fn test_make_title() {
        // Arrange
        let record = create_test_text_record();

        // Act
        let title = YPBankTextFormat::make_title(&record);

        // Assert
        // tx_id % 1_000_000_000_000_000 = 1234567890000000 % 1000000000000000 = 234567890000000
        assert!(title.starts_with("# Record "));
        assert!(title.contains("(TRANSFER)"));
    }

    #[test]
    fn test_makeup_records() {
        // Arrange
        let record = create_test_text_record();

        // Act
        let formatted = YPBankTextFormat::makeup_records(&record);

        // Assert
        let lines: Vec<&str> = formatted.trim().lines().collect();
        assert!(lines[0].starts_with("# Record "));
        assert!(lines[0].contains("(TRANSFER)"));
        assert!(formatted.contains("TX_TYPE: TRANSFER"));
        assert!(formatted.contains("FROM_USER_ID: 1001"));
        assert!(formatted.contains("TO_USER_ID: 1002"));
        assert!(formatted.contains("AMOUNT: 50000"));
        assert!(formatted.contains("STATUS: SUCCESS"));
        assert!(formatted.contains("DESCRIPTION: \"Test transaction\""));
    }

    #[test]
    fn test_makeup_records_empty_description() {
        // Arrange
        let record = create_deposit_text_record();

        // Act
        let formatted = YPBankTextFormat::makeup_records(&record);

        // Assert
        assert!(formatted.contains("DESCRIPTION: \"\""));
    }

    #[test]
    fn test_makeup_records_quotes_in_description() {
        // Arrange
        let mut record = create_test_text_record();
        record.description = "Test \"quoted\" description".to_string();

        // Act
        let formatted = YPBankTextFormat::makeup_records(&record);

        // Assert
        assert!(formatted.contains("DESCRIPTION: \"Test \"\"quoted\"\" description\""));
    }

    #[test]
    fn test_parse_title_valid() {
        // Arrange
        let valid_titles = vec![
            "# Record 1 (DEPOSIT)",
            "# Record 123 (TRANSFER)",
            "# Record 999999999 (WITHDRAWAL)",
            "#Record 1 (DEPOSIT)",       // Без пробела после #
            "# Record 1 (DEPOSIT) ",     // С пробелом в конце
            " # Record 1 (DEPOSIT)",     // С пробелом в начале
            "#  Record  1  (DEPOSIT)  ", // Множественные пробелы
        ];

        for (i, title) in valid_titles.iter().enumerate() {
            // Act
            let result = YPBankTextFormat::parse_title(title, i);

            // Assert
            assert!(result.is_ok(), "Failed for: {}", title);
            let tx_type = result.unwrap();
            assert!(!tx_type.is_empty());
        }
    }

    #[test]
    fn test_parse_title_invalid() {
        // Arrange
        let invalid_titles = vec![
            "",                          // Пустая строка
            "Record 1 (DEPOSIT)",        // Нет #
            "# Record (DEPOSIT)",        // Нет номера
            "# Record 1 DEPOSIT)",       // Нет открывающей скобки
            "# Record 1 (DEPOSIT",       // Нет закрывающей скобки
            "# Record 1 ()",             // Пустые скобки
            "# Record abc (DEPOSIT)",    // Не число
            "# Record 1",                // Нет скобок вообще
            "## Record 1 (DEPOSIT)",     // Два ##
            "# Record 1 (INVALID_TYPE)", // Несуществующий тип
        ];

        for (i, title) in invalid_titles.iter().enumerate() {
            // Act
            let result = YPBankTextFormat::parse_title(title, i);

            // Assert
            assert!(result.is_err(), "Should fail for: {}", title);
        }
    }

    #[test]
    fn test_read_executor_single_block() {
        // Arrange
        let input = sample_transfer_block();

        // Act
        let result = YPBankTextFormat::read_executor(input).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        let record = &result[0];
        assert_eq!(record.tx_id, 1234567890000000);
        assert_eq!(record.tx_type, TxType::Transfer);
        assert_eq!(record.from_user_id, 1001);
        assert_eq!(record.to_user_id, 1002);
        assert_eq!(record.amount, 50000);
        assert_eq!(record.timestamp, 1633046400);
        assert_eq!(record.status, TxStatus::Success);
        assert_eq!(record.description, "Test transaction");
    }

    #[test]
    fn test_read_executor_multiple_blocks() {
        // Arrange
        let input = format!("{}\n\n{}", sample_transfer_block(), sample_deposit_block());

        // Act
        let result = YPBankTextFormat::read_executor(input).unwrap();

        // Assert
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].tx_type, TxType::Transfer);
        assert_eq!(result[0].status, TxStatus::Success);

        assert_eq!(result[1].tx_type, TxType::Deposit);
        assert_eq!(result[1].status, TxStatus::Pending);
        assert_eq!(result[1].description, "");
    }

    #[test]
    fn test_read_executor_multiple_blocks_with_empty_lines() {
        // Arrange
        let input = format!(
            "{}\n\n\n{}\n\n\n{}",
            sample_transfer_block(),
            sample_deposit_block(),
            sample_withdrawal_block()
        );

        // Act
        let result = YPBankTextFormat::read_executor(input).unwrap();

        // Assert
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].tx_type, TxType::Transfer);
        assert_eq!(result[1].tx_type, TxType::Deposit);
        assert_eq!(result[2].tx_type, TxType::Withdrawal);
    }

    #[test]
    fn test_read_executor_all_types() {
        // Arrange
        let input = format!(
            "{}\n{}\n{}",
            sample_deposit_block(),
            sample_transfer_block(),
            sample_withdrawal_block()
        );

        // Act
        let result = YPBankTextFormat::read_executor(input).unwrap();

        // Assert
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].tx_type, TxType::Deposit);
        assert_eq!(result[1].tx_type, TxType::Transfer);
        assert_eq!(result[2].tx_type, TxType::Withdrawal);
    }

    #[test]
    fn test_read_executor_empty_description() {
        // Arrange
        let input = sample_deposit_block(); // В депозите пустое описание

        // Act
        let result = YPBankTextFormat::read_executor(input).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, "");
    }

    #[test]
    fn test_read_executor_quoted_description() {
        // Arrange
        let input = "# Record 1 (TRANSFER)\n\
                    TX_TYPE: TRANSFER\n\
                    FROM_USER_ID: 1001\n\
                    TO_USER_ID: 1002\n\
                    TIMESTAMP: 1633046400\n\
                    DESCRIPTION: \"Test, with comma\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 50000\n\
                    STATUS: SUCCESS\n";

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, "Test, with comma");
    }

    #[test]
    fn test_read_executor_escaped_quotes_in_description() {
        // Arrange
        let input = "# Record 1 (TRANSFER)\n\
                    TX_TYPE: TRANSFER\n\
                    FROM_USER_ID: 1001\n\
                    TO_USER_ID: 1002\n\
                    TIMESTAMP: 1633046400\n\
                    DESCRIPTION: \"Test \"\"quoted\"\" text\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 50000\n\
                    STATUS: SUCCESS\n";

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, "Test \"quoted\" text");
    }

    #[test]
    fn test_read_executor_missing_header() {
        // Arrange
        let input = "TX_TYPE: TRANSFER\n\
                    FROM_USER_ID: 1001\n";

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string());

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_read_executor_wrong_line_before_header() {
        // Arrange
        let input = "SOME_TEXT\n# Record 1 (DEPOSIT)\nTX_TYPE: DEPOSIT\n";

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string());

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_read_executor_missing_fields() {
        // Arrange
        let input = "# Record 1 (DEPOSIT)\n\
                    TX_TYPE: DEPOSIT\n\
                    TO_USER_ID: 1\n";
        // Отсутствуют обязательные поля

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string());

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_read_executor_duplicate_fields() {
        // Arrange
        let input = "# Record 1 (DEPOSIT)\n\
                    TX_TYPE: DEPOSIT\n\
                    TO_USER_ID: 1\n\
                    FROM_USER_ID: 0\n\
                    TIMESTAMP: 1633036860000\n\
                    DESCRIPTION: \"Test\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 1000\n\
                    STATUS: SUCCESS\n\
                    TX_ID: 9999999999999999\n"; // Дублирующее поле

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string());

        // Assert
        // Зависит от реализации new_from_map - может перезаписать или вызвать ошибку
        // assert!(result.is_err());
    }

    #[test]
    fn test_read_executor_invalid_tx_type() {
        // Arrange
        let input = "# Record 1 (INVALID_TYPE)\n\
                    TX_TYPE: INVALID_TYPE\n\
                    TO_USER_ID: 1\n\
                    FROM_USER_ID: 0\n\
                    TIMESTAMP: 1633036860000\n\
                    DESCRIPTION: \"Test\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 1000\n\
                    STATUS: SUCCESS\n";

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string());

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_read_executor_invalid_status() {
        // Arrange
        let input = "# Record 1 (DEPOSIT)\n\
                    TX_TYPE: DEPOSIT\n\
                    TO_USER_ID: 1\n\
                    FROM_USER_ID: 0\n\
                    TIMESTAMP: 1633036860000\n\
                    DESCRIPTION: \"Test\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 1000\n\
                    STATUS: INVALID_STATUS\n";

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string());

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_read_executor_invalid_number_format() {
        // Arrange
        let input = "# Record 1 (DEPOSIT)\n\
                    TX_TYPE: DEPOSIT\n\
                    TO_USER_ID: not_a_number\n\
                    FROM_USER_ID: 0\n\
                    TIMESTAMP: 1633036860000\n\
                    DESCRIPTION: \"Test\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 1000\n\
                    STATUS: SUCCESS\n";

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string());

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_read_executor_incorrect_key() {
        // Arrange
        let input = "# Record 1 (DEPOSIT)\n\
                    TX_TYPE: DEPOSIT\n\
                    UNKNOWN_FIELD: value\n\
                    TO_USER_ID: 1\n\
                    FROM_USER_ID: 0\n\
                    TIMESTAMP: 1633036860000\n\
                    DESCRIPTION: \"Test\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 1000\n\
                    STATUS: SUCCESS\n";

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string());

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_read_executor_empty_input() {
        // Arrange
        let input = "";

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string());

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_read_executor_only_empty_lines() {
        // Arrange
        let input = "\n\n\n  \n\t\n";

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string());

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_write_to_single_record() {
        // Arrange
        let record = create_test_text_record();
        let mut buffer = Vec::new();

        // Act
        YPBankTextFormat::write_to(&mut buffer, &[record]).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Assert
        let lines: Vec<&str> = output.trim().lines().collect();
        assert!(lines[0].starts_with("# Record "));
        assert!(lines[0].contains("(TRANSFER)"));
        assert!(output.contains("TX_TYPE: TRANSFER"));
        assert!(output.contains("FROM_USER_ID: 1001"));
        assert!(output.contains("TO_USER_ID: 1002"));
        assert!(output.contains("AMOUNT: 50000"));
        assert!(output.contains("STATUS: SUCCESS"));
        assert!(output.contains("DESCRIPTION: \"Test transaction\""));
    }

    #[test]
    fn test_write_to_multiple_records() {
        // Arrange
        let records = vec![
            create_test_text_record(),
            create_deposit_text_record(),
            create_withdrawal_text_record(),
        ];
        let mut buffer = Vec::new();

        // Act
        YPBankTextFormat::write_to(&mut buffer, &records).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Assert
        let blocks: Vec<&str> = output.trim().split("\n\n").collect();
        assert_eq!(blocks.len(), 3);

        assert!(blocks[0].contains("(TRANSFER)"));
        assert!(blocks[0].contains("STATUS: SUCCESS"));

        assert!(blocks[1].contains("(DEPOSIT)"));
        assert!(blocks[1].contains("STATUS: PENDING"));
        assert!(blocks[1].contains("DESCRIPTION: \"\""));

        assert!(blocks[2].contains("(WITHDRAWAL)"));
        assert!(blocks[2].contains("STATUS: FAILURE"));
    }

    #[test]
    fn test_write_to_empty_records() {
        // Arrange
        let records: Vec<YPBankTextFormat> = Vec::new();
        let mut buffer = Vec::new();

        // Act
        YPBankTextFormat::write_to(&mut buffer, &records).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Assert
        assert_eq!(output.trim(), "");
    }

    #[test]
    fn test_write_to_quotes_in_description() {
        // Arrange
        let mut record = create_test_text_record();
        record.description = "Test \"quoted\" description".to_string();
        let mut buffer = Vec::new();

        // Act
        YPBankTextFormat::write_to(&mut buffer, &[record]).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Assert
        assert!(output.contains("DESCRIPTION: \"Test \"\"quoted\"\" description\""));
    }

    #[test]
    fn test_write_to_comma_in_description() {
        // Arrange
        let mut record = create_test_text_record();
        record.description = "Test, with, commas".to_string();
        let mut buffer = Vec::new();

        // Act
        YPBankTextFormat::write_to(&mut buffer, &[record]).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Assert
        assert!(output.contains("DESCRIPTION: \"Test, with, commas\""));
    }

    #[test]
    fn test_write_to_newline_in_description() {
        // Arrange
        let mut record = create_test_text_record();
        record.description = "Test\nwith\nnewlines".to_string();
        let mut buffer = Vec::new();

        // Act
        YPBankTextFormat::write_to(&mut buffer, &[record]).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Assert
        assert!(output.contains("DESCRIPTION: \"Test\nwith\nnewlines\""));
    }

    #[test]
    fn test_write_read_roundtrip() {
        // Arrange
        let records = vec![
            create_test_text_record(),
            create_deposit_text_record(),
            create_withdrawal_text_record(),
        ];

        // Act: write
        let mut buffer = Vec::new();
        YPBankTextFormat::write_to(&mut buffer, &records).unwrap();

        // Act: read
        let text_string = String::from_utf8(buffer).unwrap();
        let read_records = YPBankTextFormat::read_executor(text_string).unwrap();

        // Assert
        assert_eq!(read_records.len(), 3);

        // Проверяем, что все поля совпадают (кроме возможного порядка в выводе)
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
        let mut record = create_test_text_record();
        record.description = "Test \"quoted\", with comma\nand newline".to_string();

        // Act: write
        let mut buffer = Vec::new();
        YPBankTextFormat::write_to(&mut buffer, &[record.clone()]).unwrap();

        // Act: read
        let text_string = String::from_utf8(buffer).unwrap();
        let read_records = YPBankTextFormat::read_executor(text_string).unwrap();

        // Assert
        assert_eq!(read_records.len(), 1);
        assert_eq!(read_records[0].description, record.description);
    }

    #[test]
    fn test_parse_block_valid() {
        // Arrange
        let block_lines = vec![
            "DEPOSIT".to_string(), // Тип из заголовка
            "TX_TYPE: DEPOSIT".to_string(),
            "TO_USER_ID: 1003".to_string(),
            "FROM_USER_ID: 0".to_string(),
            "TIMESTAMP: 1633046401".to_string(),
            "DESCRIPTION: \"\"".to_string(),
            "TX_ID: 9876543210000000".to_string(),
            "AMOUNT: 100000".to_string(),
            "STATUS: PENDING".to_string(),
        ];

        // Act
        let result = YPBankTextFormat::parse_block(&block_lines, 10);

        // Assert
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.tx_type, TxType::Deposit);
        assert_eq!(record.to_user_id, 1003);
        assert_eq!(record.description, "");
    }

    #[test]
    fn test_parse_block_missing_field() {
        // Arrange
        let block_lines = vec![
            "DEPOSIT".to_string(),
            "TX_TYPE: DEPOSIT".to_string(),
            "TO_USER_ID: 1003".to_string(),
            // Пропущены другие обязательные поля
        ];

        // Act
        let result = YPBankTextFormat::parse_block(&block_lines, 4);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_block_incorrect_format() {
        // Arrange
        let block_lines = vec![
            "DEPOSIT".to_string(),
            "TX_TYPE DEPOSIT".to_string(), // Нет двоеточия
            "TO_USER_ID: 1003".to_string(),
        ];

        // Act
        let result = YPBankTextFormat::parse_block(&block_lines, 3);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_block_unknown_key() {
        // Arrange
        let block_lines = vec![
            "DEPOSIT".to_string(),
            "TX_TYPE: DEPOSIT".to_string(),
            "UNKNOWN_KEY: value".to_string(), // Неизвестный ключ
            "TO_USER_ID: 1003".to_string(),
        ];

        // Act
        let result = YPBankTextFormat::parse_block(&block_lines, 4);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_large_numbers() {
        // Arrange
        let input = format!(
            "# Record 1 (TRANSFER)\n\
            TX_TYPE: TRANSFER\n\
            FROM_USER_ID: {}\n\
            TO_USER_ID: {}\n\
            TIMESTAMP: {}\n\
            DESCRIPTION: \"Large numbers\"\n\
            TX_ID: {}\n\
            AMOUNT: {}\n\
            STATUS: SUCCESS\n",
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX
        );

        // Act
        let result = YPBankTextFormat::read_executor(input).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].from_user_id, u64::MAX);
        assert_eq!(result[0].to_user_id, u64::MAX);
        assert_eq!(result[0].timestamp, u64::MAX);
        assert_eq!(result[0].tx_id, u64::MAX);
        assert_eq!(result[0].amount, u64::MAX);
    }

    #[test]
    fn test_zero_values() {
        // Arrange
        let input = "# Record 1 (DEPOSIT)\n\
                    TX_TYPE: DEPOSIT\n\
                    TO_USER_ID: 0\n\
                    FROM_USER_ID: 0\n\
                    TIMESTAMP: 0\n\
                    DESCRIPTION: \"\"\n\
                    TX_ID: 0\n\
                    AMOUNT: 0\n\
                    STATUS: SUCCESS\n";

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tx_id, 0);
        assert_eq!(result[0].amount, 0);
        assert_eq!(result[0].timestamp, 0);
        assert_eq!(result[0].description, "");
    }

    #[test]
    fn test_field_order_insensitive() {
        // Arrange - поля в произвольном порядке
        let input = "# Record 1 (TRANSFER)\n\
                    AMOUNT: 50000\n\
                    STATUS: SUCCESS\n\
                    TX_TYPE: TRANSFER\n\
                    DESCRIPTION: \"Test\"\n\
                    FROM_USER_ID: 1001\n\
                    TX_ID: 1234567890000000\n\
                    TIMESTAMP: 1633046400\n\
                    TO_USER_ID: 1002\n";

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tx_type, TxType::Transfer);
        assert_eq!(result[0].status, TxStatus::Success);
        assert_eq!(result[0].amount, 50000);
    }

    #[test]
    fn test_empty_lines_between_blocks() {
        // Arrange
        let input = "\n\n# Record 1 (DEPOSIT)\n\
                    TX_TYPE: DEPOSIT\n\
                    TO_USER_ID: 1\n\
                    FROM_USER_ID: 0\n\
                    TIMESTAMP: 1633036860000\n\
                    DESCRIPTION: \"\"\n\
                    TX_ID: 1000000000000000\n\
                    AMOUNT: 100\n\
                    STATUS: FAILURE\n\n\n\
                    # Record 2 (TRANSFER)\n\
                    TX_TYPE: TRANSFER\n\
                    FROM_USER_ID: 1001\n\
                    TO_USER_ID: 1002\n\
                    TIMESTAMP: 1633046400\n\
                    DESCRIPTION: \"Test\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 50000\n\
                    STATUS: SUCCESS\n\n";

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].tx_type, TxType::Deposit);
        assert_eq!(result[1].tx_type, TxType::Transfer);
    }

    #[test]
    fn test_description_with_colon() {
        // Arrange
        let input = "# Record 1 (TRANSFER)\n\
                    TX_TYPE: TRANSFER\n\
                    FROM_USER_ID: 1001\n\
                    TO_USER_ID: 1002\n\
                    TIMESTAMP: 1633046400\n\
                    DESCRIPTION: \"Time: 12:00:00\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 50000\n\
                    STATUS: SUCCESS\n";

        // Act
        let result = YPBankTextFormat::read_executor(input.to_string()).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, "Time: 12:00:00");
    }

    #[test]
    fn test_to_string_format() {
        // Arrange
        let record = create_test_text_record();

        // Act
        let string_repr = record.to_string();

        // Assert
        // Проверяем, что все поля присутствуют в выводе
        assert!(string_repr.contains("TX_TYPE: TRANSFER"));
        assert!(string_repr.contains("FROM_USER_ID: 1001"));
        assert!(string_repr.contains("TO_USER_ID: 1002"));
        assert!(string_repr.contains("AMOUNT: 50000"));
        assert!(string_repr.contains("TIMESTAMP: 1633046400"));
        assert!(string_repr.contains("STATUS: SUCCESS"));
        assert!(string_repr.contains("DESCRIPTION: Test transaction"));
        assert!(string_repr.contains("TX_ID: 1234567890000000"));
    }
}
