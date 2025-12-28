//! Взаимодействие с аргументами командной строки.

use clap::{Parser, ValueEnum};
use parser::YPFormatSupported;
use std::path::PathBuf;
use std::process::exit;

#[derive(Parser, Debug)]
#[clap(about = "Compares structured data in CSV, BIN, and TXT formats using the Parser library.")]
#[clap(author, version, long_about = None)]
struct Args {
    /// The path to the first file.
    #[clap(long, value_name = "file1")]
    first_file: PathBuf,

    /// The format of the first file (from the supported types).
    #[clap(long, value_enum, value_name = "format1")]
    first_file_format: FileFormat,

    /// The path to the second file.
    #[clap(long, value_name = "file2")]
    second_file: PathBuf,

    /// The format of the second file (from the supported types).
    #[clap(long, value_enum, value_name = "format2")]
    second_file_format: FileFormat,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
#[repr(u8)]
pub enum FileFormat {
    /// CSV format (*.csv): Comma-Separated Values format — a plain text format for tabular data
    /// where each line is a data record, and fields are separated by commas.
    Csv = 0,
    /// Binary format (*.bin): A compact, non-human-readable data format stored as raw bytes.
    Bin,
    /// Text format (*.txt): A plain text format for storing human-readable data.
    Txt,
}

impl FileFormat {
    pub fn to_parsers_fmt(self) -> YPFormatSupported {
        match self {
            FileFormat::Csv => YPFormatSupported::Csv,
            FileFormat::Bin => YPFormatSupported::Binary,
            FileFormat::Txt => YPFormatSupported::Text,
        }
    }
}

/// Структура для задачи сравнения данных.
pub struct ComparerTask {
    /// Путь к первому файлу.
    pub first_file: PathBuf,
    /// Путь ко второму файлу.
    pub second_file: PathBuf,
    /// Формат данных в первом файле (из предустановленных).
    pub first_format: FileFormat,
    /// Формат данных во втором файле (из предустановленных).
    pub second_format: FileFormat,
}

impl ComparerTask {
    /// Самопроверка данных структуры.
    ///
    /// Возвращает `None`, если проверка успешная, и текстовую строку с информацией об ошибке,
    /// если обнаружены проблемы.
    fn validate(&self) -> Option<String> {
        if !self.first_file.is_file() {
            Some(format!(
                "The file {} does not exist.",
                self.first_file.display()
            ))
        } else if !self.second_file.is_file() {
            Some(format!(
                "The file {} does not exist.",
                self.second_file.display()
            ))
        } else {
            None
        }
    }

    /// Возвращает имена файлов `first_file` и `second_file`, если поля заполнены корректно.
    ///
    /// Существуют ли файлы, и файлы ли это, не проверяется. Формально обёртка для метода
    /// `file_name()` в [`PathBuf`].
    pub fn get_filenames(&self) -> Option<(String, String)> {
        Some((
            self.first_file.file_name()?.to_string_lossy().into_owned(),
            self.second_file.file_name()?.to_string_lossy().into_owned(),
        ))
    }
}

/// Получить от пользователя вводные для сравнения данных: пути к файлам, их форматы.
///
/// Функция гарантированно возвращает успешно сформированную задачу, так как данные проверяются,
/// а при ошибках уведомляется пользователь и работа приложения прерывается.
pub fn cli_parse() -> ComparerTask {
    let args = Args::parse();

    let compare_task = ComparerTask {
        first_file: args.first_file,
        second_file: args.second_file,
        first_format: args.first_file_format,
        second_format: args.second_file_format,
    };

    if let Some(message) = compare_task.validate() {
        exit_err(&message)
    }

    compare_task
}

/// Опубликовать сообщение об ошибке и завершить работу приложения.
fn exit_err(message: &str) -> ! {
    eprintln!("Error: {}", message);
    exit(1);
}
