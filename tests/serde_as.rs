mod utils;

use crate::utils::{
    check_deserialization, check_error_deserialization, check_serialization, is_equal,
};
use serde::{Deserialize, Serialize};
use serde_with::{
    As, BytesOrString, DefaultOnError, DisplayFromStr, DurationSeconds, DurationSecondsWithFrac,
    Flexible, Integer, NoneAsEmptyString, Same, SameAs,
};
use std::{
    collections::{BTreeMap, HashMap, LinkedList, VecDeque},
    rc::Rc,
    sync::Arc,
};

#[test]
fn test_display_fromstr() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct {
        #[serde(with = "As::<DisplayFromStr>")]
        value: u32,
    };

    is_equal(Struct { value: 123 }, r#"{"value":"123"}"#);
}

#[test]
fn test_tuples() {
    use std::net::IpAddr;
    let ip = "1.2.3.4".parse().unwrap();

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct1 {
        #[serde(with = "As::<(DisplayFromStr,)>")]
        values: (u32,),
    };
    is_equal(Struct1 { values: (1,) }, r#"{"values":["1"]}"#);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct2a {
        #[serde(with = "As::<(DisplayFromStr, DisplayFromStr)>")]
        values: (u32, IpAddr),
    };
    is_equal(
        Struct2a {
            values: (555_888, ip),
        },
        r#"{"values":["555888","1.2.3.4"]}"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct2b {
        #[serde(with = "As::<(SameAs<u32>, DisplayFromStr)>")]
        values: (u32, IpAddr),
    };
    is_equal(
        Struct2b { values: (987, ip) },
        r#"{"values":[987,"1.2.3.4"]}"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct2c {
        #[serde(with = "As::<(Same, DisplayFromStr)>")]
        values: (u32, IpAddr),
    };
    is_equal(
        Struct2c { values: (987, ip) },
        r#"{"values":[987,"1.2.3.4"]}"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct6 {
        #[serde(with = "As::<(Same, Same, Same, Same, Same, Same)>")]
        values: (u8, u16, u32, i8, i16, i32),
    };
    is_equal(
        Struct6 {
            values: (8, 16, 32, -8, 16, -32),
        },
        r#"{"values":[8,16,32,-8,16,-32]}"#,
    );
}

#[test]
fn test_arrays() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct0a {
        #[serde(with = "As::<[DisplayFromStr; 0]>")]
        values: [u32; 0],
    };
    is_equal(Struct0a { values: [] }, r#"{"values":[]}"#);

    // Test "non-matching" types.
    // Arrays of size 0 should allow all convertions
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct0b {
        #[serde(with = "As::<[u8; 0]>")]
        values: [String; 0],
    };
    is_equal(Struct0b { values: [] }, r#"{"values":[]}"#);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct1 {
        #[serde(with = "As::<[DisplayFromStr; 1]>")]
        values: [u32; 1],
    };
    is_equal(Struct1 { values: [1] }, r#"{"values":["1"]}"#);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct2 {
        #[serde(with = "As::<[Same; 2]>")]
        values: [u32; 2],
    };
    is_equal(Struct2 { values: [11, 22] }, r#"{"values":[11,22]}"#);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct32 {
        #[serde(with = "As::<[Same; 32]>")]
        values: [u32; 32],
    };
    is_equal(
        Struct32 {
            values: [
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                23, 24, 25, 26, 27, 28, 29, 30, 31,
            ],
        },
        r#"{"values":[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31]}"#,
    );
}

#[test]
fn test_map_as_tuple_list() {
    use std::net::IpAddr;
    let ip = "1.2.3.4".parse().unwrap();
    let ip2 = "255.255.255.255".parse().unwrap();

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructBTree {
        #[serde(with = "As::<Vec<(DisplayFromStr, DisplayFromStr)>>")]
        values: BTreeMap<u32, IpAddr>,
    };

    let map: BTreeMap<_, _> = vec![(1, ip), (10, ip), (200, ip2)].into_iter().collect();
    is_equal(
        StructBTree {
            values: map.clone(),
        },
        r#"{"values":[["1","1.2.3.4"],["10","1.2.3.4"],["200","255.255.255.255"]]}"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructBTree2 {
        #[serde(with = "As::<Vec<(Same, DisplayFromStr)>>")]
        values: BTreeMap<u32, IpAddr>,
    };

    is_equal(
        StructBTree2 { values: map },
        r#"{"values":[[1,"1.2.3.4"],[10,"1.2.3.4"],[200,"255.255.255.255"]]}"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructHash {
        #[serde(with = "As::<Vec<(DisplayFromStr, DisplayFromStr)>>")]
        values: HashMap<u32, IpAddr>,
    };

    // HashMap serialization tests with more than 1 entry are unrelyable
    let map1: HashMap<_, _> = vec![(200, ip2)].into_iter().collect();
    let map: HashMap<_, _> = vec![(1, ip), (10, ip), (200, ip2)].into_iter().collect();
    is_equal(
        StructHash {
            values: map1.clone(),
        },
        r#"{"values":[["200","255.255.255.255"]]}"#,
    );
    check_deserialization(
        StructHash {
            values: map.clone(),
        },
        r#"{"values":[["1","1.2.3.4"],["10","1.2.3.4"],["200","255.255.255.255"]]}"#,
    );
    check_error_deserialization::<StructHash>(
        r#"{"values":{"200":"255.255.255.255"}}"#,
        "invalid type: map, expected a sequence at line 1 column 10",
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructHash2 {
        #[serde(with = "As::<Vec<(Same, DisplayFromStr)>>")]
        values: HashMap<u32, IpAddr>,
    };

    is_equal(
        StructHash2 { values: map1 },
        r#"{"values":[[200,"255.255.255.255"]]}"#,
    );
    check_deserialization(
        StructHash2 { values: map },
        r#"{"values":[[1,"1.2.3.4"],[10,"1.2.3.4"],[200,"255.255.255.255"]]}"#,
    );
    check_error_deserialization::<StructHash2>(
        r#"{"values":1}"#,
        "invalid type: integer `1`, expected a sequence at line 1 column 11",
    );
}

#[test]
fn test_tuple_list_as_map() {
    use std::{collections::HashMap, net::IpAddr};
    let ip = "1.2.3.4".parse().unwrap();
    let ip2 = "255.255.255.255".parse().unwrap();

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructHashMap {
        #[serde(with = "As::<HashMap<DisplayFromStr, DisplayFromStr>>")]
        values: Vec<(u32, IpAddr)>,
    };

    is_equal(
        StructHashMap {
            values: vec![(1, ip), (10, ip), (200, ip2)],
        },
        r#"{"values":{"1":"1.2.3.4","10":"1.2.3.4","200":"255.255.255.255"}}"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructBTreeMap {
        #[serde(with = "As::<BTreeMap<DisplayFromStr, DisplayFromStr>>")]
        values: Vec<(u32, IpAddr)>,
    };

    is_equal(
        StructBTreeMap {
            values: vec![(1, ip), (10, ip), (200, ip2)],
        },
        r#"{"values":{"1":"1.2.3.4","10":"1.2.3.4","200":"255.255.255.255"}}"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructDeque {
        #[serde(with = "As::<BTreeMap<DisplayFromStr, DisplayFromStr>>")]
        values: VecDeque<(u32, IpAddr)>,
    };

    is_equal(
        StructDeque {
            values: vec![(1, ip), (10, ip), (200, ip2)].into(),
        },
        r#"{"values":{"1":"1.2.3.4","10":"1.2.3.4","200":"255.255.255.255"}}"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructLinkedList {
        #[serde(with = "As::<HashMap<DisplayFromStr, DisplayFromStr>>")]
        values: LinkedList<(u32, IpAddr)>,
    };

    is_equal(
        StructDeque {
            values: vec![(1, ip), (10, ip), (200, ip2)].into_iter().collect(),
        },
        r#"{"values":{"1":"1.2.3.4","10":"1.2.3.4","200":"255.255.255.255"}}"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructOption {
        #[serde(with = "As::<HashMap<DisplayFromStr, DisplayFromStr>>")]
        values: Option<(u32, IpAddr)>,
    };

    is_equal(
        StructOption {
            values: Some((1, ip)),
        },
        r#"{"values":{"1":"1.2.3.4"}}"#,
    );
    is_equal(StructOption { values: None }, r#"{"values":{}}"#);
}

#[test]
fn test_none_as_empty_string() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct {
        #[serde(with = "As::<NoneAsEmptyString>")]
        value: Option<String>,
    };

    is_equal(Struct { value: None }, r#"{"value":""}"#);
    is_equal(
        Struct {
            value: Some("Hello".to_string()),
        },
        r#"{"value":"Hello"}"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructRc {
        #[serde(with = "As::<NoneAsEmptyString>")]
        value: Option<Rc<str>>,
    };

    is_equal(StructRc { value: None }, r#"{"value":""}"#);
    is_equal(
        StructRc {
            value: Some("Hello".into()),
        },
        r#"{"value":"Hello"}"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructArc {
        #[serde(with = "As::<NoneAsEmptyString>")]
        value: Option<Arc<str>>,
    };

    is_equal(StructArc { value: None }, r#"{"value":""}"#);
    is_equal(
        StructArc {
            value: Some("Hello".into()),
        },
        r#"{"value":"Hello"}"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructBox {
        #[serde(with = "As::<NoneAsEmptyString>")]
        value: Option<Box<str>>,
    };

    is_equal(StructBox { value: None }, r#"{"value":""}"#);
    is_equal(
        StructBox {
            value: Some("Hello".into()),
        },
        r#"{"value":"Hello"}"#,
    );
}

#[test]
fn test_default_on_error() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct {
        #[serde(with = "As::<DefaultOnError<DisplayFromStr>>")]
        value: u32,
    };

    // Normal
    is_equal(Struct { value: 123 }, r#"{"value":"123"}"#);
    is_equal(Struct { value: 0 }, r#"{"value":"0"}"#);
    // Error cases
    check_deserialization(Struct { value: 0 }, r#"{"value":""}"#);
    check_deserialization(Struct { value: 0 }, r#"{"value":"12+3"}"#);
    check_deserialization(Struct { value: 0 }, r#"{"value":"abc"}"#);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct2 {
        #[serde(with = "As::<DefaultOnError<Vec<DisplayFromStr>>>")]
        value: Vec<u32>,
    };

    // Normal
    is_equal(
        Struct2 {
            value: vec![1, 2, 3],
        },
        r#"{"value":["1","2","3"]}"#,
    );
    is_equal(Struct2 { value: vec![] }, r#"{"value":[]}"#);
    // Error cases
    check_deserialization(Struct2 { value: vec![] }, r#"{"value":2}"#);
    check_deserialization(Struct2 { value: vec![] }, r#"{"value":"notalist"}"#);
    // TODO why does this result in
    // thread 'test_default_on_error' panicked at 'called `Result::unwrap()` on an `Err` value: Error("expected `,` or `}`", line: 1, column: 10)', tests/utils.rs:32:9
    // check_deserialization(Struct2 { value: vec![] }, r#"{"value":{}}"#);
    check_deserialization(Struct2 { value: vec![] }, r#"{"value":}"#);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct3 {
        #[serde(with = "As::<Vec<DefaultOnError<DisplayFromStr>>>")]
        value: Vec<u32>,
    };

    // Normal
    is_equal(
        Struct3 {
            value: vec![1, 2, 3],
        },
        r#"{"value":["1","2","3"]}"#,
    );
    is_equal(Struct3 { value: vec![] }, r#"{"value":[]}"#);
    // Error cases
    check_deserialization(
        Struct3 {
            value: vec![0, 0, 0],
        },
        r#"{"value":[2,3,4]}"#,
    );
    check_deserialization(Struct3 { value: vec![0, 0] }, r#"{"value":["AA",5]}"#);
}

#[test]
fn test_bytes_or_string() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct {
        #[serde(with = "As::<BytesOrString>")]
        value: Vec<u8>,
    };

    is_equal(
        Struct {
            value: vec![1, 2, 3],
        },
        r#"{"value":[1,2,3]}"#,
    );
    check_deserialization(
        Struct {
            value: vec![72, 101, 108, 108, 111],
        },
        r#"{"value":"Hello"}"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructVec {
        #[serde(with = "As::<Vec<BytesOrString>>")]
        value: Vec<Vec<u8>>,
    };

    is_equal(
        StructVec {
            value: vec![vec![1, 2, 3]],
        },
        r#"{"value":[[1,2,3]]}"#,
    );
    check_deserialization(
        StructVec {
            value: vec![
                vec![72, 101, 108, 108, 111],
                vec![87, 111, 114, 108, 100],
                vec![1, 2, 3],
            ],
        },
        r#"{"value":["Hello","World",[1,2,3]]}"#,
    );
}

#[test]
fn test_duration_seconds() {
    use std::time::Duration;
    let zero = Duration::new(0, 0);
    let one_second = Duration::new(1, 0);
    let half_second = Duration::new(0, 500_000_000);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructIntStrict {
        #[serde(with = "As::<DurationSeconds>")]
        value: Duration,
    };

    is_equal(StructIntStrict { value: zero }, r#"{"value":0}"#);
    is_equal(StructIntStrict { value: one_second }, r#"{"value":1}"#);
    check_serialization(StructIntStrict { value: half_second }, r#"{"value":1}"#);
    check_error_deserialization::<StructIntStrict>(
        r#"{"value":"1"}"#,
        r#"invalid type: string "1", expected u64 at line 1 column 12"#,
    );
    check_error_deserialization::<StructIntStrict>(
        r#"{"value":-1}"#,
        r#"invalid value: integer `-1`, expected u64 at line 1 column 11"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructIntFlexible {
        #[serde(with = "As::<DurationSeconds<Integer, Flexible>>")]
        value: Duration,
    };

    is_equal(StructIntFlexible { value: zero }, r#"{"value":0}"#);
    is_equal(StructIntFlexible { value: one_second }, r#"{"value":1}"#);
    check_serialization(StructIntFlexible { value: half_second }, r#"{"value":1}"#);
    check_deserialization(
        StructIntFlexible { value: half_second },
        r#"{"value":"0.5"}"#,
    );
    check_deserialization(StructIntFlexible { value: one_second }, r#"{"value":"1"}"#);
    check_deserialization(StructIntFlexible { value: zero }, r#"{"value":"0"}"#);
    check_error_deserialization::<StructIntFlexible>(
        r#"{"value":"a"}"#,
        r#"invalid value: string "a", expected an integer, a float, or a string containing a number at line 1 column 12"#,
    );
    check_error_deserialization::<StructIntFlexible>(
        r#"{"value":-1}"#,
        r#"Negative values are not supported for Duration. Found -1 at line 1 column 11"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Structf64Strict {
        #[serde(with = "As::<DurationSeconds<f64>>")]
        value: Duration,
    };

    is_equal(Structf64Strict { value: zero }, r#"{"value":0.0}"#);
    is_equal(Structf64Strict { value: one_second }, r#"{"value":1.0}"#);
    check_serialization(Structf64Strict { value: half_second }, r#"{"value":1.0}"#);
    check_deserialization(Structf64Strict { value: half_second }, r#"{"value":0.5}"#);
    check_error_deserialization::<Structf64Strict>(
        r#"{"value":"1"}"#,
        r#"invalid type: string "1", expected f64 at line 1 column 12"#,
    );
    check_error_deserialization::<Structf64Strict>(
        r#"{"value":-1.0}"#,
        r#"underflow when converting float to duration at line 1 column 14"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Structf64Flexible {
        #[serde(with = "As::<DurationSeconds<f64, Flexible>>")]
        value: Duration,
    };

    is_equal(Structf64Flexible { value: zero }, r#"{"value":0.0}"#);
    is_equal(Structf64Flexible { value: one_second }, r#"{"value":1.0}"#);
    check_serialization(Structf64Flexible { value: half_second }, r#"{"value":1.0}"#);
    check_deserialization(
        Structf64Flexible { value: half_second },
        r#"{"value":"0.5"}"#,
    );
    check_deserialization(Structf64Flexible { value: one_second }, r#"{"value":"1"}"#);
    check_deserialization(Structf64Flexible { value: zero }, r#"{"value":"0"}"#);
    check_error_deserialization::<Structf64Flexible>(
        r#"{"value":"a"}"#,
        r#"invalid value: string "a", expected an integer, a float, or a string containing a number at line 1 column 12"#,
    );
    check_error_deserialization::<Structf64Flexible>(
        r#"{"value":-1}"#,
        r#"Negative values are not supported for Duration. Found -1 at line 1 column 11"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructStringStrict {
        #[serde(with = "As::<DurationSeconds<String>>")]
        value: Duration,
    };

    is_equal(StructStringStrict { value: zero }, r#"{"value":"0"}"#);
    is_equal(StructStringStrict { value: one_second }, r#"{"value":"1"}"#);
    check_serialization(
        StructStringStrict { value: half_second },
        r#"{"value":"1"}"#,
    );
    check_error_deserialization::<StructStringStrict>(
        r#"{"value":1}"#,
        // TODO the error message should not talk about "json object"
        r#"invalid type: integer `1`, expected valid json object at line 1 column 10"#,
    );
    check_error_deserialization::<StructStringStrict>(
        r#"{"value":-1}"#,
        r#"invalid type: integer `-1`, expected valid json object at line 1 column 11"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructStringFlexible {
        #[serde(with = "As::<DurationSeconds<String, Flexible>>")]
        value: Duration,
    };

    is_equal(StructStringFlexible { value: zero }, r#"{"value":"0"}"#);
    is_equal(
        StructStringFlexible { value: one_second },
        r#"{"value":"1"}"#,
    );
    check_serialization(
        StructStringFlexible { value: half_second },
        r#"{"value":"1"}"#,
    );
    check_deserialization(
        StructStringFlexible { value: half_second },
        r#"{"value":"0.5"}"#,
    );
    check_deserialization(
        StructStringFlexible { value: one_second },
        r#"{"value":"1"}"#,
    );
    check_deserialization(StructStringFlexible { value: zero }, r#"{"value":"0"}"#);
    check_error_deserialization::<StructStringFlexible>(
        r#"{"value":"a"}"#,
        r#"invalid value: string "a", expected an integer, a float, or a string containing a number at line 1 column 12"#,
    );
    check_error_deserialization::<StructStringFlexible>(
        r#"{"value":-1}"#,
        r#"Negative values are not supported for Duration. Found -1 at line 1 column 11"#,
    );
}

#[test]
fn test_duration_seconds_with_frac() {
    use std::time::Duration;
    let zero = Duration::new(0, 0);
    let one_second = Duration::new(1, 0);
    let half_second = Duration::new(0, 500_000_000);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Structf64Strict {
        #[serde(with = "As::<DurationSecondsWithFrac<f64>>")]
        value: Duration,
    };

    is_equal(Structf64Strict { value: zero }, r#"{"value":0.0}"#);
    is_equal(Structf64Strict { value: one_second }, r#"{"value":1.0}"#);
    is_equal(Structf64Strict { value: half_second }, r#"{"value":0.5}"#);
    check_error_deserialization::<Structf64Strict>(
        r#"{"value":"1"}"#,
        r#"invalid type: string "1", expected f64 at line 1 column 12"#,
    );
    check_error_deserialization::<Structf64Strict>(
        r#"{"value":-1.0}"#,
        r#"underflow when converting float to duration at line 1 column 14"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Structf64Flexible {
        #[serde(with = "As::<DurationSecondsWithFrac<f64, Flexible>>")]
        value: Duration,
    };

    is_equal(Structf64Flexible { value: zero }, r#"{"value":0.0}"#);
    is_equal(Structf64Flexible { value: one_second }, r#"{"value":1.0}"#);
    is_equal(Structf64Flexible { value: half_second }, r#"{"value":0.5}"#);
    check_deserialization(Structf64Flexible { value: one_second }, r#"{"value":"1"}"#);
    check_deserialization(Structf64Flexible { value: zero }, r#"{"value":"0"}"#);
    check_error_deserialization::<Structf64Flexible>(
        r#"{"value":"a"}"#,
        r#"invalid value: string "a", expected an integer, a float, or a string containing a number at line 1 column 12"#,
    );
    check_error_deserialization::<Structf64Flexible>(
        r#"{"value":-1}"#,
        r#"Negative values are not supported for Duration. Found -1 at line 1 column 11"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructStringStrict {
        #[serde(with = "As::<DurationSecondsWithFrac<String>>")]
        value: Duration,
    };

    is_equal(StructStringStrict { value: zero }, r#"{"value":"0"}"#);
    is_equal(StructStringStrict { value: one_second }, r#"{"value":"1"}"#);
    is_equal(
        StructStringStrict { value: half_second },
        r#"{"value":"0.5"}"#,
    );
    check_error_deserialization::<StructStringStrict>(
        r#"{"value":1}"#,
        r#"invalid type: integer `1`, expected a string at line 1 column 10"#,
    );
    check_error_deserialization::<StructStringStrict>(
        r#"{"value":-1}"#,
        r#"invalid type: integer `-1`, expected a string at line 1 column 11"#,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StructStringFlexible {
        #[serde(with = "As::<DurationSecondsWithFrac<String, Flexible>>")]
        value: Duration,
    };

    is_equal(StructStringFlexible { value: zero }, r#"{"value":"0"}"#);
    is_equal(
        StructStringFlexible { value: one_second },
        r#"{"value":"1"}"#,
    );
    is_equal(
        StructStringFlexible { value: half_second },
        r#"{"value":"0.5"}"#,
    );
    check_deserialization(StructStringFlexible { value: zero }, r#"{"value":"0"}"#);
    check_error_deserialization::<StructStringFlexible>(
        r#"{"value":"a"}"#,
        r#"invalid value: string "a", expected an integer, a float, or a string containing a number at line 1 column 12"#,
    );
    check_error_deserialization::<StructStringFlexible>(
        r#"{"value":-1}"#,
        r#"Negative values are not supported for Duration. Found -1 at line 1 column 11"#,
    );
}
