//! Общие модели представления данных для чтения/записи, парсинга.

use crate::errors::ParseError;
use parser_macros::{TxDisplay, YPBankFields};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

/// Макрос преобразования структур `YPBankCsvFormat`, `YPBankTextFormat` в универсальную,
/// и предусмотрены необходимые схожие проверки.
macro_rules! impl_try_from_yp_format_to_transaction {
    ($source_type:ty) => {
        impl TryFrom<$source_type> for YPBankTransaction {
            type Error = ParseError;

            fn try_from(source: $source_type) -> Result<Self, ParseError> {
                let amount: i64 = source
                    .amount
                    .try_into()
                    .map_err(|_| ParseError::over_flow_size("u64", "i64", source.amount))?;

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

/// Макрос преобразования структуры `YPBankTransaction` в структуры `YPBankCsvFormat`
/// и `YPBankTextFormat`. Для бинарной структуры создан отдельный метод, потому что внутри
/// предусматривается индивидуальная проработка с полями.
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

                Ok($dest_type {
                    tx_id: value.tx_id,
                    tx_type: value.tx_type,
                    from_user_id: value.from_user_id,
                    to_user_id: value.to_user_id,
                    amount: value.amount as u64,
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
#[derive(Debug, YPBankFields, Clone)]
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
#[derive(Debug, YPBankFields, Clone)]
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
#[derive(Debug, YPBankFields, Clone)]
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
        writeln!(f, "DESCRIPTION: {}", self.description)
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
