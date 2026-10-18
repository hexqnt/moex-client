//! Новости и события MOEX.

use chrono::NaiveDateTime;

use super::super::errors::{ParseEventError, ParseSiteNewsError};
use super::super::helpers::{non_empty_text, positive_integer};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Новость MOEX ISS (`sitenews`).
pub struct SiteNews {
    id: u64,
    tag: Box<str>,
    title: Box<str>,
    published_at: NaiveDateTime,
    modified_at: NaiveDateTime,
}

impl SiteNews {
    /// Построить запись новости из wire-значений ISS с валидацией.
    pub fn try_new(
        id: i64,
        tag: String,
        title: String,
        published_at: NaiveDateTime,
        modified_at: NaiveDateTime,
    ) -> Result<Self, ParseSiteNewsError> {
        let id = positive_integer(
            id,
            ParseSiteNewsError::NonPositiveId,
            ParseSiteNewsError::IdOutOfRange,
        )?;

        let tag = non_empty_text(tag, ParseSiteNewsError::EmptyTag)?;
        let title = non_empty_text(title, ParseSiteNewsError::EmptyTitle)?;

        Ok(Self {
            id,
            tag,
            title,
            published_at,
            modified_at,
        })
    }

    /// Идентификатор новости (`id`).
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Тег новости (`tag`).
    pub fn tag(&self) -> &str {
        self.tag.as_ref()
    }

    /// Заголовок новости (`title`).
    pub fn title(&self) -> &str {
        self.title.as_ref()
    }

    /// Время публикации (`published_at`).
    pub fn published_at(&self) -> NaiveDateTime {
        self.published_at
    }

    /// Время изменения (`modified_at`).
    pub fn modified_at(&self) -> NaiveDateTime {
        self.modified_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Событие MOEX ISS (`events`).
pub struct Event {
    id: u64,
    tag: Box<str>,
    title: Box<str>,
    from: Option<NaiveDateTime>,
    modified_at: NaiveDateTime,
}

impl Event {
    /// Построить запись события из wire-значений ISS с валидацией.
    pub fn try_new(
        id: i64,
        tag: String,
        title: String,
        from: Option<NaiveDateTime>,
        modified_at: NaiveDateTime,
    ) -> Result<Self, ParseEventError> {
        let id = positive_integer(
            id,
            ParseEventError::NonPositiveId,
            ParseEventError::IdOutOfRange,
        )?;

        let tag = non_empty_text(tag, ParseEventError::EmptyTag)?;
        let title = non_empty_text(title, ParseEventError::EmptyTitle)?;

        Ok(Self {
            id,
            tag,
            title,
            from,
            modified_at,
        })
    }

    /// Идентификатор события (`id`).
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Тег события (`tag`).
    pub fn tag(&self) -> &str {
        self.tag.as_ref()
    }

    /// Заголовок события (`title`).
    pub fn title(&self) -> &str {
        self.title.as_ref()
    }

    /// Время начала/актуальности события (`from`), если задано.
    pub fn from(&self) -> Option<NaiveDateTime> {
        self.from
    }

    /// Время изменения (`modified_at`).
    pub fn modified_at(&self) -> NaiveDateTime {
        self.modified_at
    }
}

tuple_row_conversion!(SiteNewsRow => SiteNews, ParseSiteNewsError; 0, 1, 2, 3, 4);

tuple_row_conversion!(EventRow => Event, ParseEventError; 0, 1, 2, 3, 4);
