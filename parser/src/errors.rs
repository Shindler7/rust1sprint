//! Собственные исключения библиотеки.

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Error as IOError;

/// Библиотека предоставляет набор собственных ошибок и методов для их обслуживания.
#[derive(Debug)]
pub enum ParseError {
    /// Ошибка чтения-записи файлов.
    IOError {
        /// Исходная ошибка ввода-вывода.
        err_source: std::io::Error,

        /// Дополнительное описание к ошибке.
        description: String,
    },

    /// Превышен максимальный размер входных данных.
    SizeLimitExceeded {
        /// Размер полученных данных.
        actual: usize,

        /// Максимальный допустимый размер для данных.
        limit: usize,
    },

    /// Потерянное, отсутствующее поле для структуры данных.
    IncorrectField {
        /// Имя отсутствующего ключа (поля).
        key: String,
    },

    /// Ошибка парсинга файла (например, нарушена структура).
    ParseError {
        /// Сообщение, описывающее причину ошибки.
        message: String,

        /// Линия во входном потоке, где возникла ошибка.
        ///
        /// Рекомендуется передавать `0`, если нет данных.
        line: usize,

        /// Позиция в линии, где возникла ошибка.
        ///
        /// Рекомендуется передавать `0`, если нет данных.
        column: usize,
    },

    /// Ошибка преобразования бинарных данных в строку UTF-8 при парсинге.
    ParseBinaryError {
        /// Сообщение с описанием ошибки.
        message: String,
    },

    /// Предоставленный комплект для парсинга пустой.
    EmptyData,

    /// Ошибка, вызванная некорректным форматом файла. Ожидался, например,
    /// `txt`, получен `csv`.
    InvalidFormat {
        /// Информация об ожидаемом формате (типе) файла/данных.
        expected: String,
        /// Информация о полученном формате (типе) файла/данных.
        got: String,
        /// Исходная ошибка, ставшая причиной ошибки формата, если имеется.
        err_source: Option<Box<dyn Error + Send + Sync>>,
    },

    /// Ошибка переполнения при преобразовании.
    ///
    /// Например:
    /// - `u64` → `i64`: Не влезет если u64 ≥ 2⁶³;
    /// - `i64` → `u64`: Не влезет если i64 < 0.
    ///
    /// Ошибка применяется при управляемом преобразовании `TryFrom`.
    OverflowSize {
        /// Исходный тип для преобразования.
        from: String,
        /// Целевой тип преобразования.
        to: String,
        /// Описание к ошибке.
        description: String,
    },

    /// Ошибка для попыток использования неподдерживаемых форматов парсинга.
    UnsupportedFormat {
        /// Информация о запрошенном неподдерживаемом формате.
        invalid_format: String,
    },
}

impl From<std::io::Error> for ParseError {
    fn from(value: IOError) -> Self {
        let msg = value.to_string();
        ParseError::IOError {
            err_source: value,
            description: msg,
        }
    }
}

impl From<std::string::FromUtf8Error> for ParseError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ParseError::parse_bin_error(err.to_string())
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseError::IOError { err_source, .. } => Some(err_source),
            ParseError::InvalidFormat { err_source, .. } => {
                err_source.as_ref().map(|e| e.as_ref() as &dyn Error)
            }
            _ => None,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IOError { description, .. } => {
                write!(f, "Ошибка чтения/записи: {}", description)
            }
            ParseError::IncorrectField { key } => {
                write!(f, "Некорректные данные для поля: {key}")
            }
            ParseError::SizeLimitExceeded { actual, limit } => {
                write!(
                    f,
                    "Объём данных для парсинга ({actual} б) превышает лимит {limit} б"
                )
            }
            ParseError::ParseError {
                message,
                line,
                column,
            } => {
                write!(
                    f,
                    "Ошибка парсинга файла (строка {}, символ {}): {}",
                    line, column, message
                )
            }
            ParseError::ParseBinaryError { message } => {
                if message.is_empty() {
                    write!(f, "Ошибка парсинга бинарного файла")
                } else {
                    write!(f, "Ошибка парсинга бинарного файла: {}", message)
                }
            }

            ParseError::InvalidFormat { expected, got, .. } => {
                write!(
                    f,
                    "Некорректный формат: ожидался {}, обнаружен {}",
                    expected, got
                )
            }
            ParseError::OverflowSize {
                from,
                to,
                description,
            } => {
                write!(
                    f,
                    "Переполнение типа — {from} не может быть преобразован в {to}: {description}"
                )
            }
            ParseError::UnsupportedFormat { invalid_format } => {
                write!(
                    f,
                    "Запрошенный формат {} не поддерживается. См. документацию",
                    invalid_format
                )
            }
            ParseError::EmptyData => {
                write!(f, "Отсутствуют данные для парсинга")
            }
        }
    }
}

impl ParseError {
    /// Конструктор ошибки `ParseError::IOError`.
    ///
    /// ## Пример
    ///
    /// ```no_run
    /// use std::fs::read_to_string;
    /// use parser::errors::ParseError;
    ///
    /// let content = read_to_string("file.txt")
    ///         .map_err(|err| { ParseError::io_error(err, "Не могу прочитать файл")});
    /// ```
    pub fn io_error(err_source: IOError, description: impl Into<String>) -> Self {
        Self::IOError {
            err_source,
            description: description.into(),
        }
    }

    /// Конструктор для ошибки превышения объёма входных данных:
    /// [`ParseError::SizeLimitExceeded`].
    ///
    /// ## Аргументы
    ///
    /// - `actual` — оценённый размер входных данных (в байтах)
    /// - `limit` — максимально допустимый размер (в байтах)
    pub fn lim_exceed(actual: usize, limit: usize) -> Self {
        Self::SizeLimitExceeded { actual, limit }
    }

    /// Конструктор ошибки `ParseError:ParseError`.
    pub fn parse_err(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self::ParseError {
            message: message.into(),
            line,
            column,
        }
    }

    /// Конструктор ошибки `ParseBinaryError`.
    ///
    /// Аргумент `message` может быть пустым, в таком случае будет подменён фразой
    /// по-умолчанию.
    pub fn parse_bin_error(message: impl Into<String>) -> Self {
        Self::ParseBinaryError {
            message: message.into(),
        }
    }

    /// Конструктор ошибки `ParseError:OverFlowSize`.
    pub fn over_flow_size(
        from_type: impl Into<String>,
        to_type: impl Into<String>,
        value: impl Display,
    ) -> Self {
        let to_type = to_type.into();
        let description = format!(
            "Значение {} выходит за допустимый диапазон типа {}",
            value, to_type
        );

        Self::OverflowSize {
            from: from_type.into(),
            to: to_type,
            description,
        }
    }

    /// Конструктор ошибки `ParseError:InvalidFormat`.
    pub fn invalid_format(
        expected: impl Into<String>,
        got: impl Into<String>,
        err_source: Option<Box<dyn Error + Send + Sync>>,
    ) -> Self {
        Self::InvalidFormat {
            expected: expected.into(),
            got: got.into(),
            err_source,
        }
    }
}
