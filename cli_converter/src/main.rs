//! # CLI Converter
//!
//! Консольное приложение для конвертации данных между форматами `CSV`, `BIN` и `TXT`,
//! использующее возможности библиотеки [`parser`].
//!
//! Программа принимает входной файл, его формат, целевой формат и путь для сохранения.
//! Поддерживаются параметры: перезапись выходного файла, проверка расширения и контроль
//! соответствия форматов.
//!
//! ## Поддерживаемые форматы
//!
//! - `csv`: табличный текстовый формат с разделением полей запятыми;
//! - `bin`: компактный бинарный формат (человеко-нечитаемый);
//! - `txt`: простой текстовый формат для хранения человекочитаемых записей.
//!
//! ## Учебный проект
//!
//! "Яндекс Практикум", курс *Rust для действующих разработчиков*, 2025.
//!
//! ## Справка
//!
//! Для получения списка всех параметров запуска используйте:
//!
//! 1. В режиме разработки (`debug`):
//!
//!    ```shell
//!    cargo run -- --help
//!    ```
//!
//! 2. В сборке `release` (или после установки):
//!
//!    ```shell
//!    cli_converter.exe --help
//!    ```
#![warn(missing_docs)]

use cli::{ConvertTask, cli_parse};
use parser::errors::ParseError;
use parser::models::YPBankTransaction;
use std::fs::File;
use std::process::exit;

mod cli;

fn main() {
    let convert_task = cli_parse();
    println!("Issue has been created!");

    convert_task.convert().unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        exit(1);
    });

    println!("OK! Issue has been converted!");
}

impl ConvertTask {
    /// Конвертировать данные из одного формата в другой.
    ///
    /// Структура наполняется и проверяется при формировании.
    fn convert(&self) -> Result<(), ParseError> {
        let read_data = self.read_with()?;
        self.write_with(read_data)?;
        Ok(())
    }

    /// Считать данные из исходного файла.
    fn read_with(&self) -> Result<Vec<YPBankTransaction>, ParseError> {
        let mut file = File::open(&self.input_file).map_err(|err| {
            ParseError::io_error(
                err,
                format!("Failure to open file: {}", &self.input_file.display()),
            )
        })?;

        self.input_format.to_parsers_fmt().to_transaction(&mut file)
    }

    /// Записать данные в целевой файл.
    fn write_with(&self, data: Vec<YPBankTransaction>) -> Result<(), ParseError> {
        let mut file = File::create(&self.output_file).map_err(|err| {
            ParseError::io_error(
                err,
                format!("Failure to create file: {}", &self.output_file.display()),
            )
        })?;

        self.output_format
            .to_parsers_fmt()
            .convert_transactions(&mut file, &data)
    }
}
