//! Запись и чтение файлов бинарного формата.

use crate::errors::ParseError;
use crate::models::YPBankBinFormat;
use crate::models::{TxStatus, TxType};
use std::io::{ErrorKind, Read, Write};

const MAGIC_SIZE: usize = 4;
const MAGIC: [u8; 4] = [0x59, 0x50, 0x42, 0x4E];

impl YPBankBinFormat {
    /// Чтение данных в бинарном формате.
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Vec<Self>, ParseError> {
        let mut records: Vec<Self> = Vec::new();

        let mut magic_buf = [0u8; MAGIC_SIZE];
        loop {
            match reader.read_exact(&mut magic_buf) {
                Ok(_) => {}
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(e) => return Err(ParseError::io_error(e, "Ошибка чтения бинарного файла")),
            }

            if magic_buf != MAGIC {
                return Err(ParseError::parse_error(
                    format!(
                        "Некорректный идентификатор Magic: {:?} (ожидается: {:?})",
                        magic_buf, MAGIC
                    ),
                    0,
                    0,
                ));
            }

            let record = Self::read_executor(reader)?;
            records.push(record);
        }

        Ok(records)
    }

    /// Читает одну запись из потока.
    fn read_executor<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        let record_size = Self::read_u32be(reader)?;
        let mut body = vec![0u8; record_size as usize];
        reader.read_exact(&mut body)?;
        let mut cursor = &body[..];
        let record = Self::new_from_cursor(&mut cursor)?;

        Ok(record)
    }

    /// Запись данных в бинарном формате.
    pub fn write_to<W: Write>(mut writer: W, records: &[Self]) -> Result<(), ParseError> {
        for record in records {
            // TX_ID
            let mut body = Vec::new();
            body.extend(record.tx_id.to_be_bytes());

            // TX_TYPE
            let tx_type_byte = record.tx_type.clone().as_u8();
            body.push(tx_type_byte);

            // FROM_USER
            let from_user = match record.tx_type {
                TxType::Deposit => 0,
                _ => record.from_user_id,
            };
            body.extend(from_user.to_be_bytes());

            // TO_USER
            let to_user = match record.tx_type {
                TxType::Withdrawal => 0,
                _ => record.to_user_id,
            };
            body.extend(to_user.to_be_bytes());

            // AMOUNT
            body.extend(record.amount.to_be_bytes());

            // TIMESTAMP
            body.extend(record.timestamp.to_be_bytes());

            // STATUS
            let status = record.status.clone().as_u8();
            body.push(status);

            // DESC_LEN + DESCRIPTION
            let desc_bytes = match &record.description {
                Some(desc) => desc.as_bytes(),
                None => &[],
            };
            let desc_len = desc_bytes.len() as u32;

            body.extend(desc_len.to_be_bytes());
            body.extend(desc_bytes);

            // MAGIC & RECORD_SIZE
            writer.write_all(&MAGIC)?;
            writer.write_all(&(body.len() as u32).to_be_bytes())?;

            // Записать всё накопленное.
            writer.write_all(&body)?;
        }

        Ok(())
    }

    fn read_u8<R: Read>(reader: &mut R) -> Result<u8, ParseError> {
        let mut buf = [0u8; 1];
        reader
            .read_exact(&mut buf)
            .map_err(|_| ParseError::parse_bin_error("Не удалось прочитать u8"))?;
        Ok(buf[0])
    }

    fn read_u32be<R: Read>(reader: &mut R) -> Result<u32, ParseError> {
        let mut buf = [0u8; 4];
        reader
            .read_exact(&mut buf)
            .map_err(|_| ParseError::parse_bin_error("Не удалось прочитать u32 (Big Endian)"))?;
        Ok(u32::from_be_bytes(buf))
    }

    fn read_u64_be<R: Read>(reader: &mut R) -> Result<u64, ParseError> {
        let mut buf = [0u8; 8];
        reader
            .read_exact(&mut buf)
            .map_err(|_| ParseError::parse_bin_error("Не удалось прочитать u64 (Big Endian)"))?;
        Ok(u64::from_be_bytes(buf))
    }

    fn read_i64_be<R: Read>(reader: &mut R) -> Result<i64, ParseError> {
        let mut buf = [0u8; 8];
        reader
            .read_exact(&mut buf)
            .map_err(|_| ParseError::parse_bin_error("Не удалось прочитать i64 (Big Endian)"))?;
        Ok(i64::from_be_bytes(buf))
    }

    fn new_from_cursor<R: Read>(cursor: &mut R) -> Result<Self, ParseError> {
        let tx_id = Self::read_u64_be(cursor)?;
        let tx_type_byte = Self::read_u8(cursor)?;
        let tx_type = TxType::from_u8(tx_type_byte)
            .ok_or_else(|| ParseError::parse_bin_error("Некорректный TX_TYPE"))?;

        let from_user_id = Self::read_u64_be(cursor)?;
        let to_user_id = Self::read_u64_be(cursor)?;
        let amount = Self::read_i64_be(cursor)?;
        let timestamp = Self::read_u64_be(cursor)?;
        let status_byte = Self::read_u8(cursor)?;
        let status = TxStatus::from_u8(status_byte)
            .ok_or_else(|| ParseError::parse_bin_error("Некорректный TX_STATUS"))?;
        let desc_len = Self::read_u32be(cursor)?;
        let description = if desc_len > 0 {
            let mut desc_buf = vec![0u8; desc_len as usize];
            cursor.read_exact(&mut desc_buf)?;
            Some(
                String::from_utf8(desc_buf)
                    .map_err(|_| ParseError::parse_bin_error("Описание невалидная строка UTF-8"))?,
            )
        } else {
            None
        };

        Ok(Self {
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            desc_len,
            description,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TxStatus, TxType};
    use std::io::Cursor;

    fn create_test_record(description: Option<&str>) -> YPBankBinFormat {
        YPBankBinFormat {
            tx_id: 123456789,
            tx_type: TxType::Transfer,
            from_user_id: 1001,
            to_user_id: 1002,
            amount: 50000,
            timestamp: 1633046400, // 1 Oct 2021
            status: TxStatus::Success,
            desc_len: description.map(|s| s.len() as u32).unwrap_or(0),
            description: description.map(|s| s.to_string()),
        }
    }

    fn create_deposit_record() -> YPBankBinFormat {
        YPBankBinFormat {
            tx_id: 987654321,
            tx_type: TxType::Deposit,
            from_user_id: 0, // will be ignored in write
            to_user_id: 1003,
            amount: 100000,
            timestamp: 1633046401,
            status: TxStatus::Pending,
            desc_len: 0,
            description: None,
        }
    }

    fn create_withdrawal_record() -> YPBankBinFormat {
        YPBankBinFormat {
            tx_id: 555555555,
            tx_type: TxType::Withdrawal,
            from_user_id: 1004,
            to_user_id: 0, // will be ignored in write
            amount: -25000,
            timestamp: 1633046402,
            status: TxStatus::Failure,
            desc_len: 10,
            description: Some("Withdrawal".to_string()),
        }
    }

    #[test]
    fn test_write_read_single_record() {
        // Arrange
        let record = create_test_record(Some("Test transaction"));

        // Act
        let mut buffer = Vec::new();
        YPBankBinFormat::write_to(&mut buffer, &[record.clone()]).unwrap();
        let mut cursor = Cursor::new(buffer);
        let result = YPBankBinFormat::read_from(&mut cursor).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        let read_record = &result[0];
        assert_eq!(read_record.tx_id, record.tx_id);
        assert_eq!(read_record.tx_type, record.tx_type);
        assert_eq!(read_record.from_user_id, record.from_user_id);
        assert_eq!(read_record.to_user_id, record.to_user_id);
        assert_eq!(read_record.amount, record.amount);
        assert_eq!(read_record.timestamp, record.timestamp);
        assert_eq!(read_record.status, record.status);
        assert_eq!(read_record.description, record.description);
    }

    #[test]
    fn test_write_read_multiple_records() {
        // Arrange
        let records = vec![
            create_test_record(Some("First")),
            create_deposit_record(),
            create_withdrawal_record(),
        ];

        // Act
        let mut buffer = Vec::new();
        YPBankBinFormat::write_to(&mut buffer, &records).unwrap();
        let mut cursor = Cursor::new(buffer);
        let result = YPBankBinFormat::read_from(&mut cursor).unwrap();

        // Assert
        assert_eq!(result.len(), 3);

        // Проверяем, что для депозита from_user_id при чтении корректно восстановлен
        assert_eq!(result[1].tx_type, TxType::Deposit);

        // Проверяем, что для withdrawal to_user_id при чтении корректно восстановлен
        assert_eq!(result[2].tx_type, TxType::Withdrawal);
    }

    #[test]
    fn test_write_read_empty_description() {
        // Arrange
        let record = create_test_record(None);

        // Act
        let mut buffer = Vec::new();
        YPBankBinFormat::write_to(&mut buffer, &[record.clone()]).unwrap();
        let mut cursor = Cursor::new(buffer);
        let result = YPBankBinFormat::read_from(&mut cursor).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert!(result[0].description.is_none());
    }

    #[test]
    fn test_write_read_long_description() {
        // Arrange
        let long_desc = "A".repeat(500);
        let record = YPBankBinFormat {
            desc_len: 500,
            description: Some(long_desc.clone()),
            ..create_test_record(None)
        };

        // Act
        let mut buffer = Vec::new();
        YPBankBinFormat::write_to(&mut buffer, &[record.clone()]).unwrap();
        let mut cursor = Cursor::new(buffer);
        let result = YPBankBinFormat::read_from(&mut cursor).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, Some(long_desc));
    }

    #[test]
    fn test_invalid_magic() {
        // Arrange - создаем данные с неправильным magic
        let mut invalid_data = vec![0x00, 0x00, 0x00, 0x00]; // неправильный magic
        invalid_data.extend_from_slice(&8u32.to_be_bytes()); // размер записи
        invalid_data.extend_from_slice(&[0u8; 8]); // tx_id

        // Act & Assert
        let mut cursor = Cursor::new(invalid_data);
        let result = YPBankBinFormat::read_from(&mut cursor);
        assert!(result.is_err());
        assert!(matches!(result, Err(ParseError::ParseError { .. })));
    }

    #[test]
    fn test_corrupted_record_size() {
        // Arrange - данные с magic, но без размера записи
        let corrupted_data = MAGIC.to_vec();
        // Не добавляем размер записи

        // Act & Assert
        let mut cursor = Cursor::new(corrupted_data);
        let result = YPBankBinFormat::read_from(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_corrupted_body() {
        // Arrange - данные с magic и размером, но неполным телом
        let mut corrupted_data = MAGIC.to_vec();
        corrupted_data.extend_from_slice(&100u32.to_be_bytes()); // большой размер
        corrupted_data.extend_from_slice(&[0u8; 50]); // только 50 байт вместо 100

        // Act & Assert
        let mut cursor = Cursor::new(corrupted_data);
        let result = YPBankBinFormat::read_from(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_utf8_description() {
        // Arrange - создаем валидную запись, но с невалидным UTF-8 в описании
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&MAGIC);

        // Размер тела: tx_id(8) + tx_type(1) + from_user(8) + to_user(8) +
        // amount(8) + timestamp(8) + status(1) + desc_len(4) + desc(2) = 48
        buffer.extend_from_slice(&48u32.to_be_bytes());

        // Тело записи
        buffer.extend_from_slice(&123u64.to_be_bytes()); // tx_id
        buffer.push(TxType::Transfer.as_u8()); // tx_type
        buffer.extend_from_slice(&1001u64.to_be_bytes()); // from_user
        buffer.extend_from_slice(&1002u64.to_be_bytes()); // to_user
        buffer.extend_from_slice(&50000i64.to_be_bytes()); // amount
        buffer.extend_from_slice(&1633046400u64.to_be_bytes()); // timestamp
        buffer.push(TxStatus::Success.as_u8()); // status
        buffer.extend_from_slice(&2u32.to_be_bytes()); // desc_len = 2
        buffer.extend_from_slice(&[0xFF, 0xFE]); // невалидный UTF-8

        // Act & Assert
        let mut cursor = Cursor::new(buffer);
        let result = YPBankBinFormat::read_from(&mut cursor);
        assert!(result.is_err());
        assert!(matches!(result, Err(ParseError::ParseBinaryError { .. })));
    }

    #[test]
    fn test_invalid_tx_type() {
        // Arrange - создаем запись с невалидным типом транзакции
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&MAGIC);
        buffer.extend_from_slice(&25u32.to_be_bytes()); // размер тела

        // Тело с невалидным tx_type
        buffer.extend_from_slice(&123u64.to_be_bytes()); // tx_id
        buffer.push(99); // невалидный tx_type (99)
        // остальные поля не важны для этого теста

        // Act & Assert
        let mut cursor = Cursor::new(buffer);
        let result = YPBankBinFormat::read_from(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_status() {
        // Arrange - создаем запись с невалидным статусом
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&MAGIC);
        buffer.extend_from_slice(&41u32.to_be_bytes()); // размер тела

        // Тело с невалидным status
        buffer.extend_from_slice(&123u64.to_be_bytes()); // tx_id
        buffer.push(TxType::Transfer.as_u8()); // tx_type
        buffer.extend_from_slice(&1001u64.to_be_bytes()); // from_user
        buffer.extend_from_slice(&1002u64.to_be_bytes()); // to_user
        buffer.extend_from_slice(&50000i64.to_be_bytes()); // amount
        buffer.extend_from_slice(&1633046400u64.to_be_bytes()); // timestamp
        buffer.push(99); // невалидный status (99)
        buffer.extend_from_slice(&0u32.to_be_bytes()); // desc_len = 0

        // Act & Assert
        let mut cursor = Cursor::new(buffer);
        let result = YPBankBinFormat::read_from(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_file() {
        // Arrange
        let empty_data = Vec::new();

        // Act
        let mut cursor = Cursor::new(empty_data);
        let result = YPBankBinFormat::read_from(&mut cursor).unwrap();

        // Assert
        assert!(result.is_empty());
    }

    #[test]
    fn test_deposit_write_read() {
        // Arrange
        let deposit = create_deposit_record();

        // Act
        let mut buffer = Vec::new();
        YPBankBinFormat::write_to(&mut buffer, &[deposit.clone()]).unwrap();
        let mut cursor = Cursor::new(buffer);
        let result = YPBankBinFormat::read_from(&mut cursor).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tx_type, TxType::Deposit);
        assert_eq!(result[0].from_user_id, 0); // Для депозита from_user должно быть 0
        assert_eq!(result[0].to_user_id, deposit.to_user_id);
    }

    #[test]
    fn test_withdrawal_write_read() {
        // Arrange
        let withdrawal = create_withdrawal_record();

        // Act
        let mut buffer = Vec::new();
        YPBankBinFormat::write_to(&mut buffer, &[withdrawal.clone()]).unwrap();
        let mut cursor = Cursor::new(buffer);
        let result = YPBankBinFormat::read_from(&mut cursor).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tx_type, TxType::Withdrawal);
        assert_eq!(result[0].from_user_id, withdrawal.from_user_id);
        assert_eq!(result[0].to_user_id, 0); // Для withdrawal to_user должно быть 0
    }

    #[test]
    fn test_negative_amount() {
        // Arrange
        let record = YPBankBinFormat {
            amount: -1000,
            ..create_test_record(Some("Negative amount"))
        };

        // Act
        let mut buffer = Vec::new();
        YPBankBinFormat::write_to(&mut buffer, &[record.clone()]).unwrap();
        let mut cursor = Cursor::new(buffer);
        let result = YPBankBinFormat::read_from(&mut cursor).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].amount, -1000);
    }

    #[test]
    fn test_all_tx_types() {
        // Arrange
        let records = vec![
            YPBankBinFormat {
                tx_type: TxType::Deposit,
                ..create_test_record(None)
            },
            YPBankBinFormat {
                tx_type: TxType::Withdrawal,
                ..create_test_record(None)
            },
            YPBankBinFormat {
                tx_type: TxType::Transfer,
                ..create_test_record(None)
            },
        ];

        // Act
        let mut buffer = Vec::new();
        YPBankBinFormat::write_to(&mut buffer, &records).unwrap();
        let mut cursor = Cursor::new(buffer);
        let result = YPBankBinFormat::read_from(&mut cursor).unwrap();

        // Assert
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].tx_type, TxType::Deposit);
        assert_eq!(result[1].tx_type, TxType::Withdrawal);
        assert_eq!(result[2].tx_type, TxType::Transfer);
    }

    #[test]
    fn test_all_statuses() {
        // Arrange
        let records = vec![
            YPBankBinFormat {
                status: TxStatus::Pending,
                ..create_test_record(None)
            },
            YPBankBinFormat {
                status: TxStatus::Success,
                ..create_test_record(None)
            },
            YPBankBinFormat {
                status: TxStatus::Failure,
                ..create_test_record(None)
            },
        ];

        // Act
        let mut buffer = Vec::new();
        YPBankBinFormat::write_to(&mut buffer, &records).unwrap();
        let mut cursor = Cursor::new(buffer);
        let result = YPBankBinFormat::read_from(&mut cursor).unwrap();

        // Assert
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].status, TxStatus::Pending);
        assert_eq!(result[1].status, TxStatus::Success);
        assert_eq!(result[2].status, TxStatus::Failure);
    }

    #[test]
    fn test_large_values() {
        // Arrange
        let record = YPBankBinFormat {
            tx_id: u64::MAX,
            from_user_id: u64::MAX,
            to_user_id: u64::MAX,
            amount: i64::MAX,
            timestamp: u64::MAX,
            ..create_test_record(Some("Large values"))
        };

        // Act
        let mut buffer = Vec::new();
        YPBankBinFormat::write_to(&mut buffer, &[record.clone()]).unwrap();
        let mut cursor = Cursor::new(buffer);
        let result = YPBankBinFormat::read_from(&mut cursor).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tx_id, u64::MAX);
        assert_eq!(result[0].amount, i64::MAX);
        assert_eq!(result[0].timestamp, u64::MAX);
    }

    #[test]
    fn test_tx_type_enum_values() {
        // Проверяем, что числовые значения enum соответствуют ожидаемым
        assert_eq!(TxType::Deposit.as_u8(), 0);
        assert_eq!(TxType::Transfer.as_u8(), 1);
        assert_eq!(TxType::Withdrawal.as_u8(), 2);
    }

    #[test]
    fn test_tx_status_enum_values() {
        // Проверяем, что числовые значения enum соответствуют ожидаемым
        assert_eq!(TxStatus::Success.as_u8(), 0);
        assert_eq!(TxStatus::Failure.as_u8(), 1);
        assert_eq!(TxStatus::Pending.as_u8(), 2);
    }

    #[test]
    fn test_deposit_zero_from_user_on_write() {
        // Arrange
        let deposit = YPBankBinFormat {
            tx_type: TxType::Deposit,
            from_user_id: 9999, // Должно быть проигнорировано при записи
            to_user_id: 1001,
            ..create_test_record(None)
        };

        // Act
        let mut buffer = Vec::new();
        YPBankBinFormat::write_to(&mut buffer, &[deposit.clone()]).unwrap();

        // Проверяем, что в записанных данных from_user = 0
        // Пропускаем magic (4) и record_size (4) = 8 байт
        // tx_id (8) + tx_type (1) = 9 байт, from_user начинается с 17-го байта
        let from_user_bytes = &buffer[17..25];
        let from_user = u64::from_be_bytes(from_user_bytes.try_into().unwrap());

        // Assert
        assert_eq!(from_user, 0);
    }

    #[test]
    fn test_withdrawal_zero_to_user_on_write() {
        // Arrange
        let withdrawal = YPBankBinFormat {
            tx_type: TxType::Withdrawal,
            from_user_id: 1001,
            to_user_id: 9999, // Должно быть проигнорировано при записи
            ..create_test_record(None)
        };

        // Act
        let mut buffer = Vec::new();
        YPBankBinFormat::write_to(&mut buffer, &[withdrawal.clone()]).unwrap();

        // Проверяем, что в записанных данных to_user = 0
        // Пропускаем: magic(4) + record_size(4) + tx_id(8) + tx_type(1) + from_user(8) = 25 байт
        // to_user начинается с 25-го байта
        let to_user_bytes = &buffer[25..33];
        let to_user = u64::from_be_bytes(to_user_bytes.try_into().unwrap());

        // Assert
        assert_eq!(to_user, 0);
    }

    #[test]
    fn test_transfer_both_users_on_write() {
        // Arrange
        let transfer = YPBankBinFormat {
            tx_type: TxType::Transfer,
            from_user_id: 1001,
            to_user_id: 1002,
            ..create_test_record(None)
        };

        // Act
        let mut buffer = Vec::new();
        YPBankBinFormat::write_to(&mut buffer, &[transfer.clone()]).unwrap();

        // Проверяем from_user
        let from_user_bytes = &buffer[17..25];
        let from_user = u64::from_be_bytes(from_user_bytes.try_into().unwrap());

        // Проверяем to_user
        let to_user_bytes = &buffer[25..33];
        let to_user = u64::from_be_bytes(to_user_bytes.try_into().unwrap());

        // Assert
        assert_eq!(from_user, 1001);
        assert_eq!(to_user, 1002);
    }
}
