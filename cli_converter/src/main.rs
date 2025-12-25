//! Консольное приложение, использующее функциональность парсеров.

use cli::current_dir;
use std::fs::File;
use std::io::Stdout;

use parser::*;

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
    let data = read_text(&mut file).unwrap();

    println!("OK");
    println!("Количество записей: {}", data.len());
    println!("Последняя запись: {}", data.last().unwrap());

    // Теперь попытка опубликовать последнюю запись.
    let record_txt_new = source_dir.join("records_new.txt");
    println!("{}", record_txt_new.to_string_lossy());

    let mut file = File::create(record_txt_new).unwrap();

    let data_last = data.last().unwrap().clone();

    write_text(&mut file, &[data_last]).unwrap();

    // CSV.
    let records_csv = source_dir.join("records_example.csv");
    if !records_csv.exists() {
        panic!("Необходимый файл CSV с записями отсутствует!")
    }
    let mut file_csv = File::open(records_csv).unwrap();
    let data = read_csv(&mut file_csv).unwrap();
    println!("OK CSV");
    println!("Количество записей CSV: {}", data.len());
    println!("Последняя запись: {:?}", data.last().unwrap());
}
