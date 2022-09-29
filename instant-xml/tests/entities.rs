use std::borrow::Cow;

use similar_asserts::assert_eq;

use instant_xml::{from_str, to_string, Error, FromXml, ToXml};

#[derive(Debug, PartialEq, Eq, FromXml, ToXml)]
#[xml(ns("URI"))]
struct StructSpecialEntities<'a> {
    string: String,
    str: &'a str,
    cow: Cow<'a, str>,
    vec: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, FromXml, ToXml)]
#[xml(ns("URI"))]
struct VecEntities<'a> {
    complex: Vec<StructSpecialEntities<'a>>,
    list1: Vec<String>,
    list2: Vec<Vec<String>>,
}

#[test]
fn vec_entities() {
    let serialized = r#"<VecEntities xmlns="URI"><complex><StructSpecialEntities><string>&lt;&gt;&amp;&quot;&apos;adsad&quot;</string><str>str</str><cow>str&amp;</cow><vec><element xmlns="">one</element><element xmlns="">two</element><element xmlns="">three</element></vec></StructSpecialEntities></complex><list1><element xmlns="">a</element><element xmlns="">b</element></list1><list2><list xmlns=""><element xmlns="">a</element><element xmlns="">b</element></list></list2></VecEntities>"#;

    let instance = VecEntities {
        complex: vec![StructSpecialEntities {
            string: String::from("<>&\"'adsad\""),
            str: "str",
            cow: Cow::Owned("str&".to_string()),
            vec: vec!["one".into(), "two".into(), "three".into()],
        }],
        list1: vec!["a".into(), "b".into()],
        list2: vec![vec!["a".into(), "b".into()]],
    };

    assert_eq!(to_string(&instance).unwrap(), serialized);
    assert_eq!(from_str(serialized), Ok(instance));
}

#[test]
fn escape_back() {
    assert_eq!(
        from_str(
            "<StructSpecialEntities xmlns=\"URI\"><string>&lt;&gt;&amp;&quot;&apos;adsad&quot;</string><str>str</str><cow>str&amp;</cow><vec><element xmlns=\"\">one</element><element xmlns=\"\">two</element><element xmlns=\"\">three</element></vec></StructSpecialEntities>"
        ),
        Ok(StructSpecialEntities {
            string: String::from("<>&\"'adsad\""),
            str: "str",
            cow: Cow::Owned("str&".to_string()),
	    vec: vec!["one".into(), "two".into(), "three".into()]
        })
    );

    // Wrong str char
    assert_eq!(
        from_str(
            "<StructSpecialEntities xmlns=\"URI\"><string>&lt;&gt;&amp;&quot;&apos;adsad&quot;</string><str>str&amp;</str><vec><element xmlns=\"\">one</element><element xmlns=\"\">two</element><element xmlns=\"\">three</element></vec></StructSpecialEntities>"
        ),
        Err::<StructSpecialEntities, _>(Error::UnexpectedValue)
    );

    // Borrowed
    let escape_back = from_str::<StructSpecialEntities>(
        "<StructSpecialEntities xmlns=\"URI\"><string>&lt;&gt;&amp;&quot;&apos;adsad&quot;</string><str>str</str><cow>str</cow><vec><element xmlns=\"\">one</element><element xmlns=\"\">two</element><element xmlns=\"\">three</element></vec></StructSpecialEntities>"
    )
    .unwrap();

    if let Cow::Owned(_) = escape_back.cow {
        panic!("Should be Borrowed")
    }

    // Owned
    let escape_back = from_str::<StructSpecialEntities>(
            "<StructSpecialEntities xmlns=\"URI\"><string>&lt;&gt;&amp;&quot;&apos;adsad&quot;</string><str>str</str><cow>str&amp;</cow><vec><element xmlns=\"\">one</element><element xmlns=\"\">two</element><element xmlns=\"\">three</element></vec></StructSpecialEntities>"
        )
        .unwrap();

    if let Cow::Borrowed(_) = escape_back.cow {
        panic!("Should be Owned")
    }
}

#[test]
fn special_entities() {
    assert_eq!(
        to_string(&StructSpecialEntities{
            string: "&\"<>\'aa".to_string(),
            str: "&\"<>\'bb",
            cow: Cow::from("&\"<>\'cc"),
        vec: vec!["one".into(), "two".into(), "three".into()]
        }).unwrap(),
        "<StructSpecialEntities xmlns=\"URI\"><string>&amp;&quot;&lt;&gt;&apos;aa</string><str>&amp;&quot;&lt;&gt;&apos;bb</str><cow>&amp;&quot;&lt;&gt;&apos;cc</cow><vec><element xmlns=\"\">one</element><element xmlns=\"\">two</element><element xmlns=\"\">three</element></vec></StructSpecialEntities>",
    );
}
