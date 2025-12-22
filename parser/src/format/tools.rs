//! Вспомогательные общие утилиты для обработки форматов.

/// Поддерживающий трейт для работы со строками.
pub trait LineUtils {
    fn is_empty_line(&self) -> bool;
    fn is_hash_marker(&self) -> bool;
    fn split_into_key_value(&self) -> Option<(String, String)>;
}

impl<T: AsRef<str>> LineUtils for T {
    /// Проверяет, содержит ли строка данные или пустая.
    fn is_empty_line(&self) -> bool {
        self.as_ref().trim().is_empty()
    }

    fn is_hash_marker(&self) -> bool {
        self.as_ref().trim().starts_with("#")
    }

    fn split_into_key_value(&self) -> Option<(String, String)> {
        let (k, v) = self.as_ref().split_once(':')?;
        let key = k.trim().to_uppercase();
        let value = v.trim();
        if key.is_empty() || value.is_empty() {
            return None;
        }

        let val_clean = value
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or(value)
            .to_string();

        Some((key, val_clean))
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
