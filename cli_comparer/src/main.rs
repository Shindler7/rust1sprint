//! # CLI Comparer
//!
//! Консольное приложение для сравнения данных, сохранённых в доступных форматах с помощью
//! библиотеки [`parser`].
//!
//! Принимает ссылки на два файла для сравнения и данные об их форматах. Обрабатывает файлы при
//! помощи механизмов парсера, а сравнение возможно осуществлять благодаря унифицированному
//! типу [`YPBankTransaction`].
//!
//! ## Поддерживаемые форматы
//!
//! - `csv`: табличный текстовый формат с разделением полей запятыми;
//! - `bin`: компактный бинарный формат (нечитаемый человеком);
//! - `txt`: простой текстовый формат для хранения человекочитаемых записей.//!
//!
//! ## Учебный проект
//!
//! "Яндекс Практикум", курс "Rust для действующих разработчиков", 2025.
//!
//! ## Справка
//!
//! Для получения списка параметров запуска используйте:
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
//!    cli_comparer.exe --help
//!    ```

#![warn(missing_docs)]

use crate::cli::{ComparerTask, cli_parse};
use crate::tools::open_file;
use parser::errors::ParseError;
use parser::models::YPBankTransaction;
use std::process::exit;

mod cli;
mod tools;

fn main() {
    let task = cli_parse();
    println!("Thanks. Let's go...");

    let result = execute_compare_task(&task).unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        exit(1);
    });

    let filenames = task
        .get_filenames()
        .unwrap_or_else(|| ("unknow".to_string(), "unknow".to_string()));

    if result == 0 {
        println!(
            "The transaction records in '{}' and '{}' are IDENTICAL",
            filenames.0, filenames.1
        );
    } else {
        println!(
            "The transaction records in '{}' and '{}' are NOT IDENTICAL",
            filenames.0, filenames.1
        );
        println!("Number of mismatched elements: {}", result);
    }
}

/// Сравнение данных в предоставленных файлах.
///
/// ## Args
///
/// * `compare_task` — ссылка на экземпляр [`ComparerTask`], содержащий информацию об анализируемых
///   файлах.
///
/// ## Returns
///
/// Возвращает при удачной обработке число `u64` — количество несовпадающих структур (от 0 и более).
/// При ошибках [`ParseError`].
fn execute_compare_task(comparer_task: &ComparerTask) -> Result<u64, ParseError> {
    let mut file1 = open_file(&comparer_task.first_file)?;
    let mut file2 = open_file(&comparer_task.second_file)?;

    let left_side = comparer_task
        .first_format
        .to_parsers_fmt()
        .to_transaction(&mut file1)?;

    let right_side = comparer_task
        .second_format
        .to_parsers_fmt()
        .to_transaction(&mut file2)?;

    Ok(compare_sides(&left_side, &right_side))
}

fn compare_sides(left: &[YPBankTransaction], right: &[YPBankTransaction]) -> u64 {
    let length = left.len().min(right.len());
    let counter = left
        .iter()
        .zip(right.iter())
        .take(length)
        .filter(|(l, r)| l != r)
        .count() as u64;

    let len_different = left.len().abs_diff(right.len()) as u64;

    counter + len_different
}
