//! Собственные исключения библиотеки.

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Error as IOError;

/// Библиотека предоставляет набор собственных ошибок и методов для их обслуживания.
#[derive(Debug)]
pub enum ParseError {
    /// Ошибка чтения-записи файлов.
    IOError {
        err_source: std::io::Error,
        description: String,
    },

    /// Ошибка парсинга файла (например, нарушена структура).
    ParseError {
        message: String,
        line: usize,
        column: usize,
    },

    /// Ошибка, вызванная некорректным форматом файла. Ожидался, например, `txt`, получен `csv`.
    InvalidFormat {
        expected: String,
        got: String,
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
        from: String,
        to: String,
        description: String,
    },

    // Неподдерживаемый формат парсинга.
    UnsupportedFormat {
        invalid_format: String,
    },
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
        }
    }
}

impl ParseError {
    /// Конструктор ошибки `ParseError::IOError`.
    ///
    /// ## Пример
    ///
    /// ```
    /// fn read_file() -> Result<String. ParseError>
    ///    let content = read_to_string("file.txt")
    ///         .map_err(|err| { ParseError::io_error(err, "Не могу прочитать файл")}?);
    ///
    ///     if content.is_empty() {
    ///         return Err(ParseError::parse_error("В файле нет данных", 1, 1));
    ///     }
    ///
    ///     Ok(content)
    /// ```
    pub fn io_error(err_source: IOError, description: impl Into<String>) -> Self {
        Self::IOError {
            err_source,
            description: description.into(),
        }
    }

    /// Конструктор ошибки `ParseError:ParseError`.
    pub fn parse_error(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self::ParseError {
            message: message.into(),
            line,
            column,
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
