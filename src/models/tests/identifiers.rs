use super::*;

#[test]
fn identifiers_support_parse_and_try_from_str() {
    let engine: EngineName = "stock".parse().expect("valid engine");
    let market = MarketName::try_from("shares").expect("valid market");
    let board = BoardId::try_from("TQBR").expect("valid board");
    let secid = SecId::try_from("SBER").expect("valid secid");
    let index: IndexId = "IMOEX".parse().expect("valid index");

    assert_eq!(engine.as_str(), "stock");
    assert_eq!(market.as_str(), "shares");
    assert_eq!(board.as_str(), "TQBR");
    assert_eq!(secid.as_str(), "SBER");
    assert_eq!(index.as_str(), "IMOEX");
}

#[test]
fn path_identifiers_share_owned_and_borrowed_invariants() {
    fn check<T, E>(empty: E, slash: E)
    where
        T: for<'a> TryFrom<&'a str, Error = E> + TryFrom<String, Error = E> + AsRef<str>,
        E: std::fmt::Debug + PartialEq,
    {
        for (raw, expected) in [
            ("SBER", "SBER"),
            ("\u{2003} SBER \u{a0}", "SBER"),
            ("A B", "A B"),
        ] {
            let borrowed =
                T::try_from(raw).unwrap_or_else(|err| panic!("invalid identifier: {err:?}"));
            let owned = T::try_from(raw.to_owned())
                .unwrap_or_else(|err| panic!("invalid identifier: {err:?}"));
            assert_eq!(borrowed.as_ref(), expected);
            assert_eq!(owned.as_ref(), expected);
        }
        for (raw, expected) in [
            ("", &empty),
            ("\u{2003} ", &empty),
            (" / ", &slash),
            ("A/B", &slash),
        ] {
            assert_eq!(T::try_from(raw).err().as_ref(), Some(expected));
            assert_eq!(T::try_from(raw.to_owned()).err().as_ref(), Some(expected));
        }
    }

    check::<EngineName, _>(
        ParseEngineNameError::Empty,
        ParseEngineNameError::ContainsSlash,
    );
    check::<MarketName, _>(
        ParseMarketNameError::Empty,
        ParseMarketNameError::ContainsSlash,
    );
    check::<BoardId, _>(ParseBoardIdError::Empty, ParseBoardIdError::ContainsSlash);
    check::<SecId, _>(ParseSecIdError::Empty, ParseSecIdError::ContainsSlash);
}

#[test]
fn index_identifier_retains_its_distinct_slash_and_debug_contract() {
    let borrowed = IndexId::try_from("\u{2003} A/B ").expect("index id permits slashes");
    let owned = IndexId::try_from("\u{2003} A/B ".to_owned()).expect("index id permits slashes");
    assert_eq!(borrowed, owned);
    assert_eq!(borrowed.as_str(), "A/B");
    assert_eq!(borrowed.to_string(), "A/B");
    assert_eq!(format!("{borrowed:?}"), "IndexId(\"A/B\")");
    assert_eq!(
        IndexId::try_from("\u{2003} "),
        Err(ParseIndexError::EmptyIndexId)
    );
    assert_eq!(
        IndexId::try_from(" ".to_owned()),
        Err(ParseIndexError::EmptyIndexId)
    );
}

#[test]
fn owned_identifier_reuses_untrimmed_string_storage() {
    let raw = "SBER".to_owned();
    let pointer = raw.as_ptr();
    let identifier = SecId::try_from(raw).expect("valid identifier");
    assert_eq!(identifier.as_str().as_ptr(), pointer);
    assert_eq!(format!("{identifier:?}"), "SecId(\"SBER\")");
}
