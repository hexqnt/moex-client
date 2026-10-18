use super::*;

pub(super) struct PaginationTracker<K> {
    endpoint: Box<str>,
    page_limit: NonZeroU32,
    start: u32,
    first_key_on_previous_page: Option<K>,
    finished: bool,
}

impl<K> PaginationTracker<K> {
    pub(super) fn new(endpoint: impl Into<String>, page_limit: NonZeroU32) -> Self {
        Self {
            endpoint: endpoint.into().into_boxed_str(),
            page_limit,
            start: 0,
            first_key_on_previous_page: None,
            finished: false,
        }
    }

    pub(super) fn next_page_request(&self) -> Option<Pagination> {
        if self.finished {
            return None;
        }
        Some(Pagination {
            start: Some(self.start),
            limit: Some(self.page_limit),
        })
    }
}

impl<K: Eq> PaginationTracker<K> {
    pub(super) fn accept_page<T>(
        &mut self,
        page: Vec<T>,
        first_key_of: impl FnOnce(&T) -> K,
    ) -> Result<Option<Vec<T>>, MoexError> {
        let Some(first) = page.first() else {
            self.finished = true;
            return Ok(None);
        };

        let page_limit = self.page_limit.get();
        let first_key = first_key_of(first);
        if self.first_key_on_previous_page.as_ref() == Some(&first_key) {
            return Err(MoexError::PaginationStuck {
                endpoint: self.endpoint.clone(),
                start: self.start,
                limit: page_limit,
            });
        }

        self.first_key_on_previous_page = Some(first_key);

        if (page.len() as u128) < u128::from(page_limit) {
            self.finished = true;
        } else {
            self.start = self.start.checked_add(page_limit).ok_or_else(|| {
                MoexError::PaginationOverflow {
                    endpoint: self.endpoint.clone(),
                    start: self.start,
                    limit: page_limit,
                }
            })?;
        }

        Ok(Some(page))
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
