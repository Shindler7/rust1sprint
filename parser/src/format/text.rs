//! Запись и чтение файлов формата *.txt.

use crate::errors::ParseError;
use crate::format::tools::LineUtils;
use crate::models::{TxType, YPBankTextFormat, YPBankTransaction};
use crate::traits::YPBankIO;
use regex::Regex;
use std::collections::HashMap;
use std::io::{Error, Read, Write};

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

            if line.is_hash_marker() {
                if block_buffer.is_empty() {
                    // Первая строка. Начинаем наполнять буфер.
                    let title = Self::parse_title_block(line, count)?;
                    block_buffer.push(title);
                    continue;
                }
                // Буфер собрали уже. Надо отдать его на обработку и обнулить.
                let block_data = Self::parse_block(&block_buffer)?;
                transaction.push(block_data.try_into()?);
                block_buffer.clear()
            } else {
                if block_buffer.is_empty() {
                    return Err(ParseError::parse_error("Некорректная строка", count, 0));
                }

                block_buffer.push(line.to_string());
            }

            println!("{} {}", count, line);
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
    fn parse_block(block: &Vec<String>) -> Result<YPBankTextFormat, ParseError> {
        let mut fields = HashMap::new();

        for (count, line) in block.iter().enumerate() {
            if let Some((key, value)) = line.split_into_key_value() {
                fields.insert(key, value);
            } else {
                return Err(ParseError::parse_error(
                    format!("Неверный формат строки txt: {}", line),
                    count,
                    0,
                ));
            }
        }

        Ok()
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
