//! Вспомогательные универсальные утилиты библиотеки.
use std::time::SystemTime;

/// Предоставляет количество секунд от начала эпохи UNIX, на основе системного времени.
///
/// В случае возникновения ошибки паникует.
pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
