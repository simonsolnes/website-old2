use std::collections::HashMap;

use crate::parse::comb::either_of;
use crate::parse::comb::{either, map, optional, result, result_halt};
use crate::parse::repeat::repeat_any;
use crate::parse::repeat::separated_items;
use crate::parse::sequence::{around, between, preceded, serial, serial3, serial4};
use crate::parse::str::{
    char, char_of, literal, other_than, pop, take, take_any_while, take_some_while,
};
use crate::parse::tools::halt;
use crate::parse::Parse;

#[derive(PartialEq)]
pub enum JSON {
    UnsignedInt(usize),
    SignedInt(isize),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Object(HashMap<String, JSON>),
    Array(Vec<JSON>),
}

impl JSON {
    fn parse(input: &str) -> Parse<&str, JSON> {
        either(object, array)(input)
    }
}

fn whitespace(i: &str) -> Parse<&str, &str> {
    take_any_while(|c| match c as u32 {
        0x0020 => true, // Space
        0x000A => true, // LF
        0x000D => true, // CR
        0x0009 => true, // HT
        _ => false,
    })(i)
}

fn null(i: &str) -> Parse<&str, JSON> {
    map(literal("null"), |_| JSON::Null)(i)
}

fn bool(i: &str) -> Parse<&str, JSON> {
    either(
        map(literal("true"), |_| JSON::Bool(true)),
        map(literal("false"), |_| JSON::Bool(false)),
    )(i)
}

fn value(i: &str) -> Parse<&str, JSON> {
    between(
        whitespace,
        either_of(&[string, number, object, array, bool, null][..]),
        whitespace,
    )(i)
}

fn array(i: &str) -> Parse<&str, JSON> {
    between(
        char('['),
        map(separated_items(char(','), value), |arr| JSON::Array(arr)),
        char(']'),
    )(i)
}

fn object(i: &str) -> Parse<&str, JSON> {
    between(
        char('{'),
        map(
            separated_items(
                char(','),
                around(between(whitespace, string, whitespace), char(':'), value),
            ),
            |m| {
                let mut map = HashMap::new();
                for (ks, v) in m {
                    if let JSON::String(k) = ks {
                        map.insert(k, v);
                    } else {
                        panic!();
                    }
                }
                JSON::Object(map)
            },
        ),
        char('}'),
    )(i)
}

fn number(i: &str) -> Parse<&str, JSON> {
    result(
        serial4(
            // Negative
            map(optional(char('-')), |s| match s {
                Some('-') => '-',
                None => '+',
                _ => unreachable!(),
            }),
            // 0 or number
            either(
                map(char('0'), |c| c.to_string()),
                map(
                    serial(
                        char_of("123456789"),
                        take_any_while(|c| "0123456789".contains(c)),
                    ),
                    |(a, b)| format!("{a}{b}"),
                ),
            ),
            // Decimals
            optional(preceded(
                char('.'),
                take_some_while(|c| "0123456789".contains(c)),
            )),
            // Exponent
            optional(map(
                serial3(
                    either(char('e'), char('E')),
                    optional(either(char('+'), char('-'))),
                    take_some_while(|c| "0123456789".contains(c)),
                ),
                |(e, os, n)| {
                    if let Some(s) = os {
                        format!("{e}{s}{n}")
                    } else {
                        format!("{e}{n}")
                    }
                },
            )),
        ),
        |res| -> Result<JSON, &str> {
            match res {
                // Integer
                (sign, number, None, None) => {
                    if let Ok(n) = number.parse() {
                        match sign {
                            '+' => Ok(JSON::UnsignedInt(n)),
                            '-' => Ok(JSON::SignedInt(-(n as isize))),
                            _ => unreachable!(),
                        }
                    } else {
                        Err("fisk")
                    }
                }

                // Float
                (sign, number, Some(decimal), exponent) => {
                    let exponent = exponent.unwrap_or("".to_string());
                    if let Ok(n) = format!("{sign}{number}.{decimal}{exponent}").parse() {
                        Ok(JSON::Float(n))
                    } else {
                        Err("fisk")
                    }
                }
                _ => unreachable!(),
            }
        },
        "e",
    )(i)
}

// JSON string
// Currently accepts raw UTF-8
fn string(i: &str) -> Parse<&str, JSON> {
    map(
        between(
            char('"'),
            repeat_any(either(
                // Normal characters
                map(other_than("\"\\"), |s: &str| s.to_string()),
                // Backslash escaped
                preceded(
                    char('\\'),
                    halt(
                        "expected legal escaped character",
                        either(
                            // Escaped characters
                            result(
                                pop,
                                |c| {
                                    match match c {
                                        '"' => Ok('"'),    // Quote
                                        '\\' => Ok('\\'),  // Backslash
                                        '/' => Ok('/'),    // Farward slash
                                        'b' => Ok('\x08'), // BS Backspace
                                        'f' => Ok('\x0C'), // FF Form feed
                                        'n' => Ok('\n'),   // LF Line Feed
                                        'r' => Ok('\r'),   // CR Carriage Return
                                        't' => Ok('\t'),   // HT Horizontal Tab
                                        _ => Err(()),
                                    } {
                                        Ok(c) => Ok(c.to_string()),
                                        Err(e) => Err(e),
                                    }
                                },
                                "escaped character",
                            ),
                            // UTF-8 hex characters
                            preceded(
                                char('u'),
                                result_halt(
                                    take(4),
                                    |h| {
                                        println!("h: {h}");
                                        Ok(char::from_u32(u32::from_str_radix(h, 16).or(Err(()))?)
                                            .ok_or(())?
                                            .to_string())
                                    },
                                    "hex formatted unicode",
                                ),
                            ),
                        ),
                    ),
                ),
            )),
            char('"'),
        ),
        |s| JSON::String(s.join("")),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null() {
        println!("{:?}", null("null"));
        assert!(true);
    }

    #[test]
    fn test_bool() {
        assert_eq!(bool("truem"), Parse::Success(JSON::Bool(true), "m"));
        assert_eq!(bool("false"), Parse::Success(JSON::Bool(false), ""));
        assert!(bool("s").is_err());
    }

    #[test]
    fn test_object() {
        let mut expected = HashMap::new();
        expected.insert("something".to_string(), JSON::Bool(false));
        let mut fisk = HashMap::new();
        fisk.insert("fisk".to_string(), JSON::UnsignedInt(3));
        expected.insert(
            "something else".to_string(),
            JSON::Array(vec![
                JSON::Bool(true),
                JSON::Bool(false),
                JSON::String("hello".to_string()),
                JSON::UnsignedInt(3),
                JSON::Object(fisk),
            ]),
        );

        let result = object("{\"something\": false, \"something else\": [true, false, \"hello\", 3, {\"fisk\": 3}]}");
        assert_eq!(result, Parse::Success(JSON::Object(expected), ""));
    }
}
