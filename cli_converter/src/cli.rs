//! Взаимодействие с аргументами командной строки.

use clap::{Parser, ValueEnum};
use std::env;
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

    /// Do not overwrite the output file if it already exists. By default, the file will be overwritten.
    #[clap(short = 'n', long = "not-overwrite")]
    no_overwrite: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum FileFormat {
    /// CSV format (*.csv): Comma-Separated Values format — a plain text format for tabular data where each line is a data record, and fields are separated by commas.
    Csv,
    /// Binary format (*.bin): A compact, non-human-readable data format stored as raw bytes.
    Bin,
    /// Text format (*.txt): A plain text format for storing human-readable data.
    Txt,
}

/// Структура данных задачи для конвертации.
pub struct ConvertTask {
    pub input_file: PathBuf,
    pub output_file: PathBuf,
    pub input_format: FileFormat,
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

    if let Err(err) = validate_paths(&convert_task, args.no_overwrite) {
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
///   файла.
fn validate_paths(convert_task: &ConvertTask, no_overwrite: bool) -> Result<(), String> {
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

    Ok(())
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
