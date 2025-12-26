//! Вспомогательные общие утилиты для обработки форматов.

/// Поддерживающий трейт для работы со строками.
pub trait LineUtils {
    fn is_empty_line(&self) -> bool;
    fn is_hash_marker(&self) -> bool;
    fn split_into_key_value(&self) -> Option<(String, String)>;
    fn is_eq(&self, other: &str) -> bool;
    fn split_csv_line(&self) -> Option<Vec<String>>;
    fn clean_quote(&self) -> String;
}

impl<T: AsRef<str>> LineUtils for T {
    /// Проверяет, содержит ли строка данные или пустая.
    fn is_empty_line(&self) -> bool {
        self.as_ref().trim().is_empty()
    }

    fn is_hash_marker(&self) -> bool {
        self.as_ref().trim().starts_with("#")
    }

    /// Возвращает два значения `ключ` и `значение` для строки вида
    /// `key:parameter`.
    ///
    /// `Key` будет преобразован в `uppercase`.
    fn split_into_key_value(&self) -> Option<(String, String)> {
        let (k, v) = self.as_ref().split_once(':')?;
        let key = k.trim().to_uppercase();
        let value = v.trim();
        if key.is_empty() || value.is_empty() {
            return None;
        }

        let val_clean = value.clean_quote();

        Some((key, val_clean))
    }

    /// Проверить соответствие строк, исключая пробелы и другие избыточные символы.
    fn is_eq(&self, other: &str) -> bool {
        self.as_ref().trim().eq(other.trim())
    }

    /// Парсер строк csv-записей. Разбирает строку на блоки, разделённые запятыми. Особое внимание
    /// к последнему блоку, который должен быть в кавычках, а внутри также может содержать запятые,
    /// лишние кавычки.
    ///
    /// Корректность (длина, наличие всех блоков) собранной строки не проверяет.
    fn split_csv_line(&self) -> Option<Vec<String>> {
        let mut fields = Vec::new();
        let mut buffer = String::new();
        let mut chars = self.as_ref().chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    // Начало поля с кавычками — предполагаем, что description
                    if !buffer.trim().is_empty() {
                        // Так не может или не должно быть: буфер очищается при запятой, а мы
                        // обнаружили его на кавычке: значит строка уже неточная.
                        return None;
                    }

                    while let Some(c) = chars.next() {
                        match c {
                            '"' => {
                                if let Some('"') = chars.peek() {
                                    chars.next();
                                    buffer.push('"');
                                } else {
                                    break;
                                }
                            }
                            '\t' | '\n' => continue,
                            _ => buffer.push(c),
                        }
                    }

                    fields.push(buffer.trim().to_string());
                    // После description больше ничего не ожидается.
                    return Some(fields);
                }

                ',' => {
                    fields.push(buffer.trim().to_string());
                    buffer.clear();
                }

                _ => buffer.push(ch),
            }
        }

        if !buffer.trim().is_empty() {
            fields.push(buffer.trim().to_string());
        }

        if fields.len() < 2 { None } else { Some(fields) }
    }

    /// Очищает строковые данные от кавычек, если есть. Возвращает без них, если найдены, или
    /// оригинальную строку, если кавычек не было.
    fn clean_quote(&self) -> String {
        let mut line = self.as_ref();

        if line.starts_with('"') && line.ends_with('"') && line.len() >= 2 {
            line = &line[1..line.len() - 1];
        }

        line.replace("\"\"", "\"")
    }
}

#[macro_export]
macro_rules! parse_field {
        ($key:literal, $type:ty) => {


            fields
                .get($key)
                .ok_or_else(|| ParseError::parse_error(concat!("Отсутствует поле: ", $key), 0, 0))
                .and_then(|v| v.parse::<$type>().map_err(|_| {
                    ParseError::parse_error(
                        &format!("Невозможно распарсить поле `{}`: {:?}", $key, v),
                        0,
                        0,
                    )
                }))?
        };
    }
