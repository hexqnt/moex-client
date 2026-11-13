use std::num::NonZeroU32;
use std::time::Duration;

use reqwest::Url;

use crate::models::BoardId;
use crate::moex::constants::BASE_URL;
use crate::moex::*;

// Одинаковые контракты конструкторов и owned scopes проверяем для обоих клиентов.
macro_rules! client_tests {
    ($module:ident, $client:ty, $http:ty) => {
        mod $module {
            use super::*;

            #[test]
            fn builder_constructs_client() {
                let client = <$client>::builder()
                    .metadata(true)
                    .build()
                    .expect("valid default builder");
                assert_eq!(client.rate_limit(), None);
            }

            #[test]
            fn constructors_without_semantic_bool_build_clients() {
                let _default = <$client>::new().expect("default constructor must work");
                let _with_metadata =
                    <$client>::new_with_metadata().expect("metadata constructor must work");

                let http = <$http>::builder().build().expect("valid HTTP client");
                let _with_client = <$client>::with_client(http).expect("constructor with client");

                let http = <$http>::builder().build().expect("valid HTTP client");
                let _with_client_metadata = <$client>::with_client_with_metadata(http)
                    .expect("constructor with client and metadata");

                let http = <$http>::builder().build().expect("valid HTTP client");
                let _with_base_url =
                    <$client>::with_base_url(http, Url::parse(BASE_URL).expect("valid base url"));

                let http = <$http>::builder().build().expect("valid HTTP client");
                let _with_base_url_metadata = <$client>::with_base_url_with_metadata(
                    http,
                    Url::parse(BASE_URL).expect("valid base url"),
                );
            }

            #[test]
            fn builder_user_agent_from_crate_constructs_client() {
                let _client = <$client>::builder()
                    .user_agent_from_crate()
                    .build()
                    .expect("user-agent from crate must be valid");
            }

            #[test]
            fn builder_proxy_constructs_client() {
                let proxy = reqwest::Proxy::all("http://127.0.0.1:8090").expect("valid proxy URL");
                let _client = <$client>::builder()
                    .proxy(proxy)
                    .build()
                    .expect("builder with proxy must construct client");
            }

            #[test]
            fn builder_no_proxy_constructs_client() {
                let _client = <$client>::builder()
                    .no_proxy()
                    .build()
                    .expect("builder with no_proxy must construct client");
            }

            #[test]
            fn builder_propagates_http_client_build_errors() {
                let err = match <$client>::builder().user_agent("invalid\nagent").build() {
                    Ok(_) => panic!("invalid user-agent must fail on build"),
                    Err(err) => err,
                };

                assert!(
                    matches!(err, MoexError::BuildHttpClient { .. }),
                    "unexpected error variant: {err:?}"
                );
            }

            #[test]
            fn supplied_http_client_skips_unused_builder_errors() {
                let http = <$http>::builder().build().expect("valid HTTP client");
                let _client = <$client>::builder()
                    .client(http)
                    .user_agent("invalid\nagent")
                    .build()
                    .expect("supplied client bypasses HTTP builder");
            }

            #[test]
            fn owned_scopes_accept_try_into_inputs_and_shortcuts() {
                let client = <$client>::builder().build().expect("valid default builder");
                let page_limit = NonZeroU32::new(1000).expect("non-zero page limit");
                let board = BoardId::try_from("TQBR").expect("valid board");
                let market_scope = client
                    .stock()
                    .expect("valid stock shortcut")
                    .shares()
                    .expect("valid shares shortcut");

                let board_scope = market_scope.clone().board(&board).expect("valid board");
                assert_eq!(board_scope.engine().as_str(), "stock");
                assert_eq!(board_scope.market().as_str(), "shares");
                assert_eq!(board_scope.board().as_str(), "TQBR");

                let market_security_scope = market_scope
                    .security("SBER")
                    .expect("valid market security");
                assert_eq!(market_security_scope.security().as_str(), "SBER");

                let security_scope = board_scope
                    .clone()
                    .security("SBER")
                    .expect("valid security");
                assert_eq!(security_scope.security().as_str(), "SBER");

                let _securities_pages = board_scope.securities_pages(page_limit);
                let _trades_pages = security_scope.trades_pages(page_limit);

                let index_scope = client.index("IMOEX").expect("valid indexid");
                assert_eq!(index_scope.indexid().as_str(), "IMOEX");
                let _analytics_pages = index_scope.analytics_pages(page_limit);

                let security_resource_scope = client.security("SBER").expect("valid secid");
                assert_eq!(security_resource_scope.secid().as_str(), "SBER");
            }

            #[test]
            fn owned_scopes_fail_fast_on_invalid_identifiers() {
                let client = <$client>::builder().build().expect("valid default builder");

                let invalid_engine = client.engine("");
                assert!(
                    matches!(
                        invalid_engine,
                        Err(crate::models::ParseEngineNameError::Empty)
                    ),
                    "unexpected engine parse result"
                );

                let invalid_board = client
                    .stock()
                    .expect("valid stock shortcut")
                    .shares()
                    .expect("valid shares shortcut")
                    .board(" ");
                assert!(
                    matches!(invalid_board, Err(crate::models::ParseBoardIdError::Empty)),
                    "unexpected board parse result"
                );

                let invalid_security = client.security(" ");
                assert!(
                    matches!(invalid_security, Err(crate::models::ParseSecIdError::Empty)),
                    "unexpected security parse result"
                );
            }
        }
    };
}

#[cfg(feature = "blocking")]
client_tests!(blocking, BlockingMoexClient, reqwest::blocking::Client);
#[cfg(feature = "async")]
client_tests!(asynchronous, AsyncMoexClient, reqwest::Client);

#[cfg(feature = "blocking")]
#[test]
fn builder_rate_limit_is_propagated_to_client() {
    let limit = RateLimit::every(Duration::from_millis(250));
    let client = BlockingMoexClient::builder()
        .rate_limit(limit)
        .build()
        .expect("builder with rate limit must construct client");

    assert_eq!(client.rate_limit(), Some(limit));
}

#[cfg(feature = "async")]
#[test]
fn async_builder_rate_limit_without_sleep_returns_error() {
    let err = match AsyncMoexClient::builder()
        .rate_limit(RateLimit::every(Duration::from_millis(250)))
        .build()
    {
        Ok(_) => panic!("async rate limit without sleep must fail on build"),
        Err(err) => err,
    };

    assert!(matches!(err, MoexError::MissingAsyncRateLimitSleep));
}

#[cfg(feature = "async")]
#[test]
fn async_builder_rate_limit_with_sleep_constructs_client() {
    let limit = RateLimit::every(Duration::from_millis(250));
    let client = AsyncMoexClient::builder()
        .rate_limit(limit)
        .rate_limit_sleep(|_delay| std::future::ready(()))
        .build()
        .expect("async builder with rate limit sleep must construct client");

    assert_eq!(client.rate_limit(), Some(limit));
}
