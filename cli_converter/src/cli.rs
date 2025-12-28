//! Взаимодействие с аргументами командной строки.

use clap::{Parser, ValueEnum};
use parser::YPFormatSupported;
use std::env;
use std::ffi::OsStr;
use std::fmt::Display;
use std::path::PathBuf;
use std::process::exit;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The path to the data file.
    #[clap(short, value_name = "INPUT_FILE")]
    input_file: PathBuf,

    /// The format of the source file (from the supported types).
    #[clap(long, value_enum)]
    input_format: FileFormat,

    /// The target format of the data file.
    #[clap(long, value_enum)]
    output_format: FileFormat,

    /// The path to save the file (including the file name).
    #[clap(short, value_name = "OUTPUT_FILE")]
    output_file: PathBuf,

    /// Do not overwrite the output file if it already exists. By default, the file
    /// will be overwritten.
    #[clap(short = 'n', long = "not-overwrite")]
    no_overwrite: bool,

    /// If the option is applied, a mismatch between the target file extension and the specified
    /// format is not allowed. Otherwise, only a console warning will be issued.
    #[clap(short = 's', long = "strict-target-ext")]
    strict_target_ext: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum FileFormat {
    /// CSV format (*.csv): Comma-Separated Values format — a plain text format for tabular data
    /// where each line is a data record, and fields are separated by commas.
    Csv,
    /// Binary format (*.bin): A compact, non-human-readable data format stored as raw bytes.
    Bin,
    /// Text format (*.txt): A plain text format for storing human-readable data.
    Txt,
}

impl Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileFormat::Csv => write!(f, "{}", YPFormatSupported::Csv),
            FileFormat::Txt => write!(f, "{}", YPFormatSupported::Text),
            FileFormat::Bin => write!(f, "{}", YPFormatSupported::Binary),
        }
    }
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

/// Структура данных задачи для конвертации.
pub struct ConvertTask {
    /// Путь к исходному файлу.
    pub input_file: PathBuf,
    /// Путь к целевому файлу.
    pub output_file: PathBuf,
    /// Формат данных в исходном файле (из предустановленных).
    pub input_format: FileFormat,
    /// Формат данных в целевом файле (из предустановленных).
    pub output_format: FileFormat,
}

/// Получить от пользователя задание на конвертацию.
///
/// Валидированные данные возвращаются в `ConvertTask`. Об ошибках сообщается пользователю, работа
/// приложения завершается.
pub fn cli_parse() -> ConvertTask {
    let args = Args::parse();

    let convert_task = ConvertTask {
        input_file: args.input_file,
        input_format: args.input_format,
        output_file: args.output_file,
        output_format: args.output_format,
    };

    if let Err(err) = validate_paths(&convert_task, args.no_overwrite, args.strict_target_ext) {
        exit_err(&err);
    }

    convert_task
}

/// Валидировать предоставленные пути к файлам, в том числе на соблюдение условий (например,
/// запрет/разрешение) перезаписи.
///
/// ## Args
///
/// * `convert_task` — ссылка на экземпляр `ConvertTask` с данными задачи по конвертации
/// * `no_overwrite` — логический тип: если `true` запрещена перезапись существующего целевого
///   файла
/// * `strict_ext` — логический тип: при `true` расширение целевого файла должно строго
///   соответствовать выбранному формату (например, для `txt` => `file.txt`).
fn validate_paths(
    convert_task: &ConvertTask,
    no_overwrite: bool,
    strict_ext: bool,
) -> Result<(), String> {
    if convert_task.input_file == convert_task.output_file {
        return Err("The input file and the output file cannot be the same path.".to_string());
    }

    if !convert_task.input_file.is_file() {
        return Err("The input file was not found or is not a valid file.".to_string());
    }

    if convert_task.output_file.is_dir() {
        return Err("The target path must be a file, not a directory.".to_string());
    }

    if convert_task.output_file.is_file() && no_overwrite {
        return Err(
            "The output file already exists, and overwriting is disabled by the `--not-overwrite` flag.".to_string(),
        );
    }

    // Проверка валидности расширения целевого файла.
    if let Some(err) = validate_output_extension(convert_task, strict_ext) {
        return Err(err);
    }

    Ok(())
}

/// Проверить расширение целевого файла и сравнить его с расширением, ожидаемым для выбранного
/// формата.
///
/// Возвращает строку с текстом ошибки, если выявлено несовпадение, и был использован ключ
/// `strict-target-ext` в командной строке.
fn validate_output_extension(convert_task: &ConvertTask, strict_ext: bool) -> Option<String> {
    let output_ext = convert_task
        .output_file
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_lowercase();
    let expected_ext = convert_task.output_format.to_string().to_lowercase();
    let match_ext = output_ext == expected_ext;

    if match_ext {
        None
    } else if strict_ext {
        Some(format!(
            "Output file extension does not match the selected format: .{} != .{}",
            output_ext, expected_ext
        ))
    } else {
        println!("WARNING: Output file extension does not match the selected format.");
        None
    }
}

/// Предоставляет с помощью стандартных методов директорию проекта.
#[allow(dead_code)]
pub fn current_dir() -> PathBuf {
    env::current_dir().expect("Не удаётся получить директорию проекта")
}

/// Опубликовать сообщение об ошибке и завершить работу приложения.
fn exit_err(message: &str) -> ! {
    eprintln!("Error: {}", message);
    exit(1);
}
