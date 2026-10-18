//! Общие преобразования скалярных wire-значений ISS.

/// Преобразует строку ISS в компактное владение без дополнительного копирования,
/// если нормализация пробелов не меняет содержимое.
pub(super) fn into_trimmed_boxed_str(value: String) -> Box<str> {
    let trimmed = value.trim();
    if trimmed.len() == value.len() {
        value.into_boxed_str()
    } else {
        trimmed.into()
    }
}

pub(super) fn optional_non_negative<E>(
    value: Option<i64>,
    error: impl FnOnce(i64) -> E,
) -> Result<Option<u64>, E> {
    value
        .map(|raw| u64::try_from(raw).map_err(|_| error(raw)))
        .transpose()
}
