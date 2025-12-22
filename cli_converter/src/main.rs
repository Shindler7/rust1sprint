//! Консольное приложение, использующее функциональность парсеров.

use cli::current_dir;
use std::fs::File;

use parser::SupportedFormat;

mod cli;

fn main() {
    let app_dir = current_dir();
    let source_dir = app_dir
        .parent()
        .expect("Ошибка пути: родительский каталог не получен")
        .join(".sources");

    let records_txt = source_dir.join("records_example.txt");
    if !records_txt.exists() {
        panic!("Необходимый файл с записями отсутствует!")
    }
    println!("{}", records_txt.to_string_lossy());

    // Открываем файл и читаем.
    let mut file = File::open(records_txt).unwrap();

    let data = SupportedFormat::read_text(&mut file).unwrap();
    println!("OK");
    println!("Количество записей: {}", data.len());
    println!("Последняя запись: {}", data.last().unwrap());
}
