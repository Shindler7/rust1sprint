//! Общие модели представления данных для чтения/записи, парсинга.

use crate::errors::ParseError;
use parser_macros::{TxDisplay, YPBankDisplay};
use std::fmt::Formatter;

/// Макрос преобразования структур `YPBankCsvFormat`, `YPBankTextFormat` в универсальную,
/// и предусмотрены необходимые схожие проверки.
#[macro_export]
macro_rules! impl_try_from_ypbank_source {
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
                    description: Some(source.description),
                })
            }
        }
    };
}

/// Тип транзакции.
#[repr(u8)]
#[derive(Debug, TxDisplay, Clone, PartialEq)]
pub enum TxType {
    Deposit = 0,
    Transfer = 1,
    Withdraw = 2,
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
#[derive(Debug, Clone, PartialEq, YPBankDisplay)]
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
/// ```
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
#[derive(Debug, YPBankDisplay)]
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

impl_try_from_ypbank_source!(YPBankCsvFormat);

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
/// ```
/// [ЗАГОЛОВОК][ТЕЛО][ЗАГОЛОВОК][ТЕЛО]...
/// ```
///
/// Наличие значения `MAGIC` в начале каждой записи позволяет читателю повторно
/// синхронизироваться в случае потери границы записи или повреждения данных.
#[derive(Debug, YPBankDisplay)]
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

impl TryFrom<YPBankBinFormat> for YPBankTransaction {
    type Error = ParseError;
    fn try_from(bin: YPBankBinFormat) -> Result<Self, ParseError> {
        let yp_uni = YPBankTransaction {
            tx_id: bin.tx_id,
            tx_type: bin.tx_type,
            from_user_id: bin.from_user_id,
            to_user_id: bin.to_user_id,
            amount: bin.amount,
            timestamp: bin.timestamp,
            status: bin.status,
            description: bin.description,
        };

        Ok(yp_uni)
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
#[derive(Debug, YPBankDisplay)]
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

impl_try_from_ypbank_source!(YPBankTextFormat);
