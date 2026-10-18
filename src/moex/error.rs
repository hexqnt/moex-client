//! Структурированные ошибки ISS и признаки повторяемости запросов.

#[cfg(any(feature = "async", feature = "blocking"))]
use reqwest::StatusCode;
use thiserror::Error;

use crate::models::{
    ParseBoardError, ParseCandleBorderError, ParseCandleError, ParseEngineError, ParseEventError,
    ParseHistoryDatesError, ParseHistoryRecordError, ParseIndexAnalyticsError, ParseIndexError,
    ParseMarketError, ParseOrderbookError, ParseSecStatError, ParseSecurityBoardError,
    ParseSecurityError, ParseSecuritySnapshotError, ParseSiteNewsError, ParseTradeError,
    ParseTurnoverError,
};

#[derive(Debug, Error)]
/// Ошибки выполнения запросов к ISS и конвертации wire-ответов в доменные типы.
pub enum MoexError {
    /// Некорректно задан базовый URL ISS.
    #[error("invalid base URL '{base_url}': {reason}")]
    InvalidBaseUrl {
        /// Строка базового URL, которую не удалось разобрать.
        base_url: &'static str,
        /// Подробность ошибки парсинга URL.
        reason: String,
    },
    /// Ошибка сборки `reqwest::blocking::Client`.
    #[cfg(any(feature = "async", feature = "blocking"))]
    #[error("failed to build HTTP client: {source}")]
    BuildHttpClient {
        /// Исходная ошибка HTTP-клиента.
        #[source]
        source: reqwest::Error,
    },
    /// Для async rate-limit не задана функция `sleep`.
    #[error(
        "async rate limit requires sleep function; set AsyncMoexClientBuilder::rate_limit_sleep(...)"
    )]
    MissingAsyncRateLimitSleep,
    /// Не удалось построить URL конкретного endpoint.
    #[error("failed to build URL for endpoint '{endpoint}': {reason}")]
    EndpointUrl {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Подробность ошибки построения URL.
        reason: String,
    },
    /// Для raw-запроса не задан endpoint-path.
    #[error("raw request path is not set")]
    MissingRawPath,
    /// Некорректно задан endpoint-path в raw-запросе.
    #[error("invalid raw request path '{path}': {reason}")]
    InvalidRawPath {
        /// Исходный endpoint-path.
        path: Box<str>,
        /// Деталь ошибки валидации path.
        reason: Box<str>,
    },
    /// В raw JSON-ответе отсутствует запрошенная таблица ISS.
    #[error("raw endpoint '{endpoint}' does not contain table '{table}'")]
    MissingRawTable {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Имя таблицы ISS (`history`, `trades`, `securities` и т.д.).
        table: Box<str>,
    },
    /// Строка raw-таблицы содержит число значений, отличное от числа колонок.
    #[error(
        "raw table '{table}' from endpoint '{endpoint}' has invalid row width at row {row}: expected {expected}, got {actual}"
    )]
    InvalidRawTableRowWidth {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Имя таблицы ISS.
        table: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Число колонок таблицы.
        expected: usize,
        /// Число значений в конкретной строке.
        actual: usize,
    },
    /// Не удалось декодировать строку raw-таблицы в пользовательский тип.
    #[error("failed to decode raw table '{table}' row {row} from endpoint '{endpoint}': {source}")]
    InvalidRawTableRow {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Имя таблицы ISS.
        table: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Исходная ошибка JSON-декодера.
        #[source]
        source: serde_json::Error,
    },
    /// Ошибка отправки HTTP-запроса до получения ответа.
    #[cfg(any(feature = "async", feature = "blocking"))]
    #[error("request to endpoint '{endpoint}' failed: {source}")]
    Request {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Исходная ошибка HTTP-клиента.
        #[source]
        source: reqwest::Error,
    },
    /// Endpoint вернул HTTP-статус вне диапазона `2xx`.
    #[cfg(any(feature = "async", feature = "blocking"))]
    #[error(
        "endpoint '{endpoint}' returned HTTP {status} (content-type={content_type:?}, prefix={body_prefix:?})"
    )]
    HttpStatus {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// HTTP-статус ответа.
        status: StatusCode,
        /// Значение HTTP `content-type`, если присутствует.
        content_type: Option<Box<str>>,
        /// Начало тела ответа для диагностики.
        body_prefix: Box<str>,
    },
    /// Ошибка чтения тела HTTP-ответа.
    #[cfg(any(feature = "async", feature = "blocking"))]
    #[error("failed to read endpoint '{endpoint}' response body: {source}")]
    ReadBody {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Исходная ошибка HTTP-клиента.
        #[source]
        source: reqwest::Error,
    },
    /// Ошибка десериализации JSON-пейлоада ISS.
    #[error("failed to decode endpoint '{endpoint}' JSON payload: {source}")]
    Decode {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Исходная ошибка JSON-декодера.
        #[source]
        source: serde_json::Error,
    },
    /// Endpoint вернул payload, не похожий на JSON.
    #[error(
        "endpoint '{endpoint}' returned non-JSON payload (content-type={content_type:?}, prefix={body_prefix:?})"
    )]
    NonJsonPayload {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Значение HTTP `content-type`, если присутствует.
        content_type: Option<Box<str>>,
        /// Начало тела ответа для диагностики.
        body_prefix: Box<str>,
    },
    /// В endpoint `securities/{secid}` пришло больше одной строки `securities`.
    #[error("endpoint '{endpoint}' returned unexpected security rows count: {row_count}")]
    UnexpectedSecurityRows {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Фактическое число строк в таблице `securities`.
        row_count: usize,
    },
    /// В history endpoint `.../dates` пришло больше одной строки `dates`.
    #[error("endpoint '{endpoint}' returned unexpected history dates rows count: {row_count}")]
    UnexpectedHistoryDatesRows {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Фактическое число строк в таблице `dates`.
        row_count: usize,
    },
    /// Ошибка преобразования строки таблицы `indices`.
    #[error("invalid index row {row} from endpoint '{endpoint}': {source}")]
    InvalidIndex {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseIndexError,
    },
    /// Ошибка преобразования строки таблицы `dates` из history endpoint.
    #[error("invalid history dates row {row} from endpoint '{endpoint}': {source}")]
    InvalidHistoryDates {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseHistoryDatesError,
    },
    /// Ошибка преобразования строки таблицы `history`.
    #[error("invalid history row {row} from endpoint '{endpoint}': {source}")]
    InvalidHistory {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseHistoryRecordError,
    },
    /// Ошибка преобразования строки таблицы `turnovers`.
    #[error("invalid turnover row {row} from endpoint '{endpoint}': {source}")]
    InvalidTurnover {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseTurnoverError,
    },
    /// Ошибка преобразования строки таблицы `sitenews`.
    #[error("invalid sitenews row {row} from endpoint '{endpoint}': {source}")]
    InvalidSiteNews {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseSiteNewsError,
    },
    /// Ошибка преобразования строки таблицы `events`.
    #[error("invalid events row {row} from endpoint '{endpoint}': {source}")]
    InvalidEvent {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseEventError,
    },
    /// Ошибка преобразования строки таблицы `secstats`.
    #[error("invalid secstats row {row} from endpoint '{endpoint}': {source}")]
    InvalidSecStat {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseSecStatError,
    },
    /// Ошибка преобразования строки таблицы `analytics`.
    #[error("invalid index analytics row {row} from endpoint '{endpoint}': {source}")]
    InvalidIndexAnalytics {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseIndexAnalyticsError,
    },
    /// Ошибка преобразования строки таблицы `engines`.
    #[error("invalid engine row {row} from endpoint '{endpoint}': {source}")]
    InvalidEngine {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseEngineError,
    },
    /// Ошибка преобразования строки таблицы `markets`.
    #[error("invalid market row {row} from endpoint '{endpoint}': {source}")]
    InvalidMarket {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseMarketError,
    },
    /// Ошибка преобразования строки таблицы `boards`.
    #[error("invalid board row {row} from endpoint '{endpoint}': {source}")]
    InvalidBoard {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseBoardError,
    },
    /// Ошибка преобразования строки таблицы `boards` в endpoint `securities/{secid}`.
    #[error("invalid security board row {row} from endpoint '{endpoint}': {source}")]
    InvalidSecurityBoard {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseSecurityBoardError,
    },
    /// Ошибка преобразования строки таблицы `securities`.
    #[error("invalid security row {row} from endpoint '{endpoint}': {source}")]
    InvalidSecurity {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseSecurityError,
    },
    /// Ошибка преобразования строки таблиц `securities`/`marketdata` в снимок инструмента.
    #[error("invalid security snapshot {table} row {row} from endpoint '{endpoint}': {source}")]
    InvalidSecuritySnapshot {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Имя таблицы ISS (`securities` или `marketdata`).
        table: &'static str,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseSecuritySnapshotError,
    },
    /// Ошибка преобразования строки таблицы `orderbook`.
    #[error("invalid orderbook row {row} from endpoint '{endpoint}': {source}")]
    InvalidOrderbook {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseOrderbookError,
    },
    /// Ошибка преобразования строки таблицы `borders`.
    #[error("invalid candle border row {row} from endpoint '{endpoint}': {source}")]
    InvalidCandleBorder {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseCandleBorderError,
    },
    /// Ошибка преобразования строки таблицы `candles`.
    #[error("invalid candle row {row} from endpoint '{endpoint}': {source}")]
    InvalidCandle {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseCandleError,
    },
    /// Ошибка преобразования строки таблицы `trades`.
    #[error("invalid trade row {row} from endpoint '{endpoint}': {source}")]
    InvalidTrade {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseTradeError,
    },
    /// Переполнение счётчика `start` при авто-пагинации ISS.
    #[error(
        "pagination overflow for endpoint '{endpoint}': start={start}, limit={limit} exceeds u32"
    )]
    PaginationOverflow {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Текущее значение `start`.
        start: u32,
        /// Размер страницы `limit`.
        limit: u32,
    },
    /// Обнаружен зацикленный ответ при авто-пагинации ISS.
    #[error(
        "pagination is stuck for endpoint '{endpoint}': repeated page at start={start}, limit={limit}"
    )]
    PaginationStuck {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Текущее значение `start`.
        start: u32,
        /// Размер страницы `limit`.
        limit: u32,
    },
}

impl MoexError {
    /// Признак, что операцию обычно имеет смысл повторить с backoff.
    pub fn is_retryable(&self) -> bool {
        match self {
            #[cfg(any(feature = "async", feature = "blocking"))]
            Self::BuildHttpClient { .. } => false,
            #[cfg(any(feature = "async", feature = "blocking"))]
            Self::Request { source, .. } => {
                source.is_timeout()
                    || source.is_connect()
                    || source.status().is_some_and(is_retryable_status)
            }
            #[cfg(any(feature = "async", feature = "blocking"))]
            Self::ReadBody { .. } => true,
            #[cfg(any(feature = "async", feature = "blocking"))]
            Self::HttpStatus { status, .. } => is_retryable_status(*status),
            _ => false,
        }
    }

    /// HTTP-статус, если ошибка была получена после ответа сервера.
    #[cfg(any(feature = "async", feature = "blocking"))]
    pub fn status_code(&self) -> Option<StatusCode> {
        match self {
            Self::Request { source, .. } => source.status(),
            Self::HttpStatus { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// Диагностический префикс тела ответа, если он сохранён в ошибке.
    pub fn response_body_prefix(&self) -> Option<&str> {
        match self {
            #[cfg(any(feature = "async", feature = "blocking"))]
            Self::HttpStatus { body_prefix, .. } | Self::NonJsonPayload { body_prefix, .. } => {
                Some(body_prefix)
            }
            #[cfg(not(any(feature = "async", feature = "blocking")))]
            Self::NonJsonPayload { body_prefix, .. } => Some(body_prefix),
            _ => None,
        }
    }
}

#[cfg(any(feature = "async", feature = "blocking"))]
fn is_retryable_status(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}
