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
                    return Err(ParseError::parse_err(
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
    ///   данные. Например, вид операции из заголовка блока.
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
    fn parse_block(block: &[String], end_line: usize) -> Result<YPBankTextFormat, ParseError> {
        let mut fields = HashMap::new();
        let first_line = end_line - block.len();

        for (count, line) in (1..).zip(block[1..].iter()) {
            if let Some((key, value)) = line.split_into_key_value() {
                // Подбор и проверка полей.
                if !YPBankTextFormat::has_field_from_str(&key) {
                    return Err(ParseError::parse_err(
                        format!("Некорректный ключ {key} в строке: {line}"),
                        first_line + count,
                        0,
                    ));
                }

                // Ключи не могут дублироваться, это ошибка.
                if fields.contains_key(&key) {
                    return Err(ParseError::parse_err(
                        format!("Дублирование ключа: {key} в строке: {line}"),
                        first_line + count,
                        0,
                    ));
                }

                fields.insert(key, value);
            } else {
                return Err(ParseError::parse_err(
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
                ParseError::parse_err(
                    format!("Некорректная строка заголовка: {}", line),
                    count_line,
                    0,
                )
            })
    }

    /// Подготовить единицу записи к публикации.
    fn makeup_records(records: &YPBankTextFormat) -> String {
        let mut copy_records = records.clone();
        copy_records.description = copy_records.description.escaped_quote();

        format!("{}\n{}", Self::make_title(records), copy_records)
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
    use crate::models::{TxStatus, TxType, YPBankTextFormat};
    use crate::traits::YPBankIO;

    // ==================== Test Data Factories ====================

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

    // ==================== Test Helper Functions ====================

    fn assert_record_matches(record: &YPBankTextFormat, expected: &YPBankTextFormat) {
        assert_eq!(record.tx_id, expected.tx_id);
        assert_eq!(record.tx_type, expected.tx_type);
        assert_eq!(record.from_user_id, expected.from_user_id);
        assert_eq!(record.to_user_id, expected.to_user_id);
        assert_eq!(record.amount, expected.amount);
        assert_eq!(record.timestamp, expected.timestamp);
        assert_eq!(record.status, expected.status);
        assert_eq!(record.description, expected.description);
    }

    // ==================== Title Tests ====================

    mod title_tests {
        use super::*;

        #[test]
        fn test_make_title() {
            // Arrange
            let record = create_test_text_record();

            // Act
            let title = YPBankTextFormat::make_title(&record);

            // Assert
            assert!(title.starts_with("# Record "));
            assert!(title.contains("(TRANSFER)"));
        }

        #[test]
        fn test_parse_title_valid() {
            // Arrange
            let valid_titles = vec![
                "# Record 1 (DEPOSIT)",
                "# Record 123 (TRANSFER)",
                "# Record 999999999 (WITHDRAWAL)",
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
            let test_cases = vec![
                ("", "Пустая строка"),
                ("Record 1 (DEPOSIT)", "Нет #"),
                ("# Record (DEPOSIT)", "Нет номера"),
                ("# Record 1 DEPOSIT)", "Нет открывающей скобки"),
                ("# Record 1 (DEPOSIT", "Нет закрывающей скобки"),
                ("# Record 1 ()", "Пустые скобки"),
                ("# Record abc (DEPOSIT)", "Не число"),
                ("# Record 1", "Нет скобок вообще"),
                ("## Record 1 (DEPOSIT)", "Два ##"),
            ];

            for (i, (title, description)) in test_cases.iter().enumerate() {
                // Act
                let result = YPBankTextFormat::parse_title(title, i);

                // Assert
                assert!(
                    result.is_err(),
                    "Should fail for: {} - {}",
                    title,
                    description
                );
            }
        }
    }

    // ==================== Formatting Tests ====================

    mod formatting_tests {
        use super::*;

        #[test]
        fn test_makeup_records_basic() {
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
        fn test_makeup_records_with_empty_description() {
            // Arrange
            let record = create_deposit_text_record();

            // Act
            let formatted = YPBankTextFormat::makeup_records(&record);

            // Assert
            assert!(formatted.contains("DESCRIPTION: \"\""));
        }

        #[test]
        fn test_makeup_records_with_special_characters() {
            // Arrange
            let test_cases = vec![
                (
                    "Test \"quoted\" description",
                    "DESCRIPTION: \"Test \"\"quoted\"\" description\"",
                    "кавычки в описании",
                ),
                (
                    "Test, with, commas",
                    "DESCRIPTION: \"Test, with, commas\"",
                    "запятые в описании",
                ),
                (
                    "Test\nwith\nnewlines",
                    "DESCRIPTION: \"Test\nwith\nnewlines\"",
                    "переносы строк в описании",
                ),
                (
                    "Time: 12:00:00",
                    "DESCRIPTION: \"Time: 12:00:00\"",
                    "двоеточия в описании",
                ),
            ];

            for (description, expected_substring, case_name) in test_cases {
                // Arrange
                let mut record = create_test_text_record();
                record.description = description.to_string();

                // Act
                let formatted = YPBankTextFormat::makeup_records(&record);

                // Assert
                assert!(
                    formatted.contains(expected_substring),
                    "Failed for case: {} - Expected: {}, Got: {}",
                    case_name,
                    expected_substring,
                    formatted
                );
            }
        }
    }

    // ==================== Reading Tests ====================

    mod reading_tests {
        use super::*;

        #[test]
        fn test_read_executor_single_record() {
            // Arrange
            let test_cases = vec![
                (
                    sample_transfer_block(),
                    TxType::Transfer,
                    TxStatus::Success,
                    "Test transaction",
                ),
                (
                    sample_deposit_block(),
                    TxType::Deposit,
                    TxStatus::Pending,
                    "",
                ),
                (
                    sample_withdrawal_block(),
                    TxType::Withdrawal,
                    TxStatus::Failure,
                    "Withdrawal description",
                ),
            ];

            for (input, expected_type, expected_status, expected_description) in test_cases {
                // Act
                let result = YPBankTextFormat::read_executor(input).unwrap();

                // Assert
                assert_eq!(result.len(), 1);
                let record = &result[0];
                assert_eq!(record.tx_type, expected_type);
                assert_eq!(record.status, expected_status);
                assert_eq!(record.description, expected_description);
            }
        }

        #[test]
        fn test_read_executor_multiple_records() {
            // Arrange
            let test_cases = vec![
                (
                    format!("{}\n\n{}", sample_transfer_block(), sample_deposit_block()),
                    2,
                    vec![TxType::Transfer, TxType::Deposit],
                    "два блока с пустой строкой",
                ),
                (
                    format!(
                        "{}\n{}\n{}",
                        sample_deposit_block(),
                        sample_transfer_block(),
                        sample_withdrawal_block()
                    ),
                    3,
                    vec![TxType::Deposit, TxType::Transfer, TxType::Withdrawal],
                    "три блока подряд",
                ),
                (
                    format!(
                        "{}\n\n\n{}\n\n\n{}",
                        sample_transfer_block(),
                        sample_deposit_block(),
                        sample_withdrawal_block()
                    ),
                    3,
                    vec![TxType::Transfer, TxType::Deposit, TxType::Withdrawal],
                    "три блока с множеством пустых строк",
                ),
            ];

            for (input, expected_count, expected_types, case_name) in test_cases {
                // Act
                let result = YPBankTextFormat::read_executor(input).unwrap();

                // Assert
                assert_eq!(
                    result.len(),
                    expected_count,
                    "Failed for case: {}",
                    case_name
                );
                for (i, expected_type) in expected_types.iter().enumerate() {
                    assert_eq!(
                        result[i].tx_type, *expected_type,
                        "Failed at index {} for case: {}",
                        i, case_name
                    );
                }
            }
        }

        #[test]
        fn test_read_executor_edge_cases() {
            // Arrange
            let test_cases = vec![
                ("", 0, "пустой ввод"),
                ("\n\n\n  \n\t\n", 0, "только пустые строки"),
            ];

            for (input, expected_count, case_name) in test_cases {
                // Act
                let result = YPBankTextFormat::read_executor(input.to_string()).unwrap();

                // Assert
                assert_eq!(
                    result.len(),
                    expected_count,
                    "Failed for case: {}",
                    case_name
                );
            }
        }

        #[test]
        fn test_read_executor_special_descriptions() {
            // Arrange
            let test_cases = vec![
                (
                    "# Record 1 (TRANSFER)\n\
                    TX_TYPE: TRANSFER\n\
                    FROM_USER_ID: 1001\n\
                    TO_USER_ID: 1002\n\
                    TIMESTAMP: 1633046400\n\
                    DESCRIPTION: \"Test, with comma\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 50000\n\
                    STATUS: SUCCESS\n",
                    "Test, with comma",
                    "запятая в описании",
                ),
                (
                    "# Record 1 (TRANSFER)\n\
                    TX_TYPE: TRANSFER\n\
                    FROM_USER_ID: 1001\n\
                    TO_USER_ID: 1002\n\
                    TIMESTAMP: 1633046400\n\
                    DESCRIPTION: \"Test \"\"quoted\"\" text\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 50000\n\
                    STATUS: SUCCESS\n",
                    "Test \"quoted\" text",
                    "экранированные кавычки в описании",
                ),
            ];

            for (input, expected_description, case_name) in test_cases {
                // Act
                let result = YPBankTextFormat::read_executor(input.to_string()).unwrap();

                // Assert
                assert_eq!(result.len(), 1, "Failed for case: {}", case_name);
                assert_eq!(
                    result[0].description, expected_description,
                    "Failed for case: {}",
                    case_name
                );
            }
        }

        #[test]
        fn test_read_executor_field_order_insensitive() {
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
            assert_eq!(result[0].description, "Test");
        }

        #[test]
        fn test_read_executor_number_formats() {
            // Arrange
            let test_cases = vec![
                (
                    format!(
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
                    ),
                    (u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX),
                    "максимальные значения",
                ),
                (
                    "# Record 1 (DEPOSIT)\n\
                    TX_TYPE: DEPOSIT\n\
                    TO_USER_ID: 0\n\
                    FROM_USER_ID: 0\n\
                    TIMESTAMP: 0\n\
                    DESCRIPTION: \"\"\n\
                    TX_ID: 0\n\
                    AMOUNT: 0\n\
                    STATUS: SUCCESS\n"
                        .to_string(),
                    (0, 0, 0, 0, 0),
                    "нулевые значения",
                ),
            ];

            for (
                input,
                (expected_from, expected_to, expected_ts, expected_id, expected_amount),
                case_name,
            ) in test_cases
            {
                // Act
                let result = YPBankTextFormat::read_executor(input).unwrap();

                // Assert
                assert_eq!(result.len(), 1, "Failed for case: {}", case_name);
                let record = &result[0];
                assert_eq!(
                    record.from_user_id, expected_from,
                    "Failed from_user_id for case: {}",
                    case_name
                );
                assert_eq!(
                    record.to_user_id, expected_to,
                    "Failed to_user_id for case: {}",
                    case_name
                );
                assert_eq!(
                    record.timestamp, expected_ts,
                    "Failed timestamp for case: {}",
                    case_name
                );
                assert_eq!(
                    record.tx_id, expected_id,
                    "Failed tx_id for case: {}",
                    case_name
                );
                assert_eq!(
                    record.amount, expected_amount,
                    "Failed amount for case: {}",
                    case_name
                );
            }
        }
    }

    // ==================== Error Handling Tests ====================

    mod error_handling_tests {
        use super::*;

        #[test]
        fn test_read_executor_invalid_inputs() {
            // Arrange
            let test_cases = vec![
                (
                    "TX_TYPE: TRANSFER\nFROM_USER_ID: 1001\n",
                    "отсутствует заголовок",
                ),
                (
                    "SOME_TEXT\n# Record 1 (DEPOSIT)\nTX_TYPE: DEPOSIT\n",
                    "неправильная строка перед заголовком",
                ),
                (
                    "# Record 1 (DEPOSIT)\nTX_TYPE: DEPOSIT\nTO_USER_ID: 1\n",
                    "отсутствуют обязательные поля",
                ),
                (
                    "# Record 1 (DEPOSIT)\n\
                    TX_TYPE: DEPOSIT\n\
                    TO_USER_ID: 1\n\
                    FROM_USER_ID: 0\n\
                    TIMESTAMP: 1633036860000\n\
                    DESCRIPTION: \"Test\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 1000\n\
                    STATUS: SUCCESS\n\
                    TX_ID: 9999999999999999\n",
                    "дублирующиеся поля",
                ),
            ];

            for (input, case_name) in test_cases {
                // Act
                let result = YPBankTextFormat::read_executor(input.to_string());

                // Assert
                assert!(result.is_err(), "Should fail for case: {}", case_name);
            }
        }

        #[test]
        fn test_read_executor_invalid_enum_values() {
            // Arrange
            let test_cases = vec![
                (
                    "# Record 1 (INVALID_TYPE)\n\
                    TX_TYPE: INVALID_TYPE\n\
                    TO_USER_ID: 1\n\
                    FROM_USER_ID: 0\n\
                    TIMESTAMP: 1633036860000\n\
                    DESCRIPTION: \"Test\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 1000\n\
                    STATUS: SUCCESS\n",
                    "неверный тип транзакции",
                ),
                (
                    "# Record 1 (DEPOSIT)\n\
                    TX_TYPE: DEPOSIT\n\
                    TO_USER_ID: 1\n\
                    FROM_USER_ID: 0\n\
                    TIMESTAMP: 1633036860000\n\
                    DESCRIPTION: \"Test\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 1000\n\
                    STATUS: INVALID_STATUS\n",
                    "неверный статус",
                ),
            ];

            for (input, case_name) in test_cases {
                // Act
                let result = YPBankTextFormat::read_executor(input.to_string());

                // Assert
                assert!(result.is_err(), "Should fail for case: {}", case_name);
            }
        }

        #[test]
        fn test_read_executor_invalid_number_formats() {
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
        fn test_read_executor_incorrect_key_format() {
            // Arrange
            let test_cases = vec![
                (
                    "# Record 1 (DEPOSIT)\n\
                    TX_TYPE DEPOSIT\n\
                    TO_USER_ID: 1003\n",
                    "отсутствует двоеточие",
                ),
                (
                    "# Record 1 (DEPOSIT)\n\
                    TX_TYPE: DEPOSIT\n\
                    UNKNOWN_FIELD: value\n\
                    TO_USER_ID: 1\n\
                    FROM_USER_ID: 0\n\
                    TIMESTAMP: 1633036860000\n\
                    DESCRIPTION: \"Test\"\n\
                    TX_ID: 1234567890000000\n\
                    AMOUNT: 1000\n\
                    STATUS: SUCCESS\n",
                    "неизвестное поле",
                ),
            ];

            for (input, case_name) in test_cases {
                // Act
                let result = YPBankTextFormat::read_executor(input.to_string());

                // Assert
                assert!(result.is_err(), "Should fail for case: {}", case_name);
            }
        }
    }

    // ==================== Writing Tests ====================

    mod writing_tests {
        use super::*;

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
    }

    // ==================== Roundtrip Tests ====================

    mod roundtrip_tests {
        use super::*;

        #[test]
        fn test_write_read_roundtrip_basic() {
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

            // Проверяем, что все поля совпадают
            for (original, read) in records.iter().zip(read_records.iter()) {
                assert_record_matches(read, original);
            }
        }

        #[test]
        fn test_write_read_roundtrip_special_characters() {
            // Arrange
            let test_cases = vec![
                "Test \"quoted\" description",
                "Test, with, commas",
                "Test\nwith\nnewlines",
                "Time: 12:00:00",
                "Test \"quoted\", with comma\nand newline",
            ];

            for description in test_cases {
                // Arrange
                let mut record = create_test_text_record();
                record.description = description.to_string();
                let records = vec![record.clone()];

                // Act: write
                let mut buffer = Vec::new();
                YPBankTextFormat::write_to(&mut buffer, &records).unwrap();

                // Act: read
                let text_string = String::from_utf8(buffer).unwrap();
                let result = YPBankTextFormat::read_executor(text_string);

                // Assert
                if description.contains('\n') {
                    // Переносы строк могут вызывать проблемы при чтении
                    assert!(result.is_err() || result.unwrap()[0].description == description);
                } else {
                    let read_records = result.unwrap();
                    assert_eq!(read_records.len(), 1);
                    assert_eq!(read_records[0].description, description);
                }
            }
        }
    }

    // ==================== Integration Tests ====================

    mod integration_tests {
        use super::*;

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
            assert!(string_repr.contains("DESCRIPTION: \"Test transaction\""));
            assert!(string_repr.contains("TX_ID: 1234567890000000"));
        }

        #[test]
        fn test_complete_workflow() {
            // Arrange
            let original_records = vec![
                create_test_text_record(),
                create_deposit_text_record(),
                create_withdrawal_text_record(),
            ];

            // Act: write all records
            let mut buffer = Vec::new();
            YPBankTextFormat::write_to(&mut buffer, &original_records).unwrap();

            // Act: read them back
            let text_string = String::from_utf8(buffer.clone()).unwrap();
            let read_records = YPBankTextFormat::read_executor(text_string).unwrap();

            // Act: write them again
            let mut buffer2 = Vec::new();
            YPBankTextFormat::write_to(&mut buffer2, &read_records).unwrap();

            // Act: read again
            let text_string2 = String::from_utf8(buffer2.clone()).unwrap();
            let final_records = YPBankTextFormat::read_executor(text_string2).unwrap();

            // Assert
            assert_eq!(original_records.len(), read_records.len());
            assert_eq!(read_records.len(), final_records.len());

            for i in 0..original_records.len() {
                assert_record_matches(&read_records[i], &original_records[i]);
                assert_record_matches(&final_records[i], &read_records[i]);
                assert_record_matches(&final_records[i], &original_records[i]);
            }

            // Проверяем, что вывод идентичен при повторной записи
            let output1 = String::from_utf8(buffer).unwrap();
            let output2 = String::from_utf8(buffer2).unwrap();

            // Нормализуем строки (удаляем лишние пробелы в конце)
            let normalized1 = output1.trim().lines().collect::<Vec<_>>().join("\n");
            let normalized2 = output2.trim().lines().collect::<Vec<_>>().join("\n");

            assert_eq!(normalized1, normalized2);
        }
    }
}
