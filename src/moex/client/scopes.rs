use super::*;

macro_rules! owned_scope {
    ($name:ident, $client:ty, { $($(#[$doc:meta])* $field:ident: $ty:ty $(=> $getter:ident)?),* $(,)? }) => {
        /// Владеющий контекст ISS для fluent-запросов.
        #[derive(Clone)]
        pub struct $name<'a> {
            pub(in crate::moex::client) client: &'a $client,
            $(pub(in crate::moex::client) $field: $ty,)*
        }

        impl $name<'_> {
            $(owned_scope!(@getter [$(#[$doc])*] $field: $ty $(=> $getter)?);)*
        }
    };
    (@getter [$(#[$doc:meta])*] $field:ident: $ty:ty => $getter:ident) => {
        $(#[$doc])*
        pub fn $getter(&self) -> &$ty {
            &self.$field
        }
    };
    (@getter [$($doc:tt)*] $field:ident: $ty:ty) => {};
}

mod catalog;
mod security;

pub use catalog::*;
pub use security::*;

macro_rules! owned_index_scope {
    ($name:ident, $client:ty, $mode:ident, [$($asyncness:tt)?], $index_analytics_pages:ident) => {
        owned_scope!($name, $client, {
            /// Идентификатор индекса текущего контекста.
            indexid: IndexId => indexid,
        });
        impl<'a> $name<'a> {
            /// Получить состав индекса (`analytics`) для текущего контекста.
            pub $($asyncness)? fn analytics(
                &self,
                page_request: PageRequest,
            ) -> Result<Vec<IndexAnalytics>, MoexError> {
                await_result!(
                    $mode,
                    self.client
                        .index_analytics_query(&self.indexid, page_request)
                )
            }

            /// Создать ленивый пагинатор страниц `analytics` для текущего индекса.
            pub fn analytics_pages(&self, page_limit: NonZeroU32) -> $index_analytics_pages<'_> {
                self.client.index_analytics_pages(&self.indexid, page_limit)
            }
        }
    };
}

#[cfg(feature = "blocking")]
owned_index_scope!(
    OwnedIndexScope,
    BlockingMoexClient,
    blocking,
    [],
    IndexAnalyticsPages
);
#[cfg(feature = "async")]
owned_index_scope!(AsyncOwnedIndexScope, AsyncMoexClient, asynchronous, [async], AsyncIndexAnalyticsPages);
