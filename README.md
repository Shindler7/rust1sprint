# Парсер и два консольных приложения на Rust

Учебный проект. "Яндекс Практикум", "Rust для действующих разработчиков", 2025.

## Содержание проекта:

* Библиотека `Parser` — предоставляет методы для чтения (парсинга) и записи
  файлов в форматах `csv`, `txt` и `bin` с данными о банковских операциях
* Консольное приложение `cli_converter` — конвертирует файлы между форматами
* Консольное приложение `cli_comparer` — сравнивает содержимое файлов
  поддерживаемых форматов

## Установка

У вас должен быть установлен Rust Toolchain (Rust и Cargo, версия 1.85+).
[Инструкция](https://rust-lang.org/tools/install/).

Все приложения объединены в общее рабочее пространство
([workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html))
и компилируются одновременно.

```shell
git clone git@github.com:Shindler7/rust1sprint.git
cd rust1sprint
cargo build --release
```

Собранные бинарные файлы и компонентны библиотеки будут расположены в
папке `target\release\`.

В режиме разработки можно запускать отдельные бинарные крейты погружаясь
в их каталог. Например, `cli_comparer`. Исходим, что мы всё ещё находимся
в корне проекта.

```shell
cd .\cli_comparer\
cargo run -- --help
```

## Использование

Более подробную информацию об имеющихся методах можно посмотреть в документации
(как её собрать указано ниже). Здесь приведём некоторые базовые примеры.

### Parser (библиотека)

Задача библиотеки — парсинг и запись файлов, в известных форматах и на основе
предустановленных моделей данных. Вот, например, структура для CSV:

```rust
pub struct YPBankCsvFormat {
    pub tx_id: u64,
    pub tx_type: TxType,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub amount: u64,
    pub timestamp: u64,
    pub status: TxStatus,
    pub description: String,
}
```

Подобные есть для TXT и для бинарного формата. Присутствует также универсальная
структура `YPBankTransaction`, которая используется для взаимодействия между
данными разных форматов. Каждый легко преобразуется в эту структуру, а она
легко может быть трансформирована в структуру нужного формата.

```rust
pub struct YPBankTransaction {
    pub tx_id: u64,
    pub tx_type: TxType,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub amount: i64,
    pub timestamp: u64,
    pub status: TxStatus,
    pub description: Option<String>,
}
```

Библиотека взаимодействует с вводом-выводом через стандартные трейты Rust:
`Write` и `Read`. То есть, ей нужно передать читателя (reader) и писателя
(writer), и это могут быть многие варианты, поддерживающие указанные
стандартные трейты. Файлы, stdin, stdout и так далее.

Посмотрим простой пример. Создать проект записи в формате `TXT` и передать
её в stdout.

```rust
use parser::models::{TxStatus, TxType, YPBankTextFormat};
use parser::utils::get_timestamp;
use std::io;
use parser::write_text;


fn main() {
    let timestamp = get_timestamp();

    let yp_txt = vec![
        YPBankTextFormat {
            tx_id: 1000000000000982,
            tx_type: TxType::Transfer,
            from_user_id: 9223372036854775807,
            to_user_id: 29918172560165698,
            amount: 98300,
            timestamp,
            status: TxStatus::Pending,
            description: "Record number 982".to_string()
        }
    ];

    let mut stdout = io::stdout();

    write_text(&mut stdout, &yp_txt).unwrap();
}
```

Здесь используется вывод в стандартный поток (stdout), но можно использовать
любой Write, например файл.

Для доступа к верхнеуровневым методам используйте:

```rust
use parser::*;
```

Доступные, например, варианты: `write_text`, `read_text`, `write_csv`,
`read_csv` и так далее.

### cli-converter — консольное приложение

Обеспечивает конвертацию файлов из одного поддерживаемого формата в другой.
Например, `csv` в `bin`.

Для получения списка всех параметров запуска используйте:

* **В режиме разработки** (`debug`):

```shell
cargo run -- --help
```

* **В сборке** `release` (или если приложение установлено):

```shell
cli_converter.exe --help
```

### cli-comparer — консольное приложение

Выше мы упоминали об универсальной структуре `YPBankTransaction`. Благодаря ей,
сравнение данных в разных форматах существенно упрощается. Выгрузив наборы
данных и унифицировано нормализовав их, мы можем сравнивать с наиболее
гарантированным результатом корректности сверки.

Посмотрим пример.

```rust
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
```

С помощью методов `Parser` мы обеспечиваем выгрузку данных из файлов нужных
форматов,
а затем формируем векторы с набором `YPBankTransaction`.

В текущей версии сравнивается только совпадение экземпляров структур. Вот
результат,
если данные идентичны, но форматы были разные.

```shell
PS D:\Coding\yandex\1sprint_final\cli_comparer> cargo run -- 
  \ --first-file "D:\Coding\yandex\1sprint_final\.sources\records_example.txt" 
  \ --first-file-format txt 
  \ --second-file "D:\Coding\yandex\1sprint_final\.sources\records_example.csv" 
  \ --second-file-format csv

Thanks. Let's go...
The transaction records in 'records_example.txt' and 'records_example.csv' are IDENTICAL
```

## Документация

Все методы трёх крейтов документированы. Это можно использовать для сборки
документации,
с помощью стандартных методов Rust.

Например, так (из корня проекта или в каталоге отдельных крейтов):

```shell
cargo doc --open --message-format human --color auto --no-deps
```

Генерация произойдёт в папке `target/doc`. Откроется браузер с начальной
страницей документации.

Подробнее об имеющихся возможностях `doc`:

```shell
cargo doc --help
```

## Тесты

Основной функционал приложения покрыт тестами.

```shell
cargo test
```

Можно запускать с флагами для подробностей:

```shell
cargo test -- --nocapture
```

## Версионирование

### Версии компонентов workspace

| Компонент     | Версия | Описание                            |
|---------------|--------|-------------------------------------|
| parser        | 0.2.0  | Парсер банковских выписок           |
| cli_comparer  | 0.1.1  | CLI для сравнения транзакций        |
| cli_converter | 0.1.1  | CLI для конвертации между форматами |

### История изменения версий

#### parser => 0.2.0 (02.01.2026)

* Добавлена недостающая документация к методам и полям.
* Исправлена строгая привязка к версиям зависимостей.
* Сделано безопасное преобразование `usize` в `u32` для `desc_len` в модуле
  `bin.rs`: по аналогии с применяемым в моделях структур
* В структуре `YPBankTextFormat` метод `new_from_map` теперь принимает ссылку
  на Hashmap, а не владение.
* Обработка reader и writer в обёрнута в `BufReader` и `BufWriter`,
  соответственно.
* Добавлено ограничение по размеру входных данных: предусмотрены константы
  для максимального размера, и отдельная ошибка `ParseError::SizeLimitExceeded`
* Локальный рефакторинг. Исключен неиспользуемый макрос `parse_field`

#### cli_comparer, cli_converter => 0.1.1 (02.01.2026)

* Незначительные правки, не влияющие на функциональность. В частности
  уточнение значений в `cargo.toml`, включая корректное определение версий
  используемых зависимостей

## Благодарности

Команде "Яндекс Практикум" за интересный курс, а персонально автору ревью
за детальный разбор и ценнейшие предложения.
