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

pub(super) fn non_empty_text<E>(value: String, error: E) -> Result<Box<str>, E> {
    let value = into_trimmed_boxed_str(value);
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}

pub(super) fn positive_integer<T: TryFrom<i64>, E>(
    value: i64,
    non_positive: impl FnOnce(i64) -> E,
    out_of_range: impl FnOnce(i64) -> E,
) -> Result<T, E> {
    if value <= 0 {
        return Err(non_positive(value));
    }
    T::try_from(value).map_err(|_| out_of_range(value))
}

pub(super) fn non_negative_u32<E>(
    value: i64,
    negative: impl FnOnce(i64) -> E,
    out_of_range: impl FnOnce(i64) -> E,
) -> Result<u32, E> {
    if value < 0 {
        return Err(negative(value));
    }
    u32::try_from(value).map_err(|_| out_of_range(value))
}

pub(super) fn wire_bool<E>(value: i64, error: impl FnOnce(i64) -> E) -> Result<bool, E> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(error(other)),
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
