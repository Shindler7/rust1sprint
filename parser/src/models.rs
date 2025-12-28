//! Общие модели представления данных для чтения/записи, парсинга.

use crate::errors::ParseError;
use parser_macros::{TxDisplay, YPBankFields};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

/// Макрос преобразования структур [`YPBankCsvFormat`], [`YPBankTextFormat`] в универсальную,
/// и предусмотрены необходимые схожие проверки.
///
/// ## Amount
///
/// Поле `amount` преобразуется из `u64` в `i64`. При этом производится проверка на
/// переполнение, и если оно возникнет, выбросится [`ParseError::OverflowSize`].
///
/// Кроме того, в бинарном формате это поле со знаком (отрицательное для списаний), а в csv
/// и txt беззнаковое. В универсальной структуре используется знаковое поле, соответственно,
/// исходя из типа операции преобразуется и знак.
///
/// ## Примеры
///
/// ```
/// use parser::models::{TxStatus, TxType, YPBankCsvFormat, YPBankTextFormat, YPBankTransaction};
/// use parser::utils::get_timestamp;
///
/// let timestamp = get_timestamp();
///
/// let txt = YPBankTextFormat {
///     tx_id: 1000000000000011,
///     tx_type: TxType::Withdrawal,
///     from_user_id: 9223372036854775807,
///     to_user_id: 0,
///     amount: 1200,
///     timestamp,
///     status: TxStatus::Success,
///     description: "Record number 12".to_string()
/// };
///
/// let universal = YPBankTransaction::try_from(txt).unwrap();
/// let csv = YPBankCsvFormat::try_from(universal).unwrap();
/// ```
macro_rules! impl_try_from_yp_format_to_transaction {
    ($source_type:ty) => {
        impl TryFrom<$source_type> for YPBankTransaction {
            type Error = ParseError;

            fn try_from(source: $source_type) -> Result<Self, ParseError> {
                let mut amount: i64 = source
                    .amount
                    .try_into()
                    .map_err(|_| ParseError::over_flow_size("u64", "i64", source.amount))?;

                if matches!(source.tx_type, TxType::Transfer | TxType::Withdrawal) && amount > 0 {
                    amount = -amount;
                }

                Ok(YPBankTransaction {
                    tx_id: source.tx_id,
                    tx_type: source.tx_type,
                    from_user_id: source.from_user_id,
                    to_user_id: source.to_user_id,
                    amount,
                    timestamp: source.timestamp,
                    status: source.status,
                    description: source.description.into(),
                })
            }
        }
    };
}

/// Макрос преобразования структуры [`YPBankTransaction`] в структуры [`YPBankCsvFormat`]
/// и [`YPBankTextFormat`]. Для бинарной структуры создан отдельный метод, потому что внутри
/// предусматривается индивидуальная проработка с полями.
///
/// ## Amount
///
/// Знаковое поле `amount` применяется только в бинарном формате, а в csv и txt беззнаковый `u64`.
/// Для обеспечения единообразия данных, универсальная структура применяет знаковое поле, аналогично
/// формату `bin`. При преобразовании значение поля приводится к типу целевой структуры.
///
/// Возможно для макроса ложное предупреждение `PyCharm`.
macro_rules! impl_try_from_transaction_to_yp_format {
    ($dest_type:ident) => {
        impl TryFrom<YPBankTransaction> for $dest_type {
            type Error = ParseError;

            fn try_from(value: YPBankTransaction) -> Result<Self, ParseError> {
                let description = match value.description {
                    Some(d) => d,
                    None => "".to_string(),
                };

                let amount: u64 = value.amount.unsigned_abs();

                Ok($dest_type {
                    tx_id: value.tx_id,
                    tx_type: value.tx_type,
                    from_user_id: value.from_user_id,
                    to_user_id: value.to_user_id,
                    amount,
                    timestamp: value.timestamp,
                    status: value.status,
                    description,
                })
            }
        }
    };
}

/// Макрос поддержки формирования структур из текстовых значений.
macro_rules! get_field_in_map {
    ($map:expr, $key:expr, $ty:ty) => {
        $map.get($key)
            .ok_or(ParseError::IncorrectField {
                key: $key.to_string(),
            })?
            .parse::<$ty>()
            .map_err(|_| ParseError::IncorrectField {
                key: $key.to_string(),
            })?
    };
}

/// Тип транзакции.
#[repr(u8)]
#[derive(Debug, TxDisplay, Clone, PartialEq)]
pub enum TxType {
    Deposit = 0,
    Transfer = 1,
    Withdrawal = 2,
}

#[repr(u8)]
#[derive(Debug, TxDisplay, Clone, PartialEq)]
pub enum TxStatus {
    Success = 0,
    Failure = 1,
    Pending = 2,
}

/// Универсальная структура представления данных для записи/чтения, позволяющая парсить
/// исходные сведения, а также при извлечении их из хранения.
#[derive(Debug, Clone, PartialEq, YPBankFields)]
pub struct YPBankTransaction {
    pub tx_id: u64,
    pub tx_type: TxType,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub amount: i64,
    pub timestamp: u64,
    pub status: TxStatus,
    pub description: Option<String>,
}

impl_try_from_yp_format_to_transaction!(YPBankCsvFormat);
impl_try_from_yp_format_to_transaction!(YPBankTextFormat);
impl_try_from_yp_format_to_transaction!(YPBankBinFormat);

/// Текстовый файл с разделителями-запятыми (`CSV`), предназначенный для хранения
/// данных о транзакциях. Файл имеет строгую структуру: обязательная строка заголовка
/// и последующие строки, каждая из которых представляет одну транзакцию.
///
/// Файл должен быть в кодировке `UTF-8`.
///
/// ## Заголовок
///
/// Первая строка файла всегда должна содержать заголовок с именами полей. Заголовок должен точно соответствовать следующей строке:
///
/// ```plain
/// TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
/// ```
///
/// ## Записи данных
///
/// Каждая строка после заголовка представляет одну транзакцию. Поля в строке разделены
/// запятыми. Пустые строки в файле игнорируются парсером.
///
/// ## Пример
///
/// ```csv
/// TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
/// 1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding"
/// 1002,TRANSFER,501,502,15000,1672534800000,FAILURE,"Payment for services, invoice #123"
/// 1003,WITHDRAWAL,502,0,1000,1672538400000,PENDING,"ATM withdrawal"
/// ```
#[derive(Debug, YPBankFields, PartialEq, Clone)]
pub struct YPBankCsvFormat {
    pub tx_id: u64,
    pub tx_type: TxType,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub amount: u64,
    pub timestamp: u64,
    pub status: TxStatus,
    pub description: String,
}

impl_try_from_transaction_to_yp_format!(YPBankCsvFormat);

impl YPBankCsvFormat {
    pub fn new_from_map(fields: &HashMap<String, String>) -> Result<Self, ParseError> {
        Ok(Self {
            tx_id: get_field_in_map!(fields, "TX_ID", u64),
            tx_type: get_field_in_map!(fields, "TX_TYPE", TxType),
            from_user_id: get_field_in_map!(fields, "FROM_USER_ID", u64),
            to_user_id: get_field_in_map!(fields, "TO_USER_ID", u64),
            amount: get_field_in_map!(fields, "AMOUNT", u64),
            timestamp: get_field_in_map!(fields, "TIMESTAMP", u64),
            status: get_field_in_map!(fields, "STATUS", TxStatus),
            description: get_field_in_map!(fields, "DESCRIPTION", String),
        })
    }
}

/// Бинарный формат YPBankBin — это компактное, бинарное представление тех же данных
/// о транзакциях, которые описаны в текстовом формате `YPBankText`.
///
/// Файл представляет собой последовательный поток записей; каждая запись начинается
/// с небольшого заголовка, упрощающего парсинг и проверку.
///
/// Все многобайтовые целые числа кодируются в формате big-endian.
///
/// ## Структура файла:
///
/// ```plain
/// [ЗАГОЛОВОК][ТЕЛО][ЗАГОЛОВОК][ТЕЛО]...
/// ```
///
/// Наличие значения `MAGIC` в начале каждой записи позволяет читателю повторно
/// синхронизироваться в случае потери границы записи или повреждения данных.
#[derive(Debug, YPBankFields, PartialEq, Clone)]
pub struct YPBankBinFormat {
    pub tx_id: u64,
    pub tx_type: TxType,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub amount: i64,
    pub timestamp: u64,
    pub status: TxStatus,
    /// Длина следующего описания `description` в кодировке UTF-8.
    pub desc_len: u32,
    /// Необязательное текстовое описание. Если описание отсутствует, `DESC_LEN` равен `0`.
    pub description: Option<String>,
}

impl TryFrom<YPBankTransaction> for YPBankBinFormat {
    type Error = ParseError;
    fn try_from(value: YPBankTransaction) -> Result<Self, Self::Error> {
        let desc_len = match &value.description {
            Some(d) => { u32::try_from(d.len()) }
                .map_err(|_| ParseError::over_flow_size("usize", "u32", d))?,
            None => 0,
        };

        Ok(Self {
            tx_id: value.tx_id,
            tx_type: value.tx_type,
            from_user_id: value.from_user_id,
            to_user_id: value.to_user_id,
            amount: value.amount,
            timestamp: value.timestamp,
            status: value.status,
            desc_len,
            description: value.description,
        })
    }
}

/// Формат файла `YPBankText` представляет собой текстовую структуру,
/// используемую для хранения записей о транзакциях в системе YPBank.
///
/// Каждый файл состоит из последовательных записей о транзакциях.
///
/// Дополнительно:
/// - Поля могут располагаться в любом порядке.
/// - Каждое поле встречается ровно один раз.
/// - Записи о транзакциях разделяются пустыми строками.
/// - Файл может содержать однострочные комментарии, которые начинаются с "#";
///   эти строки игнорируются при парсинге.
///
/// ## Пример содержимого файла:
/// ```plain
// # Record 1 (Deposit)
/// TX_ID: 1234567890123456
/// TX_TYPE: DEPOSIT
/// FROM_USER_ID: 0
/// TO_USER_ID: 9876543210987654
/// AMOUNT: 10000
/// TIMESTAMP: 1633036800000
/// STATUS: SUCCESS
/// DESCRIPTION: "Terminal deposit"
/// ```
#[derive(Debug, YPBankFields, PartialEq, Clone)]
pub struct YPBankTextFormat {
    pub tx_id: u64,
    pub tx_type: TxType,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub amount: u64,
    pub timestamp: u64,
    pub status: TxStatus,
    pub description: String,
}

impl_try_from_transaction_to_yp_format!(YPBankTextFormat);

impl Display for YPBankTextFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TX_ID: {}", self.tx_id)?;
        writeln!(f, "TX_TYPE: {}", self.tx_type)?;
        writeln!(f, "FROM_USER_ID: {}", self.from_user_id)?;
        writeln!(f, "TO_USER_ID: {}", self.to_user_id)?;
        writeln!(f, "AMOUNT: {}", self.amount)?;
        writeln!(f, "TIMESTAMP: {}", self.timestamp)?;
        writeln!(f, "STATUS: {}", self.status)?;
        writeln!(f, "DESCRIPTION: \"{}\"", self.description)
    }
}

impl YPBankTextFormat {
    /// Создаёт экземпляр структуры на основе данных из `HashMap`, где ключ и значение,
    /// соответственно, равны этим параметрам полей структуры.
    pub fn new_from_map(fields_map: HashMap<String, String>) -> Result<Self, ParseError> {
        Ok(Self {
            tx_id: get_field_in_map!(fields_map, "TX_ID", u64),
            tx_type: get_field_in_map!(fields_map, "TX_TYPE", TxType),
            from_user_id: get_field_in_map!(fields_map, "FROM_USER_ID", u64),
            to_user_id: get_field_in_map!(fields_map, "TO_USER_ID", u64),
            amount: get_field_in_map!(fields_map, "AMOUNT", u64),
            timestamp: get_field_in_map!(fields_map, "TIMESTAMP", u64),
            status: get_field_in_map!(fields_map, "STATUS", TxStatus),
            description: get_field_in_map!(fields_map, "DESCRIPTION", String),
        })
    }
}

#[cfg(test)]
mod conversion_tests {
    use super::*;
    use crate::models::{TxStatus, TxType};
    use std::collections::HashMap;

    // Вспомогательная функция для создания тестовой универсальной транзакции
    fn create_test_transaction() -> YPBankTransaction {
        YPBankTransaction {
            tx_id: 1234567890000000,
            tx_type: TxType::Transfer,
            from_user_id: 1001,
            to_user_id: 1002,
            amount: -50000, // Отрицательная сумма для Transfer
            timestamp: 1633046400,
            status: TxStatus::Success,
            description: Some("Test transaction".to_string()),
        }
    }

    #[test]
    fn test_csv_to_transaction_conversion() {
        // Arrange: создаем CSV запись с положительной суммой для Transfer
        let csv_record = YPBankCsvFormat {
            tx_id: 1234567890000000,
            tx_type: TxType::Transfer,
            from_user_id: 1001,
            to_user_id: 1002,
            amount: 50000, // Положительная сумма в CSV
            timestamp: 1633046400,
            status: TxStatus::Success,
            description: "Test transaction".to_string(),
        };

        // Act: преобразуем CSV в универсальную транзакцию
        let transaction: YPBankTransaction = csv_record.try_into().unwrap();

        // Assert: проверяем, что сумма стала отрицательной для Transfer
        assert_eq!(transaction.tx_id, 1234567890000000);
        assert_eq!(transaction.tx_type, TxType::Transfer);
        assert_eq!(transaction.amount, -50000); // Должно стать отрицательным
        assert_eq!(
            transaction.description,
            Some("Test transaction".to_string())
        );
    }

    #[test]
    fn test_text_to_transaction_conversion() {
        // Arrange: создаем текстовую запись с положительной суммой для Withdrawal
        let text_record = YPBankTextFormat {
            tx_id: 5555555550000000,
            tx_type: TxType::Withdrawal,
            from_user_id: 1004,
            to_user_id: 0,
            amount: 25000, // Положительная сумма в текстовом формате
            timestamp: 1633046402,
            status: TxStatus::Failure,
            description: "Withdrawal".to_string(),
        };

        // Act: преобразуем текстовую запись в универсальную транзакцию
        let transaction: YPBankTransaction = text_record.try_into().unwrap();

        // Assert: проверяем, что сумма стала отрицательной для Withdrawal
        assert_eq!(transaction.tx_id, 5555555550000000);
        assert_eq!(transaction.tx_type, TxType::Withdrawal);
        assert_eq!(transaction.amount, -25000); // Должно стать отрицательным
        assert_eq!(transaction.description, Some("Withdrawal".to_string()));
    }

    #[test]
    fn test_binary_to_transaction_conversion() {
        // Arrange: создаем бинарную запись
        let bin_record = YPBankBinFormat {
            tx_id: 9876543210000000,
            tx_type: TxType::Deposit,
            from_user_id: 0,
            to_user_id: 1003,
            amount: 100000, // Уже может быть отрицательной в бинарном формате
            timestamp: 1633046401,
            status: TxStatus::Pending,
            desc_len: 0,
            description: None,
        };

        // Act: преобразуем бинарную запись в универсальную транзакцию
        let transaction: YPBankTransaction = bin_record.try_into().unwrap();

        // Assert: проверяем, что сумма осталась положительной для Deposit
        assert_eq!(transaction.tx_id, 9876543210000000);
        assert_eq!(transaction.tx_type, TxType::Deposit);
        assert_eq!(transaction.amount, 100000); // Должно остаться положительным для Deposit
        assert_eq!(transaction.description, None);
    }

    #[test]
    fn test_transaction_to_csv_conversion() {
        // Arrange: создаем универсальную транзакцию
        let transaction = create_test_transaction();

        // Act: преобразуем универсальную транзакцию в CSV формат
        let csv_record: YPBankCsvFormat = transaction.try_into().unwrap();

        // Assert: проверяем преобразование
        assert_eq!(csv_record.tx_id, 1234567890000000);
        assert_eq!(csv_record.tx_type, TxType::Transfer);
        assert_eq!(csv_record.from_user_id, 1001);
        assert_eq!(csv_record.to_user_id, 1002);
        assert_eq!(csv_record.amount, 50000); // Абсолютное значение
        assert_eq!(csv_record.timestamp, 1633046400);
        assert_eq!(csv_record.status, TxStatus::Success);
        assert_eq!(csv_record.description, "Test transaction".to_string());
    }

    #[test]
    fn test_transaction_to_binary_conversion() {
        // Arrange: создаем универсальную транзакцию с пустым описанием
        let transaction = YPBankTransaction {
            tx_id: 9876543210000000,
            tx_type: TxType::Deposit,
            from_user_id: 0,
            to_user_id: 1003,
            amount: 100000,
            timestamp: 1633046401,
            status: TxStatus::Pending,
            description: None,
        };

        // Act: преобразуем универсальную транзакцию в бинарный формат
        let bin_record: YPBankBinFormat = transaction.try_into().unwrap();

        // Assert: проверяем преобразование
        assert_eq!(bin_record.tx_id, 9876543210000000);
        assert_eq!(bin_record.tx_type, TxType::Deposit);
        assert_eq!(bin_record.from_user_id, 0);
        assert_eq!(bin_record.to_user_id, 1003);
        assert_eq!(bin_record.amount, 100000);
        assert_eq!(bin_record.timestamp, 1633046401);
        assert_eq!(bin_record.status, TxStatus::Pending);
        assert_eq!(bin_record.desc_len, 0);
        assert_eq!(bin_record.description, None);
    }

    #[test]
    fn test_deposit_amount_remains_positive() {
        // Arrange: создаем CSV запись для Deposit
        let csv_record = YPBankCsvFormat {
            tx_id: 1111111110000000,
            tx_type: TxType::Deposit,
            from_user_id: 0,
            to_user_id: 1005,
            amount: 75000, // Положительная сумма
            timestamp: 1633046403,
            status: TxStatus::Success,
            description: "Deposit".to_string(),
        };

        // Act: преобразуем в универсальную транзакцию
        let transaction: YPBankTransaction = csv_record.try_into().unwrap();

        // Assert: для Deposit сумма должна остаться положительной
        assert_eq!(transaction.tx_type, TxType::Deposit);
        assert_eq!(transaction.amount, 75000); // Положительная
        assert_eq!(transaction.description, Some("Deposit".to_string()));
    }

    #[test]
    fn test_conversion_roundtrip_csv() {
        // Arrange: создаем исходную CSV запись
        let original_csv = YPBankCsvFormat {
            tx_id: 1234567890000000,
            tx_type: TxType::Transfer,
            from_user_id: 1001,
            to_user_id: 1002,
            amount: 50000,
            timestamp: 1633046400,
            status: TxStatus::Success,
            description: "Test transaction".to_string(),
        };

        // Act: CSV -> Transaction -> CSV
        let transaction: YPBankTransaction = original_csv.clone().try_into().unwrap();
        let roundtrip_csv: YPBankCsvFormat = transaction.try_into().unwrap();

        // Assert: проверяем, что после roundtrip получили ту же самую запись
        assert_eq!(original_csv.tx_id, roundtrip_csv.tx_id);
        assert_eq!(original_csv.tx_type, roundtrip_csv.tx_type);
        assert_eq!(original_csv.from_user_id, roundtrip_csv.from_user_id);
        assert_eq!(original_csv.to_user_id, roundtrip_csv.to_user_id);
        assert_eq!(original_csv.amount, roundtrip_csv.amount);
        assert_eq!(original_csv.timestamp, roundtrip_csv.timestamp);
        assert_eq!(original_csv.status, roundtrip_csv.status);
        assert_eq!(original_csv.description, roundtrip_csv.description);
    }

    #[test]
    fn test_conversion_with_empty_description() {
        // Arrange: создаем CSV запись с пустым описанием
        let csv_record = YPBankCsvFormat {
            tx_id: 2222222220000000,
            tx_type: TxType::Deposit,
            from_user_id: 0,
            to_user_id: 1006,
            amount: 1000,
            timestamp: 1633046404,
            status: TxStatus::Pending,
            description: "".to_string(), // Пустое описание
        };

        // Act: преобразуем в универсальную транзакцию
        let transaction: YPBankTransaction = csv_record.try_into().unwrap();

        // Assert: проверяем преобразование пустого описания
        assert_eq!(transaction.tx_id, 2222222220000000);
        assert_eq!(transaction.description, Some("".to_string())); // Some с пустой строкой
    }

    #[test]
    fn test_new_from_map_for_text_format() {
        // Arrange: создаем HashMap с данными
        let mut fields = HashMap::new();
        fields.insert("TX_ID".to_string(), "1234567890000000".to_string());
        fields.insert("TX_TYPE".to_string(), "TRANSFER".to_string());
        fields.insert("FROM_USER_ID".to_string(), "1001".to_string());
        fields.insert("TO_USER_ID".to_string(), "1002".to_string());
        fields.insert("AMOUNT".to_string(), "50000".to_string());
        fields.insert("TIMESTAMP".to_string(), "1633046400".to_string());
        fields.insert("STATUS".to_string(), "SUCCESS".to_string());
        fields.insert("DESCRIPTION".to_string(), "Test transaction".to_string());

        // Act: создаем текстовую запись из HashMap
        let text_record = YPBankTextFormat::new_from_map(fields).unwrap();

        // Assert: проверяем корректность создания
        assert_eq!(text_record.tx_id, 1234567890000000);
        assert_eq!(text_record.tx_type, TxType::Transfer);
        assert_eq!(text_record.amount, 50000);
        assert_eq!(text_record.description, "Test transaction".to_string());
    }

    #[test]
    fn test_new_from_map_for_csv_format() {
        // Arrange: создаем HashMap с данными
        let mut fields = HashMap::new();
        fields.insert("TX_ID".to_string(), "9876543210000000".to_string());
        fields.insert("TX_TYPE".to_string(), "DEPOSIT".to_string());
        fields.insert("FROM_USER_ID".to_string(), "0".to_string());
        fields.insert("TO_USER_ID".to_string(), "1003".to_string());
        fields.insert("AMOUNT".to_string(), "100000".to_string());
        fields.insert("TIMESTAMP".to_string(), "1633046401".to_string());
        fields.insert("STATUS".to_string(), "PENDING".to_string());
        fields.insert("DESCRIPTION".to_string(), "".to_string()); // Пустое описание

        // Act: создаем CSV запись из HashMap
        let csv_record = YPBankCsvFormat::new_from_map(&fields).unwrap();

        // Assert: проверяем корректность создания
        assert_eq!(csv_record.tx_id, 9876543210000000);
        assert_eq!(csv_record.tx_type, TxType::Deposit);
        assert_eq!(csv_record.amount, 100000);
        assert_eq!(csv_record.description, "".to_string()); // Пустая строка
    }
}
