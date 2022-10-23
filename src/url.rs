use nom::error::Error;
///
///  protocol    password       port  query-parameters        
///     │           │            │          │                 
///     │   username│   hostname │pathname  │   fragment      
///     │      │    │      │     │   │      │    │            
///    ┌┴─┐   ┌┴─┐ ┌┴─┐ ┌──┴───┐ ├┐┌─┴──┐ ┌─┴─┐ ┌┴─┐          
///    http://user:pass@site.com:80/pa/th?q=val&s=x#hash          
///    │                └────┬────┘└────┬─────┘    │          
///    │                    host       path        │          
///    └───────────────────┬───────────────────────┘          
///                        │                                  
///                       href                                
///

#[allow(dead_code)]
#[allow(unused_variables)]
use nom::IResult;
use nom::Parser;
use std::collections::HashMap;
use std::str::Utf8Error;

/// Uri syntax components
/// https://www.rfc-editor.org/rfc/rfc3986#section-3

type Path = Vec<String>;
type Query = HashMap<String, String>;

#[derive(PartialEq, Debug)]
enum Target {
    Origin(Absolute<Path>, Option<Query>),
    Absolute(URI),
    Authority(Host),
    Asterix,
}

#[derive(PartialEq, Debug)]
struct Relative<T>(T);

#[derive(PartialEq, Debug)]
struct Absolute<T>(T);

#[derive(PartialEq)]
struct URL {}

#[derive(PartialEq, Debug)]
struct URI {}

#[derive(PartialEq, Debug)]
struct Host {}

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take, take_till, take_until, take_while, take_while1};
use nom::character::complete::{anychar, char, one_of, satisfy};
use nom::combinator::{
    all_consuming, cut, eof, map, map_parser, map_res, not, opt, peek, recognize, success,
};
use nom::multi::{many0, many1, separated_list0, separated_list1};
use nom::sequence::{preceded, separated_pair, terminated, tuple};

pub trait Parsable {
    type Output;
    fn nom_parse(i: &str) -> IResult<&str, Self::Output>;

    fn parse(i: &str) -> Result<Self::Output, ()> {
        match Self::nom_parse(i) {
            Ok((_, result)) => Ok(result),
            Err(_) => {
                eprintln!("parse error");
                Err(())
            }
        }
    }
}

/// Contains all characters in ascii, separated into url charsets
#[allow(dead_code)]
mod ascii_charsets {
    pub const NUMERIC: &'static str = "0123456789";
    pub const ALPHA_SMALL: &'static str = "abcdefghijklmnopqrstuvwxyz";
    pub const ALPHA_CAPITAL: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    pub const CONTROL: &'static str = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\
                                      \x0A\x0B\x0C\x0D\x0E\x0F\x10\x11\x12\x13\
                                      \x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\
                                      \x1d\x1E\x1F";

    pub const URL_UNRESERVED: &'static str = "-._~";
    pub const GEN_DELIMS: &'static str = ":/?#[]@";
    pub const SUB_DELIMS: &'static str = "!$&'()*+,;=";
    pub const PERCENT: char = '%';
    pub const URL_ILLEGAL: &'static str = " \"<>\\^`}{|";
}

/// Function returns true if c is contained in ascii and in either of these:
/// `CONTROL`: Invisible characters
/// `GEN_DELIMS`: General delimiters
/// `SUB_DELIMS`: Sub delimiters
/// `URL_ILLEGAL`: Illegal characters that will stop url parse
fn is_url_terminative(c: char) -> bool {
    let c = c as u32;
    c < 128
        && c > 47
        && ((c > 57 && c < 65) || c == 96 || (c > 90 && c < 95) || c > 122 && c != 126)
        || c < 45
        || c == 47
}

fn url_end(i: &str) -> IResult<&str, ()> {
    match eof::<&str, ()>(i) {
        Ok(_) => Ok((i, ())),
        Err(_) => match satisfy::<_, _, ()>(is_url_terminative)(i) {
            Ok(_) => Ok((i, ())),
            Err(_) => Err(nom::Err::Error(Error {
                input: i,
                code: nom::error::ErrorKind::Permutation,
            })),
        },
    }
}
fn percent_decode(i: &str) -> IResult<&str, String> {
    map(
        many1(alt((
            // UTF-8 encoded text
            map(take_while1(|c: char| !is_url_terminative(c)), |s: &str| {
                s.to_string()
            }),
            // Convert '+' to space
            map(char('+'), |_| " ".to_string()),
            // Percent encoding to UTF-8
            map_res(
                map_parser(
                    many1(preceded(char('%'), cut(take(2_usize)))),
                    |h: Vec<&str>| match h
                        .iter()
                        .map(|hs| u8::from_str_radix(*hs, 16))
                        .collect::<Result<Vec<u8>, std::num::ParseIntError>>()
                    {
                        Ok(r) => Ok((vec![], r)),
                        Err(_) => Err(nom::Err::Failure(nom::error::Error {
                            input: i,
                            code: nom::error::ErrorKind::Permutation,
                        })),
                    },
                ),
                |s| -> Result<String, Utf8Error> { Ok(std::str::from_utf8(&s)?.to_string()) },
            ),
        ))),
        |r| r.join(""),
    )(i)
}

/// Reseved characters
/// https://www.rfc-editor.org/rfc/rfc3986#section-2.2

// impl Parsable for Query {
//     type Output = Query;
//     fn nom_parse<'a>(i: &'a str) -> IResult<&str, Query> {}
// }

impl Parsable for Target {
    type Output = Self;
    fn nom_parse(i: &str) -> IResult<&str, Self> {
        alt((
            map(tuple((char('*'), url_end)), |_| Self::Asterix),
            map(tuple((char('*'), url_end)), |_| Self::Asterix),
        ))(i)
    }
}

impl Parsable for Relative<Path> {
    type Output = Relative<Path>;
    fn nom_parse<'a>(i: &'a str) -> IResult<&str, Relative<Path>> {
        fn take_maybe_leading_slash(i: &str) -> IResult<&str, Option<char>> {
            opt(char('/'))(i)
        }
        let (sur, res) = separated_list0(char('/'), percent_decode)(i)?;
        let (stripped, _) = take_maybe_leading_slash(sur)?;
        Ok((stripped, Relative(res)))
    }
}

impl Parsable for Absolute<Path> {
    type Output = Absolute<Path>;
    fn nom_parse(i: &str) -> IResult<&str, Absolute<Path>> {
        map(
            preceded(char('/'), Relative::<Path>::nom_parse),
            |Relative(p)| Absolute(p),
        )(i)
    }
}

impl Parsable for Option<Query> {
    type Output = Option<Query>;
    fn nom_parse(i: &str) -> IResult<&str, Option<Query>> {
        match char::<&str, ()>('?')(i) {
            Ok(_) => match Query::nom_parse(i) {
                Ok((sur, res)) => Ok((sur, Some(res))),
                Err(_) => Err(nom::Err::Error(Error {
                    input: i,
                    code: nom::error::ErrorKind::Permutation,
                })),
            },
            Err(_) => success(None)(i),
        }
    }
}

impl Parsable for Query {
    type Output = Query;
    fn nom_parse(i: &str) -> IResult<&str, Query> {
        map(
            preceded(
                char('?'),
                separated_list1(
                    char('&'),
                    separated_pair(percent_decode, char('='), percent_decode),
                ),
            ),
            |l: Vec<(String, String)>| l.into_iter().collect(),
        )(i)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_percent_decode() {
        let input1 = "stf%C2%A8%C3%92%C2%A8%C3%94%E2%80%A1%EF%AC%82%E2%80%BA%EF%AC%81%C2%B0%C2%B0%EF%AC%81%EF%AC%81%E2%88%8F%CB%9D%CB%87%C3%8E%C3%8E%C3%93";
        let expected1 = "stf¨Ò¨Ô‡ﬂ›ﬁ°°ﬁﬁ∏˝ˇÎÎÓ".to_string();
        let (_, result1) = percent_decode(input1).unwrap();
        assert_eq!(result1, expected1);
        assert!(percent_decode("fÔes%C2%A8mk%e").is_err());
    }
    #[test]
    fn test_relative_path() {
        assert_eq!(Relative::<Path>::parse("").unwrap(), Relative(vec![]));
        assert_eq!(
            Relative::<Path>::parse("hello").unwrap(),
            Relative(vec!["hello".to_owned()])
        );
        assert_eq!(
            Relative::<Path>::parse("hello/there").unwrap(),
            Relative(vec!["hello".to_owned(), "there".to_owned()])
        );
        assert_eq!(
            Relative::<Path>::nom_parse("hello/there ").unwrap(),
            (" ", Relative(vec!["hello".to_owned(), "there".to_owned()]))
        );
        assert_eq!(
            Relative::<Path>::nom_parse("hello/there/?s ").unwrap(),
            (
                "?s ",
                Relative(vec!["hello".to_owned(), "there".to_owned()])
            )
        );
        assert_eq!(
            Relative::<Path>::parse("hello/there/").unwrap(),
            Relative(vec!["hello".to_owned(), "there".to_owned()])
        );
        assert_eq!(Relative::<Path>::parse("/").unwrap(), Relative(vec![]));
    }
    #[test]
    fn test_absolute_path() {
        assert!(Absolute::<Path>::parse("hello").is_err());
        assert_eq!(
            Absolute::<Path>::parse("/hello").unwrap(),
            Absolute(vec!["hello".to_owned()])
        );
        assert_eq!(
            Absolute::<Path>::parse("/hello/there").unwrap(),
            Absolute(vec!["hello".to_owned(), "there".to_owned()])
        );
        assert_eq!(
            Absolute::<Path>::parse("/hello/there/").unwrap(),
            Absolute(vec!["hello".to_owned(), "there".to_owned()])
        );
    }
    #[test]
    fn test_query() {
        let mut t_hei_der: HashMap<String, String> = HashMap::new();
        t_hei_der.insert("hei".to_string(), "der".to_string());
        let mut t_double: HashMap<String, String> = HashMap::new();
        t_double.insert("hei".to_string(), "der".to_string());
        t_double.insert("ddd".to_string(), "padeg".to_string());

        assert_eq!(Query::parse("?hei=der").unwrap(), t_hei_der);
        assert_eq!(Query::parse("?hei=der&ddd=padeg").unwrap(), t_double);
        assert_eq!(Query::nom_parse("?hei=der"), Ok(("", t_hei_der.clone())));
        assert_eq!(
            Query::nom_parse("?hei=der&ddd=padeg"),
            Ok(("", t_double.clone()))
        );
        assert_eq!(Query::nom_parse("?hei=der "), Ok((" ", t_hei_der.clone())));
        assert_eq!(
            Query::nom_parse("?hei=der&ddd=padeg\n"),
            Ok(("\n", t_double.clone()))
        );
        assert!(Query::parse("?hei=").is_err());
        assert!(Query::parse("?=").is_err());
        assert!(Query::parse("?").is_err());
        assert!(Query::parse("?=3").is_err());
    }
    #[test]
    fn test_optional_query() {
        assert_eq!(Option::<Query>::parse("").unwrap(), None);
        assert!(Option::<Query>::parse("?=3").is_err());
        assert!(Option::<Query>::parse("?&").is_err());

        let mut t_single: HashMap<String, String> = HashMap::new();
        t_single.insert("mes".to_string(), "abc".to_string());
        let mut t_double: HashMap<String, String> = HashMap::new();
        t_double.insert("__".to_string(), "~".to_string());
        t_double.insert("r".to_string(), "ff".to_string());
        assert_eq!(
            Option::<Query>::parse("?mes=abc").unwrap(),
            Some(t_single.clone())
        );
        assert_eq!(
            Option::<Query>::parse("?__=~&r=ff").unwrap(),
            Some(t_double.clone())
        );
    }
    #[test]
    fn test_target_asterix() {
        //assert_eq!(Path::parse("hei"), 2);
        assert_eq!(Target::parse("*").unwrap(), Target::Asterix);
        assert_eq!(Target::parse("* ").unwrap(), Target::Asterix);
        assert_eq!(Target::nom_parse("* "), Ok((" ", Target::Asterix)));
    }
    #[test]
    fn test_target_asterix_invalid() {
        assert!(Target::parse("*s").is_err());
    }
}
