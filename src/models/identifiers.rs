//! Строгие идентификаторы и имена объектов MOEX.

use std::fmt;
use std::str::FromStr;

use super::errors::{
    ParseBoardIdError, ParseEngineNameError, ParseIndexError, ParseMarketNameError, ParseSecIdError,
};
use super::helpers::into_trimmed_boxed_str;

fn parse_identifier<E>(value: &str, empty: E, slash: Option<E>) -> Result<&str, E> {
    let value = value.trim();
    if value.is_empty() {
        return Err(empty);
    }
    if let Some(error) = slash
        && value.contains('/')
    {
        return Err(error);
    }
    Ok(value)
}

macro_rules! string_identifier {
    ($(#[$meta:meta])* $name:ident, $error:ident, $empty:ident, $slash:expr) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(Box<str>);

        impl $name {
            /// Вернуть строковое представление.
            pub fn as_str(&self) -> &str {
                self.0.as_ref()
            }

        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl From<&$name> for $name {
            fn from(value: &$name) -> Self {
                value.clone()
            }
        }

        impl TryFrom<String> for $name {
            type Error = $error;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                parse_identifier(&value, $error::$empty, $slash)?;
                Ok(Self(into_trimmed_boxed_str(value)))
            }
        }

        impl TryFrom<&str> for $name {
            type Error = $error;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                parse_identifier(value, $error::$empty, $slash)
                    .map(|value| Self(value.to_owned().into_boxed_str()))
            }
        }

        impl FromStr for $name {
            type Err = $error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::try_from(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

string_identifier!(
    /// Идентификатор индекса MOEX (`indexid`).
    IndexId,
    ParseIndexError,
    EmptyIndexId,
    None
);
string_identifier!(
    /// Имя торгового движка MOEX (`engine`).
    EngineName,
    ParseEngineNameError,
    Empty,
    Some(ParseEngineNameError::ContainsSlash)
);
string_identifier!(
    /// Идентификатор инструмента MOEX (`secid`).
    SecId,
    ParseSecIdError,
    Empty,
    Some(ParseSecIdError::ContainsSlash)
);
string_identifier!(
    /// Идентификатор режима торгов MOEX (`boardid`).
    BoardId,
    ParseBoardIdError,
    Empty,
    Some(ParseBoardIdError::ContainsSlash)
);
string_identifier!(
    /// Имя рынка MOEX (`market`).
    MarketName,
    ParseMarketNameError,
    Empty,
    Some(ParseMarketNameError::ContainsSlash)
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// Идентификатор торгового движка MOEX.
pub struct EngineId(pub(super) u32);

impl EngineId {
    /// Вернуть числовое значение идентификатора.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for EngineId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// Идентификатор рынка MOEX.
pub struct MarketId(pub(super) u32);

impl MarketId {
    /// Вернуть числовое значение идентификатора.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for MarketId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
