//! Вспомогательный модуль утилит, персональных для приложения.

use parser::errors::ParseError;
use std::fs::File;
use std::path::PathBuf;

/// Обёртка для метода [`File::open`], которая открывает файл и возвращает объект [`File`].
///
/// При ошибках возвращает [`ParseError`].
pub fn open_file(filepath: &PathBuf) -> Result<File, ParseError> {
    File::open(filepath).map_err(|err| {
        ParseError::io_error(err, format!("Failure to open file: {}", filepath.display()))
    })
}
