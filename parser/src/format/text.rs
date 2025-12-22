//! Запись и чтение файлов формата *.txt.

use crate::errors::ParseError;
use crate::format::tools::LineUtils;
use crate::models::{YPBankTextFormat, YPBankTransaction};
use crate::traits::YPBankIO;
use regex::Regex;
use std::collections::HashMap;
use std::io::{Read, Write};

pub struct TxtFormatIO {}

impl YPBankIO for TxtFormatIO {
    /// Чтение данных в формате TXT.
    fn read<R: Read>(reader: &mut R) -> Result<Vec<YPBankTransaction>, ParseError> {
        let mut buffer = String::new();
        reader.read_to_string(&mut buffer)?;

        let mut transaction: Vec<YPBankTransaction> = Vec::new();

        let mut block_buffer: Vec<String> = Vec::new();
        for (count, line) in buffer.lines().enumerate() {
            if line.is_empty_line() {
                continue;
            }

            match (block_buffer.is_empty(), line.is_hash_marker()) {
                (true, true) => {
                    // Начало блока.
                    let title = Self::parse_title_block(line, count)?;
                    block_buffer.push(title);
                }
                (false, true) => {
                    // Буфер собрали. Надо отдать его на обработку и обнулить.
                    let block_data = Self::parse_block(&block_buffer, count)?;
                    transaction.push(block_data.try_into()?);
                    block_buffer.clear(); // Обработанные данные.

                    let title = Self::parse_title_block(line, count)?; // Новый цикл.
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
            transaction.push(block_data.try_into()?);
        }

        if transaction.is_empty() {
            return Err(ParseError::EmptyData);
        }

        Ok(transaction)
    }

    /// Запись данных в формате TXT.
    fn write<W: Write>(writer: W, records: YPBankTransaction) -> Result<(), ParseError> {
        todo!()
    }
}

impl TxtFormatIO {
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
    fn parse_title_block(line: &str, count_line: usize) -> Result<String, ParseError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Создание строки одного корректного блока
    fn sample_block(index: usize) -> String {
        format!(
            "# Record {index} (DEPOSIT)\n\
TX_TYPE: DEPOSIT\n\
TO_USER_ID: 1\n\
FROM_USER_ID: 0\n\
TIMESTAMP: 1633036860000\n\
DESCRIPTION: \"Test description\"\n\
TX_ID: 1234567890000000\n\
AMOUNT: 1000\n\
STATUS: SUCCESS\n"
        )
    }

    #[test]
    fn test_read_single_valid_block() {
        let input = sample_block(1);
        let mut cursor = Cursor::new(input);

        let result = TxtFormatIO::read(&mut cursor);
        assert!(result.is_ok());

        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 1);
    }

    #[test]
    fn test_read_multiple_blocks() {
        let input = format!("{}\n{}", sample_block(1), sample_block(2));
        let mut cursor = Cursor::new(input);

        let result = TxtFormatIO::read(&mut cursor);
        assert!(result.is_ok());

        let txs = result.unwrap();
        assert_eq!(txs.len(), 2);
    }

    #[test]
    fn test_read_empty_input() {
        let mut cursor = Cursor::new("");
        let result = TxtFormatIO::read(&mut cursor);

        assert!(matches!(result, Err(ParseError::EmptyData)));
    }

    #[test]
    fn test_invalid_line_before_header() {
        let input = "TX_TYPE: DEPOSIT\n# Record 1 (DEPOSIT)\n...";
        let mut cursor = Cursor::new(input);
        let result = TxtFormatIO::read(&mut cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_incorrect_key_in_block() {
        let input = "\
# Record 1 (DEPOSIT)
TX_TYPE: DEPOSIT
UNKNOWN_FIELD: 123
TO_USER_ID: 1
FROM_USER_ID: 0
TIMESTAMP: 1633036860000
DESCRIPTION: \"Test description\"
TX_ID: 1234567890000000
AMOUNT: 1000
STATUS: SUCCESS
";
        let mut cursor = Cursor::new(input);
        let result = TxtFormatIO::read(&mut cursor);

        assert!(result.is_err()); // Ошибка должна быть из-за `UNKNOWN_FIELD`
    }
}
